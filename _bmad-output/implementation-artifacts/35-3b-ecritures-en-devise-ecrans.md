# Story 35-3b : écritures en devise — soldes d'ouverture, formulaire et écrans

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, C-35-4, C-35-5. Suit **35-3a**
(découpe de l'ancienne 35-3 à la rédaction). **Aucune migration.** Révisée le 2026-11-14 pour la revue de Beat
(CR-2 : R3, R6, R7, R8), qui prime sur le registre antérieur ; elle ajoute une route serveur (assistant *Conversion*,
AC-2 bis). Révisée de nouveau pour la revue CR-3 de Beat, qui prime sur CR-2 : **S2** — la ligne d'écart et l'assistant
*Conversion* prennent les comptes de différence **propres** du compte en devise concerné (35-1 AC-4 bis) ; **S5** —
l'assistant TVA achat calcule la TVA sur le montant en devise et passe la ligne de TVA en CHF au même cours (AC-2) ;
**S6** — l'écran de saisie n'affiche pas les cours AFC republiés (c'est 35-2). Révisée enfin pour la revue 2 de Beat
(CR-4), qui prime sur tout ce qui précède : **T3** — la ligne ne stocke pas de devise (35-3a AC-1) : `currency` se lit
sur le compte, facultatif à l'envoi (AC-1, AC-2, AC-3) ; **T4** — cours d'ouverture proposé = cours de **clôture**
de l'exercice précédent : veille de l'ouverture (genre `day`) ou cours mensuel du **dernier mois de l'exercice
précédent** (genre `month`) (AC-1). Révisée en dernier pour la revue 3 de Beat (CR-5), qui prime sur tout : **U1** — le
cours proposé est le **cours retenu par la société** (`exchange_rates` de 35-2 : `effectiveRate` et `source` de
`lookup` ; ses champs `afcRate` et `overridden` n'existent plus, `afcRate` devient une valeur dérivée du formulaire,
AC-2), et les indicateurs se lisent sur `source` (AC-1,
AC-2, AC-5, AC-6) ; S6 tel qu'écrit (« adoption ») devient la reprise par ↑ d'un cours AFC corrigé, par société ;
**U2** — changer le compte d'une ligne réinitialise montant et cours, vers un compte CHF retire `foreign` (AC-2) ;
**U5** — A8, A12, A13, A14 approuvés ; **U6** — débit et crédit restent en devise principale (AC-1, AC-2).*

## Story

En tant que comptable en multi-devises, je veux saisir à l'écran une écriture en devise avec le cours du mois
proposé (cours mensuel AFC, ou le cours que ma société a saisi, D9), saisir mes propres cours (ou le montant CHF réel),
y ajouter d'un clic une ligne de différence de change, passer une conversion entre deux devises par l'assistant
*Conversion* (deux écritures par le compte de conversion, R7), et ouvrir mes comptes en devise avec leur solde en
devise au cours d'ouverture choisi pour l'exercice (R6), afin de tenir mes livres sans passer par l'API.

## Critères d'acceptation

1. **AC-1 — Soldes d'ouverture** (serveur et écran).
   - **Routes.** `generate_opening_balances` (`routes/opening_balances.rs` :345 → `journal_entries::create_opening_entry`
     :655) et le complément (`complete_opening_balances` :593 → `opening_complement.rs` `create_opening_complement` :460, enveloppe en portée `Screen` de
     `create_opening_complement_in_tx` :551, qui sert aussi le mode Compléter de la 34-2 sur les comptes bancaires)
     acceptent, pour un compte en devise, le **solde en devise** et un **cours**. Un compte CHF reste inchangé.
   - **DTO.** `OpeningBalanceLineRequest` et `OpeningComplementLineRequest` prennent `foreign?: { currency?, amount, rate,
     rateSource }` (forme de 35-3a AC-6 — **même type** que le `foreign` de `CreateJournalEntryLineRequest` : champs
     `String` analysés par la même fonction, `currency` facultatif — T3 : absent, la ligne prend la devise du compte ;
     présent, il doit l'égaler, `FX_CURRENCY_MISMATCH` ; jamais stocké ; les deux grilles l'envoient toujours, comme le
     formulaire, AC-2 —, d'où un **400** et non le 422 d'axum pour `rateSource: "revaluation"`,
     statut testé sur les deux routes) ; `debit`/`credit` restent les montants en **devise principale** (aujourd'hui
     **CHF** ; U6), **fournis par le client et
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
   - **Cours proposé** (R6, CR-2 — remplace « mois précédent » de CR-1 ; **T4, CR-4** : le cours d'ouverture est le
     cours de **clôture** de l'exercice précédent, pour que les valeurs CHF concordent d'un exercice à l'autre ; cette
     puce est récrite en conséquence pour le genre `month`) : cours **effectif** (U1, CR-5 : le **cours retenu par la
     société**, ligne `exchange_rates` — copie d'une valeur publiée (`source: "afc"`) ou cours saisi (`source:
     "manual"`) ; champs `effectiveRate` et `source` de `lookup`, 35-2 AC-5) du **genre d'ouverture de
     l'exercice** de l'écriture d'ouverture (`openingRateKind` du premier exercice, 35-1 AC-1, réglé à l'écran *Cours
     de change*, 35-2 AC-6 ; lu sur `GET /fiscal-years`) : genre `day` (défaut) → **cours journalier de la veille de
     l'ouverture** (`lookup` `kind=day`, `date` = `start_date` − 1 jour : le 31.12 pour un exercice civil, le 30.06
     pour un exercice au 1er juillet, le 31.03 pour un exercice au 1er avril), qui est le cours de clôture de
     l'exercice antérieur ; genre `month` → **cours mensuel du dernier mois de l'exercice précédent** (T4 — non plus du
     premier mois du nouvel exercice), c'est-à-dire du mois de la veille de l'ouverture (`lookup` `kind=month`,
     `date` = le **1er du mois de la veille**, la veille étant `start_date` − 1 jour — non « le 1er du mois de
     `start_date`, moins un jour » : pour une ouverture au 15.03, la veille est le 14.03 et le mois est **mars**, non
     février ; l'assistant de démarrage laisse choisir un début quelconque, 35-1 AC-11 (T1) : décembre pour un exercice civil,
     juin pour un exercice au 1er juillet, mars pour un exercice au 1er avril ou au 15.03), que l'exercice précédent soit tenu dans Kesh ou non — la
     règle se calcule sur la seule date d'ouverture, sans lire l'exercice précédent ; même date que `openingDate` du
     tableau annuel de 35-2 (AC-5) : la fonction pure de 35-2 est en Rust et la page des soldes d'ouverture est en
     TypeScript — elles ne se partagent pas ; la page calcule la date par **une** fonction pure TS (nom au
     développement), dont le Vitest lit la fixture commune de 35-2 AC-5, `crates/kesh-core/tests/fixtures/fx_opening_dates.json`
     (même fichier que le test Rust, aucune copie ; genres `day` et `month`, 1er janvier, 1er avril, 1er juillet,
     15.03, 01.03.2024 — veille 29.02 —, six et dix-huit mois). Ce cours est celui de la réévaluation de l'exercice précédent quand son genre de clôture est
     `month` (35-5) ; le genre d'ouverture d'un exercice qui suit un exercice Kesh reprend par défaut le genre de
     clôture de celui-ci (35-1 AC-3 bis), si bien que la concordance tient sans réglage. Hors des bornes du `POST /fetch` de 35-2 (date < 2000-01-01, AC-5 : il rendrait 400), le
     `lookup` — qui n'a pas de borne basse et rend alors `404 RATE_UNAVAILABLE` faute de cours — n'est suivi d'**aucun**
     `POST /fetch` : saisie manuelle. Pour un complément daté `TODAY` (`complementDateKind`), cours mensuel du mois de cette date, comme
     avant. Modifiable. Le `404 RATE_UNAVAILABLE` d'un jour (`details.date`) déclenche `POST /fetch { date }` si la
     veille est strictement passée (35-2 AC-5), sinon saisie manuelle. Tests : exercice civil (1er janvier), au
     1er juillet, au 1er avril, en genre `day` et `month` — genre `month` : `lookup` de décembre de l'année
     précédente, de juin, de mars (jamais du mois d'ouverture) ; ouverture au 15.03 : mars (veille 14.03) ; la durée de
     l'exercice est sans effet (six et dix-huit mois commençant le 01.07 : même date, juin, T2). **Envoyé `rate_source = manual`** sur
     l'ouverture et le complément, proposé ou saisi : le contrôle `FX_RATE_NOT_AFC` de 35-3a (AC-4, 4) compare une
     ligne `afc` au cours **mensuel** du mois de `entry_date`, à la création **et** au `PUT` pour une ligne nouvelle ou
     modifiée (arbitrage A17) — un cours journalier, ou d'un mois que la date recalculée du complément a quitté, y
     serait refusé dès la création, et toute ligne retouchée de l'écriture d'ouverture le serait aussi. L'écran affiche la provenance (« proposé : cours journalier du JJ.MM.AAAA » ou « proposé : AFC
     <mois> », ou « cours saisi » quand `source = "manual"` — U1 : la provenance se lit sur `source`, sans
     comparaison de valeurs, comme au formulaire, AC-2) ; le Change Log de la story le dit. Le genre `day`
     (défaut) propose le cours de clôture de l'exercice antérieur, celui de sa réévaluation (35-5) : un solde repris d'un
     bilan réévalué tombe donc juste ; sinon il se saisit au CHF (« cours OU CHF » ci-dessous), ce qui reprend exactement
     le CHF de clôture. Sur `404 RATE_UNAVAILABLE` du `lookup`, même repli qu'à l'AC-2 (un `POST /exchange-rates/fetch`,
     puis relecture ; sinon saisie manuelle).
   - **Exercice mono-devise (R3)** : si le premier exercice (ouverture) ou l'exercice de la date du complément est
     mono-devise (`multiCurrency` de `GET /fiscal-years`), les comptes en devise ne sont **pas** offerts dans la grille
     (aucun champ en devise, aucune ligne pour eux) et un texte dit pourquoi (« exercice mono-devise ») ; le serveur les
     refuserait (35-3a AC-4, `MULTI_CURRENCY_DISABLED`). L'écran ne lit pas tous les exercices : un exercice
     **postérieur** mono-devise fait refuser une ligne en devise de l'ouverture ou du complément par le même code
     (`reason: "laterFiscalYear"`, `details.fiscalYearId`, 409) — les deux grilles affichent ce refus traduit (le message
     dit de passer l'exercice postérieur en multi-devises), classé comme tout refus FX de l'écran, saisie conservée. L'écriture d'ouverture et son complément peuvent porter
     plusieurs devises (exemption de `FX_MULTIPLE_CURRENCIES`, 35-3a AC-4, 2 bis). ⚠️ La page
     (`settings/opening-balances/+page.svelte`) ne charge aujourd'hui **aucun** exercice : elle charge `GET /fiscal-years`
     au montage (`multiCurrency` et `openingRateKind` ci-dessus). Si ce chargement échoue : les comptes en devise restent
     offerts (le serveur tranche, refus affiché comme ci-dessus) et **aucun** cours n'est proposé (genre inconnu) —
     saisie manuelle, avec un message ; Vitest de ce cas.
   - **Saisie manuelle : cours OU CHF** (ouverture, complément et formulaire d'écriture, AC-2). L'utilisateur saisit
     soit le cours, soit le montant CHF. Si le CHF est saisi : cours = CHF ÷ montant en devise, arrondi à 10 décimales ;
     le client vérifie que `round(montant × cours, 2)` redonne le CHF saisi et, sinon, ajuste le cours au dernier chiffre
     jusqu'à ce que ce soit le cas (borné : si aucun cours à 10 décimales ne convient, message, saisie refusée). Sert à
     reprendre exactement le CHF du bilan de clôture précédent ou le CHF réel d'une conversion ; `rate_source = manual`.
   - **Le serveur ne fait aucun `lookup`** (la résolution du cours est faite à l'écran, avant l'appel) et ne lit
     `exchange_rates` (35-2, U1) **avant aucun verrou** : `create_opening_complement_in_tx` exige que la transaction n'ait
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
     `FX_AMOUNT_MISMATCH`, devise annoncée ≠ devise du compte (`FX_CURRENCY_MISMATCH`), ligne sans `currency` → devise
     du compte (T3), `foreign` sur un compte CHF → `FX_ACCOUNT_IN_CHF`, mono-devise → `MULTI_CURRENCY_DISABLED`,
     `revaluation` refusé ; Vitest (CR-4, T3) : le corps envoyé par chacune des deux grilles porte
     `foreign.currency` = la devise du compte de la ligne ; ouverture **et** complément à lignes EUR et USD acceptés **par leurs routes** (exemption de
     `FX_MULTIPLE_CURRENCIES`, 35-3a AC-4, 2 bis), puis `PUT` du libellé de l'écriture d'ouverture ainsi créée → 200 ;
     aller-retour `.keshbackup` ; **non-régression du mode Compléter bancaire** (34-2, compte CHF inchangé) ; Vitest de
     l'écran et des totaux (`opening-balances-page.test.ts`, `opening-balances-totals.test.ts`), dont : premier
     exercice (ou exercice du complément) mono-devise → aucun compte en devise offert, motif affiché ; refus
     `laterFiscalYear` affiché sur chacune des deux grilles.
2. **AC-2 — Formulaire d'écriture** (`JournalEntryForm.svelte`), en multi-devises seulement :
   - **Sens.** Une ligne en devise se saisit dans la colonne **Débit ou Crédit en montant en devise** (le sens est
     celui de la colonne) ; le CHF est calculé (`round(montant × cours, 2)`, `Big.roundHalfUp`), affiché en lecture
     seule sous le champ, et alimente `computeBalance` et la ligne d'écart. `debit`/`credit` du payload restent les
     montants en devise principale (aujourd'hui CHF ; U6), `foreign.amount` le montant en devise. `LineDraft` (`form-helpers.ts` :13) gagne
     `foreign?: { currency, side: 'debit' | 'credit', amount, rate, rateSource }` (absent = ligne CHF ; T3, CR-4 :
     `currency` du brouillon est **la devise du compte** de la ligne, `account.currency` lue sur `AccountResponse` au
     choix du compte — jamais saisie —, et `lineResponseToDraft` la reprend de `foreign.currency` de la réponse, elle
     aussi lue sur le compte par le serveur, 35-3a AC-6) — **contrat de
     stockage** : le montant saisi en devise vit dans `foreign.amount` et le sens dans `foreign.side` (il doit survivre
     à un cours ou un CHF encore vides) ; `debit`/`credit` de la ligne ne portent **que le CHF dérivé** (l'un des deux
     vide, selon `side`), recalculé à chaque changement et seul lu par `computeBalance`, `classifyLine` et
     `isDraftLineNonEmpty` — aucun d'eux ne lit jamais `foreign.amount` comme du CHF ; une ligne en devise sans CHF
     calculable (cours ou montant vide ou invalide) est `partial` (le message de la ligne est celui du code rendu par `computeChf` — `FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO` — traduit, non un « montant invalide » générique) ; `classifyLine` (`balance.ts` :143-162),
     `computeBalance` (`balance.ts` :83), `isDraftLineNonEmpty` (`vat-purchase.ts` :106) et les fabriques de lignes
     (`JournalEntryForm` :105-106, :187 ; `vat-purchase.ts` :87-96) sont revus, inventaire au Change Log.
     **Payload (T3)** : `foreign: { currency, amount, rate, rateSource }` — le type TS de la requête porte
     `currency?: string` (facultatif, comme l'API, 35-3a AC-6), mais le formulaire, les deux grilles d'ouverture,
     l'assistant TVA et l'assistant *Conversion* l'envoient **toujours** (la devise affichée) : c'est la garde contre
     un compte dont la devise a changé depuis le chargement (`FX_CURRENCY_MISMATCH`, ci-dessous) ; le type de la
     **réponse** porte `currency: string` (toujours présent dans `foreign`). Vitest : le corps envoyé porte la devise
     du compte, y compris pour les lignes que produisent l'assistant TVA et l'assistant *Conversion* (leur Vitest).
     **Changer le compte d'une ligne (U2, CR-5)** — la devise de la ligne **suit le compte** (elle n'est verrouillée
     qu'au compte, 35-1 AC-4 ; le serveur juge la ligne sur le nouveau compte, 35-3a AC-7) : CHF → devise, champs en
     devise initialisés (montant vide, cours proposé par `lookup` de la nouvelle devise) ; devise → CHF, `foreign`
     **retiré** (la ligne redevient une ligne CHF, montant CHF vide) ; devise → **autre** devise (USD → EUR) :
     **montant et cours réinitialisés** (montant en devise vide, cours reproposé par `lookup` de la nouvelle devise,
     `rateProposed` vrai, CHF recalculé à vide) — jamais un montant USD gardé sous l'étiquette EUR ; devise → même devise
     (deux comptes USD) : montant et cours gardés, cours reproposé seulement si `rateProposed`. Le changement de compte
     compte comme une **retouche** (indicateur de l'édition, ci-dessous : la provenance relue n'est plus conservée).
     Vitest de chacun des quatre cas, dont : USD → EUR en édition d'une ligne `afc` relue → montant vide, cours EUR
     proposé, corps envoyé en EUR ; USD → CHF → aucune clé `foreign` dans le corps.
   - **Champs** : *Montant (USD)*, *Cours* (prérempli par le **cours effectif** que rend `lookup` pour le mois de la
     date, `effectiveRate` — le cours retenu par la société, U1 ; indicateur « AFC » (`source: "afc"`), « cours saisi »
     (`source: "manual"`, cours de la société, D9) ou « manuel » (retouché à la main) ; modifié à la main → `manual`),
     *CHF* ; **provenance envoyée** (D9, U1) : on note **`afcRate`** l'`effectiveRate` d'un `lookup` dont `source`
     vaut `"afc"` (`null` si `source` vaut `"manual"`, ou sans cours) — c'est exactement le cours auquel le serveur
     compare une ligne `afc` (35-3a AC-4, 4) ; `rateSource = 'afc'` si le cours de la ligne **égale** `afcRate` (non
     `null`) **du mois de la date** (`afcMonth`, puce suivante), comparaison en valeur (`Big.eq`, non en graphie),
     `manual` sinon — un cours saisi par la société, ou un mois sans cours `afc` retenu, part donc `manual` ; la
     dernière valeur publiée (`publishedRate` de `lookup`) n'entre jamais dans ce choix ; aucune autre valeur. Comme à
     l'AC-1, **soit le cours soit le CHF** se saisit à la
     main (CHF saisi → cours = CHF ÷ montant à 10 décimales, vérification `round(montant × cours, 2) = CHF`, ajustement
     au dernier chiffre) ; changer la date repropose le cours effectif des lignes dont le cours n'a pas été modifié à
     la main. `LineDraft.foreign` gagne pour cela un indicateur `rateProposed` (cours venu de `lookup`, non retouché),
     distinct de `rateSource` : une ligne au cours saisi de la société est `manual` **et** proposée ; en édition, une
     ligne relue `afc` est tenue pour proposée, une ligne relue `manual` pour non proposée (`rateProposed` faux : aucune
     reproposition au changement de date) — notion distincte de l'indicateur de **retouche** ci-dessous, qui gouverne
     la provenance envoyée, la provenance relue étant, elle, conservée dans `loadedRateSource`.
   - **Mois du cours AFC comparé** (35-3a AC-4, 4 : toujours le mois de `entry_date`). `LineDraft.foreign` retient
     `afcRate` **avec son mois** (`afcMonth`, le `rateDate` du `lookup`). Au changement de date, un `lookup` (mémoïsé) est
     lancé pour **toutes** les lignes en devise, proposées ou non : il rafraîchit `afcRate`/`afcMonth` ; le cours n'est
     remplacé que pour les lignes proposées. À l'envoi, `afc` seulement si `afcMonth` = mois de la date **et** cours =
     `afcRate` ; sinon `manual` (jamais `afc` sur un `afcRate` d'un autre mois, que le serveur refuserait en
     `FX_RATE_NOT_AFC`) ; l'envoi attend les `lookup` en cours, et un `lookup` en échec ou sans cours AFC laisse
     `afcRate = null` (`manual`). **Édition** : aucun `lookup` au montage, donc `afcRate` est inconnu ; une ligne dont
     ni le **cours**, ni la **date**, ni le **compte** n'ont changé **conserve la provenance relue** (`afc` ou `manual`)
     à l'envoi — le **montant** n'en fait pas partie : le contrôle `FX_RATE_NOT_AFC` ne lit que cours et mois, et un
     montant corrigé ne doit pas rétrograder en `manual` une ligne au cours AFC. Un cours retouché en édition lance le
     `lookup` (mémoïsé) de (devise, mois de la date), que l'envoi attend, avant d'appliquer la règle d'égalité — sans
     quoi `afcRate`, inconnu, ferait partir `manual` un cours retapé égal à l'AFC. État porté par `LineDraft.foreign` :
     `loadedRateSource` (provenance relue, posée par `lineResponseToDraft`, absente pour une ligne neuve) et un
     indicateur de retouche (cours, date ou compte), remis à faux au chargement.
   - **Cours de la société sous le champ** (D9, U1) : quand le `lookup` **du mois de la date courante** (`afcMonth`
     égal, aucun `lookup` de ce mois en cours — jamais le cours d'un mois quitté) rend un `effectiveRate` qui diffère du
     cours de la ligne, le formulaire affiche « AFC : <cours> » (`source: "afc"`) ou « Cours saisi : <cours> »
     (`source: "manual"`) et un bouton **↑** (« Reprendre le cours de la société ») qui remet **la ligne** à ce cours
     (`rateSource` = `afc` si `source` vaut `"afc"`, `manual` sinon ; ligne proposée : un changement de date ultérieur
     la repropose au cours effectif du nouveau mois, comme toute ligne proposée) — action locale du formulaire ; il ne
     touche **jamais** la table des cours de la société (le ↑ de cette table, qui reprend une valeur publiée, vit à
     l'écran *Cours de change*, 35-2 AC-6). Ligne au cours de la société : indicateur « AFC » ou « cours saisi », pas
     de ↑.
   - **Appels `lookup`** : mémoïsation par `(devise, mois)` par formulaire, **résultats négatifs compris** ; appel
     seulement pour une date valide dans `dateBounds` et après débounce (`debounce.ts`) ; jeton de requête pour ignorer
     une réponse périmée ; si le nouveau mois n'a pas de cours, le cours **proposé** précédent est **effacé** (jamais
     gardé sous l'étiquette `afc` ni comme proposition) ; en édition, aucune reproposition au montage, seulement au
     changement de date et pour les lignes proposées (`rateProposed`). `GET /lookup` est **en lecture seule** (35-2) et
     ne rend 404 que si la société n'a **aucune** ligne `exchange_rates` pour ce mois (un cours saisi seul rend 200,
     `source: "manual"`, sans `POST /fetch`) : sur `404 RATE_UNAVAILABLE`, l'écran appelle **une
     fois** `POST /exchange-rates/fetch` pour ce mois — par la fonction d'API de 35-2 (seule entrée : sans bandeau
     dégradé sur 503, `timeoutMs` de 90 s, 35-2 AC-3 et AC-5), le mois tiré de `details.month` (`AAAA-MM-01`) ; les specs
     E2E pilotent la source factice et s'appuient sur le vidage des tables par `/_test/seed` (35-2 AC-7) — (rôle Comptable ou Administrateur ; mois ≥ 2000-01 et ≤ **mois suivant** le mois courant, **à toute date**, 35-2 AC-5 et R4 amendé, non « ≤ mois courant » : le mois suivant peut être publié avant le 25), puis
     relit ; `404 RATE_NOT_PUBLISHED`, `503` ou rôle Consultation → message, saisie manuelle exigée. Pour un mois
     **postérieur au mois suivant le mois courant** (Europe/Zurich ; 35-2 AC-5 : le mois suivant est admis à toute
     date, non publié → `404 RATE_NOT_PUBLISHED` ; R4 amendé, alignement de l'orchestrateur),
     **aucun** `POST /fetch` (il rendrait 400, 35-2 AC-5) : saisie manuelle directement ; un 400 du `POST /fetch` se
     traite comme `RATE_NOT_PUBLISHED`. Le cours du mois M est publié le 25 de M−1 ou le jour ouvrable suivant selon l'ESTV, mais parfois avant (relevé du 14.11.2026, à confirmer par Beat ; R4 amendé ; 35-2 *Source*), et la tâche
     automatique de 35-2 (une vérification par jour jusqu'à son arrivée, puis pause, S1) le récupère pour les exercices multi-devises ; le cas « cours indisponible » vise surtout un
     mois futur, une devise non publiée ou une source injoignable. *(Annotation de l'orchestrateur, 2026-11-14 : la
     remédiation P1 avait écrit l'inverse ; précisée par R4, CR-2.)* La
     relecture qui suit un `POST /fetch` réussi **remplace** l'entrée négative mémoïsée de (devise, mois) — elle ne la
     relit pas —, et ses `afcRate`/`afcMonth` rafraîchissent les lignes comme un `lookup` de changement de date.
   - **Formats et bornes** — celles de 35-3a AC-3, à l'identique : le cours se valide avec `AmountFormatOptions`
     `{ maxDecimals: 10, maxIntegerDigits: 7 }` (`balance.ts` :23-31) puis cours ≤ 10⁶, rejette 0 ; le montant en
     devise : 4 décimales, ≤ 10¹³ ; message d'erreur propre au cours (le « Maximum 4 décimales » existant ne vaut que
     pour les montants). `computeChf` rend l’erreur `FX_VALUE_OUT_OF_RANGE` sur ces bornes **et sur un montant ou un cours ≤ 0** (35-3a AC-3 étape 2 : même code) (les vecteurs `error` de la
     fixture partagée l'exigent), **et aussi pour un CHF arrondi > 10¹³** (borne
     de 35-3a AC-3, vecteur « CHF > 10¹³ » de la fixture) — une seule borne, au client comme au serveur, et non les
     quinze chiffres entiers de `DECIMAL(19,4)` ; un cours déduit d'un CHF saisi passe par les mêmes bornes. Le payload sérialise par `toFixed`, jamais `toString()`.
   - **Refus du serveur propres aux devises** (`POST` et `PUT`) : `RATE_UNAVAILABLE {currency, month}` — p. ex. le
     `PUT` d'une écriture ancienne dont une ligne `afc` est **nouvelle ou modifiée** (compte, débit ou crédit, devise, montant en devise, cours,
     provenance — une ligne `manual` repassée `afc` au même cours est modifiée — ou date de l'écriture ; ni l'ordre des
     lignes ni le projet), pour un mois où la société n'a aucun cours retenu (p. ex. exercice passé en multi-devises
     avant toute récupération ; depuis U1, une restauration ramène les cours de la société, 35-2 AC-7) ; le
     `PUT` du seul libellé, de lignes CHF ou d'un projet passe, ses lignes `afc` inchangées n'étant jamais recontrôlées (35-3a AC-4, arbitrage A17) — →
     l'écran propose « Récupérer le mois » (`POST /exchange-rates/fetch`, mêmes rôles et même
     règle que ci-dessus) puis le renvoi, ou le passage de la ligne en `manual` ; `FX_RATE_NOT_AFC` → le cours attendu
     (`details.expectedRate`, 35-3a AC-4) est affiché et proposé à la reprise ; `FX_AMOUNT_MISMATCH` porte
     `details.expected` / `details.provided` (35-3a AC-3) ; `FX_CURRENCY_MISMATCH` (T3 : la devise annoncée diffère
     de celle du compte — elle a changé depuis le chargement ; le message dit de recharger la page, `details.expected`
     nomme la devise du compte), `FX_ACCOUNT_IN_CHF` (ligne en devise sur un compte devenu CHF ; même conseil),
     `ACCOUNT_REQUIRES_CURRENCY`, `FX_AMOUNT_MISMATCH` → message traduit sur la ligne fautive (clés des deux messages neufs
     `FX_CURRENCY_MISMATCH` et `FX_ACCOUNT_IN_CHF` dans les quatre locales ; Vitest de leur affichage au formulaire, aux
     deux grilles d'ouverture et dans l'assistant *Conversion*, sur la bonne ligne ; numéro en `details` : le rang **1-based** de la ligne dans `lines`,
     lu dans `details.line` (35-3a AC-3), y compris pour `FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO` et
     `FX_RATE_NOT_AFC`). `ACCOUNT_REQUIRES_CURRENCY` portant `details.reason = "monoCurrencyFiscalYear"` (et
     `fiscalYearId`, 35-3a AC-4, 3 : ligne CHF sur un compte en devise dans un exercice mono-devise) a son **message
     propre** — passer l'exercice en multi-devises, jamais l'invite « saisissez le montant en devise » —, clé i18n dans
     les quatre locales, au formulaire, aux deux grilles d'ouverture et dans l'assistant *Conversion* ; Vitest des deux
     messages. ⚠️ Ce rang est celui du **corps envoyé**, qui omet les lignes `empty` (`nonEmptyLines`,
     `JournalEntryForm.svelte:171-174`, payload :206) : l'écran le traduit en ligne du formulaire par l'`index` de
     `nonEmptyLines[line − 1]`, jamais par `lines[line − 1]` ; même règle sur les grilles d'ouverture (`nonEmptyRows`,
     `complementLines`). Vitest de chacun, dont un refus sur une ligne précédée d'une ligne vide. `editRefusalOutcome`
     (`form-helpers.ts` :108) classe ces refus en édition : `ENTRY_IS_FX_REVALUATION` → `stale` (la fiche se recharge,
     le motif s'affiche) ; `RATE_UNAVAILABLE`, `FX_*`, `ACCOUNT_REQUIRES_CURRENCY`, `MULTI_CURRENCY_DISABLED` (réglage retiré entre le chargement et l'envoi) → `stay` (le formulaire reste
     ouvert) ; sinon ils tomberaient en `other`. Test par code. Un `FX_RATE_NOT_AFC` sur une ligne au cours saisi
     de la société ne devrait pas se produire (elle part `manual`) : s'il survient, même traitement. **Après une
     reprise** (cours AFC corrigé **repris par ↑** à l'écran *Cours de change*, pour cette société, 35-2 AC-5 et AC-6,
     U1, U3), il survient aussi au `PUT` d'une écriture ancienne, mais **pour
     ses seules lignes `afc` nouvelles ou modifiées** (35-3a AC-4, arbitrage A17 : une ligne inchangée n'est jamais
     recontrôlée, si bien que le `PUT` du seul libellé passe) — typiquement une ligne dont seul le **montant** a été
     corrigé, qui garde sa provenance relue `afc` sans `lookup` (*Mois du cours AFC comparé*) ; une ligne dont la
     date ou le cours a changé a, elle, relu le cours retenu par la société et part `manual` s'il diffère. Pour ces lignes, le
     refus propose **deux** actions, « Reprendre le cours attendu » (`details.expectedRate`, `afc`, le CHF change) et
     « Garder mon cours » (la ligne passe en `manual` au cours qu'elle porte, son CHF n'est pas recalculé) ; aucune
     action sur les autres lignes. **`expectedRate` à `null`** (35-3a AC-4, 4 : le cours de la société pour ce mois est
     désormais **`manual`** — un `PUT …/rates` l'a saisi après l'écriture — et il n'y a pas de cours `afc` à reprendre) :
     « Reprendre le cours attendu » n'est **pas offert**, seule « Garder mon cours » l'est, avec un message qui dit que
     la société a fixé son propre cours pour ce mois (CR5-P1-1) ; Vitest de ce cas. Vitest des deux actions, et Vitest d'une édition du seul libellé d'une écriture `afc`
     (corps aux lignes inchangées, provenance relue, aucun `lookup`). **U1, CR-5 (remplace S6 de CR-3)** : l'écran de
     saisie n'affiche **ni** les corrections de l'AFC **ni** l'avertissement de connexion — le cours proposé et la
     mention sous le champ sont ceux du **cours retenu par la société** que rend `lookup` (`effectiveRate`, `source`) ;
     `publishedRate` de `lookup` n'est pas affiché ici ; le rouge « AFC corrigé », le ↑ (direct tant que l'option « toutes les sociétés » est masquée, A23 ; son dialogue
     n'apparaît qu'ensuite) et
     l'acquittement vivent à l'écran *Cours de change* et au bandeau de 35-2 (AC-6).
   - **Ligne d'écart** (R8) : écriture déséquilibrée en devise principale (aujourd'hui CHF) avec au moins une ligne en devise → bouton « Ajouter la
     différence de change », qui **ajoute une ligne à l'écriture en cours** — il ne crée **aucune** écriture et
     n'envoie rien : une ligne CHF sur le compte de gains (écart au crédit) ou de pertes (au débit) du **compte en
     devise concerné** — S2, CR-3 : `fxAccountsFor(compte, société)` (35-1 AC-4 bis), soit ses comptes **propres**, soit
     ceux de la société —, que l'utilisateur voit, modifie ou retire avant d'enregistrer. **Compte concerné** (une
     écriture ne porte qu'une devise, R7, mais peut toucher plusieurs comptes de cette devise, p. ex. PayPal USD et
     ventes USD) : le premier compte en devise **de bilan** (`Asset`/`Liability`) dans l'ordre des lignes, sinon le
     premier compte en devise ; si les comptes en devise de l'écriture ne résolvent pas tous la même paire, la ligne
     est posée sur celle du compte retenu et un avis nomme ce compte (le compte de la ligne reste modifiable) —
     arbitrage A12, approuvé par Beat (CR-5) ; au-delà de n × 0.005 CHF (n = nombre de lignes en
     devise, calculé en `Big`), le texte précise qu'il ne s'agit pas d'un arrondi ; rien n'est ajouté sans clic
     (C-35-4). Libellé accessible et aide du bouton : « Ajouter une ligne de différence de change ». Vitest : un clic
     ajoute exactement une ligne et aucun appel réseau.
   - **Une devise par écriture (R7)** : dès qu'une ligne porte une devise, les comptes d'une **autre** devise sont
     retirés du sélecteur des autres lignes (les comptes CHF restent) ; une écriture relue qui en porte déjà plusieurs —
     **l'écriture d'ouverture et son complément** (exemptés, 35-3a AC-4, 2 bis), une contre-passation de l'une d'elles,
     **ou** un état hérité — **reste modifiable** : 35-3a AC-4 (2 bis) n'oppose `FX_MULTIPLE_CURRENCIES` au `PUT` que si
     le corps porte plus d'une devise **et** une devise que l'écriture stockée ne portait pas. Pour une telle écriture, le sélecteur offre les comptes CHF et ceux
     des devises **déjà portées par l'écriture relue** (jamais une devise de plus) ; un `FX_MULTIPLE_CURRENCIES` reçu
     malgré tout (corps qui ajoute une seconde devise, écriture changée entre la lecture et l'envoi) s'affiche traduit,
     `stay`. Pour changer de devise, l'assistant *Conversion* (AC-2 bis) est proposé par un lien (Vitest : avec une
     ligne USD, le sélecteur des autres lignes propose les comptes CHF et USD, aucun compte EUR ; le lien apparaît ;
     retirer la ligne en devise rend les comptes de toutes les devises ; en édition d'une ouverture à lignes EUR et USD,
     le sélecteur propose CHF, EUR et USD, aucune autre devise, et l'enregistrement d'un libellé changé part sans
     refus). Le formulaire reçoit les props `multiCurrency`,
     `fxGainAccountId`, `fxLossAccountId` (le **défaut** de la société ; les comptes propres d'un compte en devise se
     lisent sur son `AccountResponse`, `fxGainAccountId?`/`fxLossAccountId?`, 35-1 AC-4 bis, et `fxAccountsFor` compose
     les deux), fournies par `journal-entries/+page.svelte` (:163) et
     `journal-entries/[id]/+page.svelte` (:305), qui chargent déjà `fetchCompanyCurrent`. Si cet appel échoue
     (les deux pages le tolèrent), `multiCurrency` vaut vrai dès qu'un compte de `accounts` porte une devise (35-1 : un
     compte en devise n'existe qu'en multi-devises ; `AccountResponse.currency` et `currencyLocked` sont **omis** pour
     un compte CHF, 35-1 AC-4 et C-35-9 : type TS `currency?: string`, tout test de devise lit `account.currency ?? null`,
     jamais `=== null` — alignement de l'orchestrateur sur 35-1-P5-IF1) — sans quoi l'édition d'une écriture en devise perdrait `foreign`
     au `PUT` ; les comptes de différence restent alors inconnus et le bouton d'écart l'explique. **Contrôle client seulement**
     (la ligne est une ligne CHF ordinaire envoyée au `POST` ; la revalidation serveur de C-35-2 (3) vaut pour la
     réévaluation) : le bouton revalide sur la liste des comptes la paire résolue par `fxAccountsFor` (existe, actif,
     imputable, `Expense`/`Revenue`, sans devise) ; comptes non désignés ou invalides : il explique quoi régler, en
     nommant l'origine (« comptes propres du compte 1021 » ou « réglage de la société »). Vitest : compte en devise à
     comptes propres → ligne sur ses comptes ; compte « par défaut » → comptes de la société ; deux comptes USD à paires
     différentes → compte de bilan retenu et avis ; compte propre archivé → explication, aucune ligne.
   - **Arrondi** : calcul côté client avec `big.js` dans une fonction pure (`fx.ts`, `computeChf`), `.round(2,
     Big.roundHalfUp)` **explicite** (pas `Big.RM` global), même résultat que le serveur (`MidpointAwayFromZero`) ;
     Vitest lit la fixture de vecteurs **partagée** `crates/kesh-core/tests/fixtures/fx_rounding.json` fournie par 35-3a
     (dont `x.xx5`, cours à 10 décimales, 15 chiffres, hors bornes).
   - **Tests Vitest de la provenance** (D9, U1) : `lookup` `source: "manual"` → ligne proposée et `manual`, même si
     son cours égale `publishedRate` ; `source: "afc"` → `afc` ; `lookup` 404 ou `source: "manual"` (`afcRate` dérivé
     `null`) puis cours retapé → `manual` ; cours retapé égal à l'`effectiveRate` d'une ligne `afc` de la société →
     `afc` ; `publishedRate` différent de `effectiveRate` (correction non reprise) : cours retapé égal à `publishedRate`
     → `manual` ; date changée vers un
     mois dont l'AFC diffère, ligne retouchée au cours de l'ancien mois → `manual` (`afcMonth`) ; reproposition des
     seules lignes proposées ; édition non touchée → provenance relue conservée, sans `lookup` au montage ; édition,
     ligne `afc` dont seul le montant change → reste `afc`, sans `lookup` ; édition, cours retapé égal à l'AFC → `lookup`
     attendu puis `afc` ; ↑ (absent tant que le `lookup` du nouveau mois est en cours) ; relecture après `POST /fetch`
     malgré l'entrée négative mémoïsée ; efface le cours proposé d'un mois sans cours.
   - **Assistant TVA achat en devise (S5, CR-3)** — `VatPurchaseAssistant.svelte` et `vat-purchase.ts`
     (`buildPurchaseVatLines` :73, `lineVatAmount` :36 ; monté en création seulement, `JournalEntryForm.svelte:349-356` ; il construit aujourd'hui trois lignes CHF : charge HT, impôt
     préalable, contrepartie TTC, équilibrées par construction). Dans un exercice **multi-devises** (règle du
     formulaire ci-dessous), ses deux sélecteurs (charge, contrepartie) offrent aussi les comptes en devise
     (`excludeForeignCurrency = false`, AC-4) ; le compte d'impôt préalable reste celui des réglages de facturation, en
     CHF (35-1 : un compte désigné ne reçoit pas de devise). Dès que la charge ou la contrepartie est un compte en
     devise, l'assistant passe **en devise** : sa devise est celle de ce compte (charge et contrepartie en devise
     doivent porter **la même** devise — l'écriture reste à une seule devise étrangère, R7 ; l'autre sélecteur ne
     propose plus que les comptes CHF et ceux de cette devise), le champ *Montant HT* se lit dans cette devise
     (« Montant HT (USD) »), et un champ *Cours* prérempli par le `lookup` du mois de la date du formulaire (même
     fonction mémoïsée que les lignes, même règle `afc`/`manual`, mêmes bornes ; « cours OU CHF » non offert ici :
     le cours seul) — props neuves du composant : la date de l'écriture, le mode de son exercice, le réglage de la
     société et la fonction de `lookup` du formulaire (aucun second cache). Les lignes produites sont des `LineDraft`
     complets (`foreign` avec `side`, `rateProposed`, etc., contrat de l'AC-2), construits par la même fabrique que
     le formulaire. Calcul (fonction pure de `vat-purchase.ts`, `big.js`, arrondi `Big.roundHalfUp` explicite, la
     règle de 35-3a AC-3 par `computeChf`) : `HT` = round(montant HT, 2) et `TVA` = `lineVatAmount(HT, taux)` **en
     devise** (même fonction qu'en CHF, DRY) ; `TTC` = `HT` + `TVA` en devise ; chaque ligne sur un compte en devise
     porte `foreign` (montant en devise, cours, provenance) et son CHF = `computeChf(montant, cours)` ; une ligne sur
     un compte CHF reçoit le CHF de son montant en devise au **même cours** — la ligne d'**impôt préalable** (toujours
     CHF) vaut `computeChf(TVA, cours)`, la charge CHF `computeChf(HT, cours)`, la contrepartie CHF `computeChf(TTC,
     cours)`. Les trois arrondis au centime peuvent laisser un écart d'**un centime** entre débit et crédit : l'assistant
     n'ajoute aucune ligne ; le formulaire affiche le déséquilibre et propose la ligne d'écart par son bouton (R8,
     ci-dessus ; l'écart ne dépasse pas 0.01 CHF — trois erreurs d'arrondi d'au plus un demi-centime chacune (0.005
     tout juste au cas `x.xx5` ci-dessous), soit au plus 0.015, et l'écart est un multiple de 0.01 —, ce que le texte du bouton dit être un arrondi : le seuil n × 0.005 compte **toutes** les lignes dont le CHF est dérivé d'un montant en devise — les lignes
     en devise, et les lignes CHF que l'assistant a calculées au cours, marquées par un indicateur de `LineDraft`
     (p. ex. `chfFromForeign`, nom au développement) que toute retouche du montant efface ; Vitest du seuil avec et
     sans retouche). **Une ligne CHF dérivée qui s'arrondit à 0.00** (montants minuscules : TVA ou HT en devise à faible
     valeur) : l'insertion est désactivée avec le message de `FX_LINE_ROUNDS_TO_ZERO` (jamais une ligne à zéro, que
     `chk_jel_debit_credit_exclusive` refuserait) ; Vitest. **Changement de date après l'insertion** : les lignes en devise
     proposées prennent le cours du nouveau mois, mais les lignes CHF dérivées (`chfFromForeign`) gardent leur CHF — l'indicateur
     retient le cours de l'insertion ; quand le cours de la ligne en devise insérée en diffère, le
     formulaire affiche le déséquilibre, un avis « cours changé : réinsérez l'assistant » et le bouton d'écart,
     dont le texte dit que l'écart n'est pas un arrondi (il dépasse n × 0.005) ; Vitest. Le décompte TVA (en CHF, `vat_report.rs`) n'est pas modifié : il lit le débit CHF de la
     ligne d'impôt préalable. Libellé suggéré inchangé. **Tests** (Vitest de `vat-purchase.ts` et du composant) :
     achat **USD 100.00, TVA 8,1 %, cours 0.8846**, charge CHF et contrepartie PayPal USD → TVA USD 8.10, charge CHF
     88.46, TVA CHF 7.17 (7.16526), contrepartie USD 108.10 / CHF 95.63, équilibré ; **cas `x.xx5`** : USD 12.29,
     8,1 %, cours 0.8850 → TVA USD 1.00, TVA CHF 0.885 → **0.89** (arrondi loin de zéro), charge CHF 10.88
     (10.87665), contrepartie USD 13.29 / CHF 11.76 (11.76165) : débit 11.77, crédit 11.76, écart d'un centime
     proposé par le bouton d'écart sur les comptes de différence de la contrepartie (`fxAccountsFor`, S2) ; charge
     et contrepartie dans deux devises différentes → refusé à l'écran (insertion désactivée, motif) ; exercice
     mono-devise → assistant inchangé (CHF seul) ; taux exempt (TVA 0) → deux lignes, aucune ligne d'impôt
     préalable. Vecteurs relevés au calcul exact le 2026-11-14 (arrondi au centime, loin de zéro).
   - **Mono-devise** : formulaire strictement inchangé (test de non-régression) — dans une société mono-devise
     **et**, depuis R3, dans un **exercice** mono-devise d'une société multi-devises : les champs en devise et les
     comptes en devise sont masqués quand la date de l'écriture tombe dans un exercice dont `multiCurrency` est faux
     (exercices lus par `GET /fiscal-years`). ⚠️ Le formulaire ne les a pas aujourd'hui : `dateBounds` ne lit que la prop
     `entryFiscalYear` (`{ startDate, endDate }`, édition seulement), `journal-entries/+page.svelte` (création) ne
     charge aucun exercice ; `[id]/+page.svelte` les charge **déjà au montage** (`loadFiscalYears`, :100-115, appelée
     par `loadEntry` :153, état `fiscalYears` :68, pour la contre-passation de la 29-1b) et une seconde fois à
     l'ouverture de l'édition (`startEdit`, :307). Une prop neuve (`fiscalYears`, avec `multiCurrency`) est fournie par
     les deux pages — la fiche réemploie l'état `fiscalYears` du montage (un état `'error'` ou `null` vaut « exercices
     inconnus », ci-après), sans troisième lecture ; la liste de création se charge au montage,
     avec `fetchCompanyCurrent` ; si ce chargement échoue, le formulaire se comporte comme sans exercices connus (champs
     en devise offerts si `multiCurrency` de la société, le serveur tranche). Au changement de
     date vers un exercice mono-devise alors que des lignes en devise existent, elles restent affichées, marquées
     « exercice mono-devise », et l'enregistrement est désactivé avec ce motif (le serveur rendrait 409
     `MULTI_CURRENCY_DISABLED`, `details.fiscalYearId`, classé `stay` par `editRefusalOutcome`) ; de même pour
     `reason: "laterFiscalYear"` (35-3a), dont le message dit de passer l'exercice postérieur en multi-devises. Vitest
     des deux cas.
2 bis. **AC-2 bis — Assistant *Conversion* (R7).** Une conversion entre deux devises (EUR → USD) ne tient pas dans une
   écriture (35-3a, `FX_MULTIPLE_CURRENCIES`) : elle passe par le **compte de conversion** en CHF de la société (35-1,
   `fxTransitAccountId`), en **deux écritures**, chacune rapprochable d'un seul relevé :
   - **écriture 1** (date du débit sur le compte EUR) : crédit du compte EUR (montant EUR × cours EUR = CHF₁), débit du
     compte de conversion pour le solde, lignes de frais facultatives ;
   - **écriture 2** (date du crédit sur le compte USD) : débit du compte USD (montant USD × cours USD = CHF₂), crédit
     du compte de conversion du **même montant CHF** qu'à l'écriture 1 (le compte de conversion revient à **0**),
     lignes de frais facultatives, et la **différence de change** qui équilibre l'écriture 2 — au débit du compte de
     pertes si le compte de conversion rend plus que CHF₂ et les frais, au crédit du compte de gains sinon : les
     comptes de différence **du compte en devise de l'écriture 2** (le compte USD, débité au grand livre à cette
     écriture ; « crédité » ci-dessus parle du relevé), par
     `fxAccountsFor` — ses comptes propres s'il en a, sinon ceux de la société (S2, CR-3) ; l'aperçu nomme le compte
     retenu ; aucune ligne si elle est nulle ; comptes résolus invalides (contrôle de la liste, comme le bouton
     d'écart) → « Passer les écritures » désactivé avec le motif ;
   - chaque ligne de frais est en CHF ou dans **la** devise de son écriture (EUR à l'écriture 1, USD à l'écriture 2),
     sur un compte choisi (les comptes en devise d'une autre devise ne sont pas offerts) ; les écritures peuvent donc
     avoir plusieurs lignes.
   **Écran** : depuis *Écritures*, bouton « Conversion de devises » (société multi-devises, rôles qui saisissent des
   écritures) ; page ou dialogue en deux colonnes, cours proposés et « cours OU CHF » comme au formulaire (AC-2 :
   `lookup` du mois de **chaque** date, provenance `afc`/`manual`, bornes, `computeChf` et la fixture partagée), aperçu
   des deux écritures avec la ligne de différence calculée, puis **« Passer les écritures »**. Chaque date doit tomber
   dans un exercice **multi-devises** ouvert (R3), sinon le motif s'affiche et le bouton est désactivé. Le compte
   de la colonne *Débit* (côté EUR, débité au relevé, **crédité** au grand livre) et celui de la colonne *Crédit*
   (côté USD, crédité au relevé, **débité** au grand livre) sont des comptes en devise de **deux devises différentes** (sinon
   la conversion n'en est pas une : une écriture ordinaire suffit) ; la ligne de différence est calculée et **non
   modifiable** dans l'assistant (le compte de conversion doit revenir à 0), mais visible à l'aperçu avant « Passer les
   écritures » (D6 : proposée, jamais silencieuse). Un refus du serveur s'affiche sur la colonne de l'écriture que
   désigne `details.entry` (1 = Débit, 2 = Crédit) ; son `details.line` est le rang dans le corps de **cette**
   écriture tel que l'assistant l'a construit (compte en devise, frais, compte de conversion, ligne de différence) et
   se traduit en champ de l'assistant par la même table que celle qui a construit le corps ; un refus sans
   `details.entry` (`FX_ACCOUNT_INVALID`, `FX_CONVERSION_TRANSIT_UNBALANCED`) s'affiche au-dessus de l'aperçu, saisie
   conservée ; Vitest d'un refus par colonne. Maquette :
   ```
   ┌ Conversion de devises ─────────────────────────────────────────────────────────┐
   │  Débit                                  │  Crédit                              │
   │  Date        [05.03.2024]               │  Date        [06.03.2024]            │
   │  Compte      [1021 Wise EUR       ▾]    │  Compte      [1022 Wise USD     ▾]   │
   │  Montant EUR [1'000.00]                 │  Montant USD [1'078.40]              │
   │  Cours       [0.9470] AFC ✓             │  Cours       [0.8846] AFC ✓          │
   │  CHF          947.00                    │  CHF          953.95                 │
   │  + Ajouter des frais                    │  + Ajouter des frais                 │
   ├─────────────────────────────────────────┴──────────────────────────────────────┤
   │  Aperçu                                                                        │
   │  1) 05.03.2024  Conversion EUR → USD (1/2)                                     │
   │       1090 Compte de conversion        947.00                                  │
   │       1021 Wise EUR       EUR 1'000.00 × 0.9470            947.00              │
   │  2) 06.03.2024  Conversion EUR → USD (2/2)                                     │
   │       1022 Wise USD       USD 1'078.40 × 0.8846   953.95                       │
   │       1090 Compte de conversion                            947.00              │
   │       6840 Gains de change                                   6.95              │
   │  Compte de conversion après les deux écritures : 0.00                          │
   │                                                     [ Passer les écritures ]   │
   └────────────────────────────────────────────────────────────────────────────────┘
   ```
   **Serveur** (route neuve, la seule de cette story) : `POST /api/v1/journal-entries/conversions
   { entries: [CreateJournalEntryRequest, CreateJournalEntryRequest] }` — les deux écritures **dans une
   transaction** (`create_in_tx` deux fois, tout ou rien), handler sous `retry_on_deadlock` ; mêmes rôles et même
   `lib.rs` que `POST /journal-entries` ; chaque écriture passe **tous** les contrôles de `POST /journal-entries`
   (35-3a, dont `FX_MULTIPLE_CURRENCIES` et `MULTI_CURRENCY_DISABLED` par exercice) ; en plus : compte de conversion
   de la société revalidé à l'usage (de la société, actif, imputable, en CHF, de bilan ; sinon 400 `FX_ACCOUNT_INVALID`,
   `details` de 35-1, `field: "fxTransitAccountId"`) ; il figure dans les deux écritures, au débit de l'une et au
   crédit de l'autre, pour le même CHF (sinon 400 `FX_CONVERSION_TRANSIT_UNBALANCED`, code neuf, quatre locales,
   `details: { entry1, entry2 }`) ; un refus d'une écriture rend son code avec `details.entry` (1 ou 2) en plus de ses
   `details` (un refus sans `details` en reçoit : mécanisme commun, au développement, à inventorier au Change Log), et
   rien n'est créé ; `entries` de longueur ≠ 2 → 400 `Validation` ; le compte de conversion peut figurer sur plusieurs
   lignes d'une écriture : le contrôle compare le **solde net** de ses lignes (débit − crédit) dans chaque écriture.
   **Verrous et ordre des refus** (une seule raison) : `entries` de longueur ≠ 2 (avant la transaction) ; puis, dans
   la transaction et **avant** le premier `create_in_tx` : la ligne `companies` (`fx_transit_account_id`) lue
   `LOCK IN SHARE MODE` (jamais `FOR SHARE`), puis le compte de conversion lu `LOCK IN SHARE MODE` — ordre `companies` →
   réglages → `accounts` de 35-1 AC-3, comme la réévaluation (35-5 AC-2, étapes (0) et (3)) ; le `companies` en partage
   que `create_in_tx_inner` reprend en tête de son étape 0 (35-3a AC-4) est alors déjà tenu, compatible → refus
   `FX_ACCOUNT_INVALID` ; → `FX_CONVERSION_TRANSIT_UNBALANCED` (calculé sur les corps, identifiant relu sous verrou) ;
   → les deux écritures dans l'ordre d'exécution ci-après, chronologique (refus de chacune dans l'ordre de 35-3a AC-4, avec `details.entry` = son rang dans le corps). Les deux
   `create_in_tx` s'exécutent dans l'ordre **chronologique** de leurs dates (corps dans l'ordre à date égale), pour
   prendre les exercices `FOR UPDATE` dans l'ordre de la bascule et de `close` (antérieurs d'abord) ; la réponse garde
   l'ordre du corps et `details.entry` le rang dans le corps. Le compte de conversion lu en partage avant l'exercice
   (l'écriture ordinaire prend l'exercice puis les comptes en partage) ne forme pas de cycle : aucun chemin ne prend un
   exercice puis ce compte **en exclusif** (la bascule de 35-1 n'archive que des comptes en devise) — argument écrit au
   Change Log, défendu de toute façon par le rejeu (`retry_on_deadlock`). Réponse 201 `{ entries: [JournalEntryResponse, JournalEntryResponse] }` ; l'audit
   est celui des deux créations (`journal_entry.created` ×2), les libellés se renvoient (« (1/2) », « (2/2) »).
   Registre des routes : `LIB_ROUTES` +1, `Traced, Rejouee` (elle écrit au journal comptable ; enveloppe exigée par
   `every_replayed_route_calls_an_envelope`) — **+1 route, +1 tracée, total +1, `rejouees` +1, `tout.len()` +1**, en
   deltas cumulés avec 35-1, 35-2 et 35-5 ; l'assertion `examinees` de `every_replayed_route_calls_an_envelope`
   (`audit_route_registry.rs:1325`, 27) prend +1, chiffre et toutes lettres (alignement de l'orchestrateur sur
   35-5-CR2-P2-IF2) ; phrase du manuel d'administration recomptée.
   **Tests** : dépôt et API — conversion sans frais (gain), avec frais aux deux écritures (perte), différence nulle
   (aucune ligne d'écart), compte de conversion à 0 après, refus de la seconde écriture **exécutée** (la plus tardive, placée en tête du corps pour exercer l'ordre chronologique) → rien de créé, `details.entry` = son rang dans le corps, compte de
   conversion non désigné ou invalide, transit déséquilibré, date dans un exercice mono-devise, `details.entry`,
   `entries` de longueur 0, 1 ou 3, rôle Consultation → 403, clé d'API en lecture seule refusée ;
   Vitest de l'assistant (calcul de la différence sur la fixture partagée, frais, cours OU CHF, bouton désactivé hors
   exercice multi-devises ; compte USD à comptes propres → ligne de différence sur ces comptes, compte USD « par
   défaut » → comptes de la société, S2) ; E2E (AC-5). L'écriture 2 reste modifiable après coup comme toute écriture (le lien entre
   les deux n'est pas gardé en base : limite écrite au manuel).
3. **AC-3 — Affichage et édition** : détail et liste des écritures montrent montant en devise (avec son code), cours
   **sans zéros terminaux** et source (`AFC`, `manuel`, `réévaluation`) à côté du CHF ; `lineResponseToDraft` recharge
   `foreign`. Une écriture portant une ligne `revaluation` **n'est pas modifiable** : le champ existant
   `modificationBlockedBy` (`journal-entries.types.ts:84`, union `ModificationBlocker` :117 ; serveur
   `ModificationBlocker`/`ModificationGuard`, `kesh-db/src/errors.rs:134,210`) reçoit la **valeur** neuve
   `ENTRY_IS_FX_REVALUATION`, rendue par le serveur (35-3a AC-7) ; 35-3b l'ajoute à l'union TS et à
   `modificationBlockerLabel` (quatre locales ; `blocker-messages.test.ts`, « couvre exactement douze codes »
   `:51-54`, passe à treize), et la fiche affiche le motif. Les types TS `JournalEntryLineResponse` et
   `CreateJournalEntryLineRequest` (`journal-entries.types.ts`) gagnent `foreign` (T3 : `currency: string` dans la
   réponse, lue sur le compte ; `currency?: string` dans la requête, AC-2). Le code affiché à côté du montant est celui
   de `foreign.currency` de la réponse (la devise du compte), jamais une valeur gardée par l'écran. « Modifier » disparaît ; **« Supprimer »
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
   bancaires liés, comptes par défaut, produits, assistant TVA achat (hors exercice multi-devises : S5, AC-2) : les
   comptes en devise n'y apparaissent pas.
   - Un seul prédicat `isForeignCurrencyAccount(account)` ; `AccountAutocomplete` prend `excludeForeignCurrency`
     (défaut `true` ; `false` dans `JournalEntryForm` **et** dans l'assistant *Conversion* de l'AC-2 bis, qui choisit
     ses comptes en devise par devise) ; les `<select>` natifs passent par
     `withoutForeignCurrency` ; le filtre central `reconciliation-counterparties.ts` :29 l'applique.
   - Inventaire **clos** au Change Log, à partir de l'inventaire serveur de 35-3a et du grep des composants qui
     importent `fetchAccounts`/`AccountAutocomplete` : au moins `InvoiceForm`, `invoices/[id]`, `invoices/due-dates`,
     `credit-notes/[id]`, `SettleInvoiceDialog`, `SupplierInvoiceForm` et les pages fournisseur
     (`new`, `[id]`, `[id]/edit`), `settings/invoicing`, `products/+page.svelte` (:697), `ManualMatchModal`,
     `TransactionSplitModal`, `ReconciliationProposals`, `reconciliation/rules` (`RuleFormModal`),
     `bank-accounts/+page.svelte` (`<select>` :860, :1116) et `BankAccountJournalLinkForm` (:67) — tous trois par
     `linkableLedgerAccounts` (`features/bank-accounts/format.ts:180`), où le filtre se pose une fois —,
     `VatPurchaseAssistant` (:159, :204 — reçoit les comptes **non filtrés** du formulaire,
     `JournalEntryForm.svelte:350-355` ; **S5, CR-3** : dans un exercice multi-devises, ses deux `AccountAutocomplete`
     passent `excludeForeignCurrency = false` et le composant restreint lui-même à CHF plus **une** devise (AC-2,
     assistant TVA achat en devise) ; dans un exercice mono-devise, le défaut `true` reste — ne pas filtrer dans le
     formulaire, qui a besoin des comptes en devise ; exemption de la garde d'inventaire justifiée par ce motif).
   - **À ne pas filtrer** (libellés, consultation, ou saisie en devise) : `reports`, `OpenItemsScreen`,
     `accounts/+page.svelte`, `journal-entries/[id]`, `journal-entries/+page.svelte` (liste, et comptes passés au
     formulaire), `RulesList`, et **`settings/opening-balances/+page.svelte`** (`fetchAccounts` :8, :77 : les comptes en
     devise y figurent, exception C-35-5), et **l'assistant *Conversion*** (AC-2 bis : composant neuf, exemption
     justifiée à la garde ci-dessous) — le filtre ne se pose donc pas dans `fetchAccounts`. `onboarding/+page.svelte`
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
   par `/api/v1/_test/exchange-rates/source`), écriture USD avec ligne d'écart ajoutée par le bouton (R8) — une fois
   sur un compte « par défaut » (comptes de la société), une fois sur un compte USD à **comptes propres** (S2 : la
   ligne va sur ses comptes) —, **achat USD avec TVA 8,1 % par l'assistant TVA** (S5 : ligne d'impôt préalable en CHF
   au cours de l'achat, écart d'un centime complété par le bouton d'écart, écriture enregistrée), conversion
   EUR → USD **par l'assistant** (deux écritures, compte de conversion à 0 après, R7), écriture à deux devises refusée
   (`FX_MULTIPLE_CURRENCIES`) **par l'API** (`authedApiContext`) — le formulaire ne la laisse pas composer (R7) ; à
   l'écran, on vérifie que le sélecteur ne propose pas le compte de l'autre devise, formulaire sans champs en devise dans un exercice mono-devise (R3), saisie du CHF au
   lieu du cours, soldes d'ouverture d'un compte USD au cours journalier de la veille de l'ouverture (genre `day`, R6), puis, l'exercice passé en genre `month` par `PUT /fiscal-years/{id}/rate-kinds` (35-2 ; les deux champs `openingRateKind` et `closingRateKind` sont obligatoires : le second reste inchangé ; `applyToNextOpening` absent, sans exercice suivant) et la page rechargée, au cours mensuel du dernier mois de l'exercice précédent (T4 ; l'exercice du seed `with-company` s'ouvre le 01.01.2020, `crates/kesh-db/src/test_fixtures.rs:118-119` : veille 31.12.2019, mois décembre 2019 ; ce jour et ce mois servis par la source factice) — les deux cours se vérifient **au préremplissage**, l'ouverture ne s'enregistre qu'après le passage en `month` : enregistrée en genre `day`, elle ferait passer la page au complément, dont le cours est celui du mois de sa propre date (AC-1) (l'E2E repart d'une société vierge par `seedTestState('with-company')`,
   précédent `opening-balances.spec.ts:21, :80` : l'ouverture exige une société sans écriture, `ALREADY_HAS_ENTRIES`),
   mono-devise inchangé ; **cours saisi par la société** (D9) : un cours posé par `PUT /exchange-rates/rates` (35-2,
   `source: "manual"` ; si le mois a déjà été récupéré, une ligne `afc` copiée existe et le `PUT` doit porter sa `version`,
   lue par `GET /exchange-rates`, sinon 409) pour le mois est proposé, affiché « cours saisi », et la ligne part `rateSource: "manual"`
   (relue telle) ; après ↑ à l'écran *Cours de change* (`POST /exchange-rates/rates/reset`, avec la `version` de la ligne et le `publishedRate` affiché ; le mois doit avoir une
   valeur publiée, servie par la source factice et récupérée, sans quoi une ligne `manual` serait **supprimée** et non
   reprise ; la ligne de la société repasse `afc`) ; **cours `manual` apparu après l'écriture** : édition du montant
   seul d'une ligne `afc` relue, le mois étant passé en `manual` par `PUT …/rates` → `FX_RATE_NOT_AFC`, `expectedRate`
   `null`, seule « Garder mon cours » offerte, le cours proposé part `afc` ; **changement de compte (U2)** : une ligne USD passée sur un compte EUR
   repart montant vide et cours EUR proposé, l'écriture s'enregistre en EUR ;
   préalables posés par l'API (Administrateur) : mode multi-devises activé **avec les comptes de gain, de perte et de
      conversion désignés** (35-1 AC-3, sans quoi le bouton d'écart n'ajoute rien et l'assistant est désactivé),
      exercices passés en multi-devises par l'activation ; **S2/S5** : un compte USD à comptes propres (gains et pertes
      désignés par l'API à sa création) en plus d'un compte USD « par défaut », et, pour l'assistant TVA, le compte
      d'impôt préalable des réglages de facturation et un taux de TVA 8,1 % (sans quoi l'assistant est désactivé :
      `configMissing` sans compte d'impôt préalable, `noRates` sans taux, `VatPurchaseAssistant.svelte:55-56`) ; ⚠️ **le scénario R3 (exercice mono-devise) tourne sur son propre
      `seedTestState('with-company')`**, dont l'exercice unique est passé en mono-devise par la bascule (35-1) **avant**
      tout compte ou ligne en devise — le compte USD de ce scénario se crée **après** la bascule (la société restant
      multi-devises, la création l'admet, 35-1), sans quoi le masquage du formulaire n'aurait rien à masquer et la
      bascule s'arrêterait sur `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE` : un exercice mono-devise **postérieur** refuserait toute ligne en devise des
      exercices antérieurs (`laterFiscalYear`, 35-3a AC-4) — donc les scénarios USD, l'ouverture comprise — et la
      bascule vers le mono-devise est refusée tant qu'un compte en devise porte un solde ou une ligne
      (`FISCAL_YEAR_HAS_FOREIGN_BALANCES`, 35-1) ; aucun scénario ne mêle les deux états,
      compte USD (et EUR) créé, cours des mois et jours utiles servis par la source factice — dont la **veille** de
      l'ouverture ; un scénario au moins part
      d'une table des cours vide et exerce le repli de l'écran (`404 RATE_UNAVAILABLE` → `POST /exchange-rates/fetch`,
      qui copie le cours publié dans les cours de la société, 35-2 AC-3 bis → relecture), les autres récupèrent leurs
      mois par `POST /exchange-rates/fetch` en préalable (`lookup` ne lit que les cours de la société) ;
      captures d'écran des états (thème clair) ; liste des échecs attendus de `docs/testing.md` recontrôlée.
6. **AC-6 — Manuel utilisateur** : saisir une écriture en devise, cours proposé (cours mensuel AFC — **fixé d'avance** pour le mois et publié par l'AFC, Terminologie ; jamais « cours moyen »
   ni « moyenne » ; la méthode de calcul n'est décrite qu'après confirmation de Beat, R4 amendé — ou cours saisi par la société, et la
   provenance enregistrée qui en découle ; « AFC : <cours> » ou « Cours saisi : <cours> » sous le champ et le bouton
   ↑, qui remet la ligne au cours de la société sans toucher à la table des cours) et manuel (cours ou CHF), changer
   le compte d'une ligne (la devise suit le compte : montant et cours repartent à vide, U2), une seule devise par
   écriture (R7) et la ligne de différence de
   change ajoutée par le bouton (R8) — sur les comptes propres du compte en devise s'il en a (S2) —, l'achat en devise
   avec TVA par l'assistant (S5 : TVA calculée en devise, ligne d'impôt préalable en CHF au même cours, centime
   d'arrondi éventuel), le cours affiché toujours celui **retenu par la société** (une correction de l'AFC se règle à
   l'écran *Cours de change*, U1, U3 ; après la reprise par ↑, la correction d'une ligne ancienne au cours remplacé propose
   « Reprendre le cours attendu » ou « Garder mon cours », sans toucher au reste de l'écriture, A17), conversion EUR → USD pas à pas par l'assistant et le compte de conversion (R7),
   exercice mono-devise (R3), soldes d'ouverture en devise (« solde repris au cours de clôture » : cours journalier de
   la veille de l'ouverture, ou cours mensuel du dernier mois de l'exercice précédent, selon le choix de l'exercice,
   R6 et T4 : le cours d'ouverture est le cours de clôture précédent) ; i18n quatre
   locales ; PDF contrôlé. ⚠️ **Garde G4-bis**
   (`crates/kesh-api/tests/textes_coherents.rs`) : tout numéro de compte cité dans le manuel, le guide de démarrage ou
   un catalogue i18n doit exister dans l'un des trois plans livrés, sous son nom — or ni 1090, ni 1021, ni 1022, ni
   6840 n'y figurent (la maquette de l'AC-2 bis est une illustration de la fiche, pas un texte à recopier) : les exemples du
   manuel et des catalogues n'emploient que des numéros livrés (p. ex. 1020 *Banque*, 6940 *Différences d'arrondi*) ou
   parlent de « votre compte de conversion » sans numéro ; contrôle : `cargo test -p kesh-api --test textes_coherents`.
   **Guide d'API** (`docs/api-external.md`) : la ligne « Écritures comptables » de son tableau de routes (l'assistant,
   `POST /journal-entries/conversions`, ouvert aux clés d'API comme `POST /journal-entries`) et le paragraphe de
   `LATER_FISCAL_YEAR_CLOSED` (toute création d'écriture, la conversion comprise) ; les champs `foreign` sont ceux de
   35-3a — le guide dit que la devise d'une ligne est **celle de son compte** (T3), que `foreign.currency` est
   facultatif à l'envoi et, s'il est donné, doit égaler la devise du compte (`FX_CURRENCY_MISMATCH`), et qu'un client
   qui l'omet renonce à cette garde ; une ligne en devise sur un compte CHF rend `FX_ACCOUNT_IN_CHF`.

## Tâches

- [ ] Soldes d'ouverture : serveur (DTO, `ComplementLine`, statut), écran, totaux, tests (AC-1).
- [ ] `fx.ts` (calcul CHF, cours depuis CHF), formulaire, `balance.ts`, `form-helpers.ts`, détail, liste, Vitest sur la
      fixture partagée (AC-2, AC-3).
- [ ] Sélecteurs, prédicat commun, garde d'inventaire (AC-4).
- [ ] E2E, captures, manuel, i18n, CHANGELOG (AC-5, AC-6).
- [ ] CR-3 : ligne d'écart et assistant *Conversion* sur les comptes de différence du compte en devise (S2,
      `fxAccountsFor`) ; assistant TVA achat en devise (S5 : `vat-purchase.ts`, `chfFromForeign`, ligne CHF à zéro,
      changement de date) ; `FX_RATE_NOT_AFC` après la reprise par ↑ d'un cours corrigé (ex-« adoption », U1), pour
      les seules lignes nouvelles ou modifiées (A17, « Garder mon cours ») ; E2E et manuel complétés.
- [ ] CR-5 : U1 — `lookup` à `effectiveRate` + `source` (+ `publishedRate` non affiché) : type TS, `afcRate` dérivé
      (`source === 'afc' ? effectiveRate : null`), indicateurs « AFC » / « cours saisi » sur `source`, ↑ sous le champ
      vers le cours de la société, soldes d'ouverture (provenance affichée) ; U2 — changement de compte d'une ligne
      (USD → EUR : montant et cours réinitialisés ; vers CHF : `foreign` retiré ; CHF → devise ; même devise) au
      formulaire, Vitest et E2E ; manuel (AC-6).
- [ ] CR5-P1 : `FX_RATE_NOT_AFC` à `expectedRate` `null` (action « Reprendre » absente, Vitest) ; manuel sans « moyenne » ;
      préalables E2E du cours saisi et du ↑ (`version`, valeur publiée) ; genre `month` par `PUT /rate-kinds` complet.
- [ ] CR-4 : T3 — `currency` facultatif des DTO d'ouverture, types TS (réponse `currency: string`, requête
      `currency?`), devise du compte dans `LineDraft`, envoi systématique par formulaire, grilles et assistants,
      messages `FX_CURRENCY_MISMATCH` / `FX_ACCOUNT_IN_CHF`, guide d'API ; T4 — cours d'ouverture de genre `month` au
      mois de la veille de l'ouverture (fonction pure TS, vecteurs dont le 15.03), tests Vitest et E2E, manuel.
- [ ] CR-2 : cours d'ouverture selon le genre de l'exercice (AC-1, R6) ; exercice mono-devise à l'ouverture et au
      formulaire (AC-1, AC-2, R3) ; bouton d'écart qui ajoute une ligne (R8) ; une devise par écriture au formulaire
      (R7) ; assistant *Conversion* — route `POST /journal-entries/conversions`, code `FX_CONVERSION_TRANSIT_UNBALANCED`,
      registre des routes (+1), écran et maquette (AC-2 bis).

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
- 2026-11-14 — révision CR-1 (D9, D10, terminologie ; Opus) : le cours proposé (formulaire et soldes d'ouverture) est
  le **cours effectif** de 35-2 (`effectiveRate` de `lookup`, cours saisi par la société sinon cours mensuel AFC) ;
  provenance envoyée `afc` seulement si le cours égale `afcRate`, sinon `manual` ; indicateur `rateProposed` distinct
  de `rateSource` pour la reproposition au changement de date ; `lookup` ne rend 404 que sans aucun cours ; le cours
  journalier (D10) n'est pas proposé à l'ouverture (saisie au CHF) ; E2E et manuel d'un cours saisi ; terminologie.
  AC touchés : Story, AC-1 (*Cours proposé*), AC-2 (*Champs*, *Appels `lookup`*, refus du serveur), AC-5, AC-6.
- 2026-11-14 — validation CR-1 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW ; remédiés dans la fiche sauf aucun
  inter-fiches (CR1-P1-1 provenance d'une ligne non touchée en édition, `afcRate` inconnu au montage ; CR1-P1-2 mois
  du cours AFC comparé après changement de date, `afcMonth` ; LOW : cours AFC et ↑ sous le champ du formulaire,
  tests Vitest de la provenance).
- 2026-11-14 — validation CR-1 P2 (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW ; remédiés dans la fiche sauf aucun
  inter-fiches (CR1-P2-1 en édition, une ligne `afc` dont seul le montant change ne rétrograde plus en `manual`, un
  cours retouché lance le `lookup` attendu à l'envoi, état `loadedRateSource` et retouche dans `LineDraft.foreign`
  posé par `lineResponseToDraft` ; LOW : `afcMonth` rappelé dans la règle des *Champs*, ↑ limité au cours AFC du mois
  courant et ligne suivant les propositions après ↑, relecture après `POST /fetch` remplaçant l'entrée négative
  mémoïsée, libellé « cours saisi » de l'ouverture aligné sur le formulaire et ↑ au manuel).
- 2026-11-14 — validation CR-1 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche, aucun
  inter-fiches (CR1-P3-1 : le rappel « la provenance d'origine n'est pas stockée » contredisait `loadedRateSource` ; `rateProposed`
  et l'indicateur de retouche distingués).
- 2026-11-14 — révision CR-2 (revue de Beat R1–R10 ; Opus) : **R7** — assistant *Conversion* (AC-2 bis, neuf) : deux
  écritures par le compte de conversion de 35-1, la différence de change dans la seconde pour que le compte revienne à
  0, lignes de frais possibles, maquette ; route serveur `POST /journal-entries/conversions` (deux écritures en une
  transaction, `FX_CONVERSION_TRANSIT_UNBALANCED`, `details.entry`, registre +1 route `Rejouee`) ; au formulaire, une
  seule devise par écriture. **R8** — « Ajouter la différence de change » ajoute une ligne à l'écriture en cours, ne
  crée aucune écriture (test). **R6** — cours proposé à l'ouverture selon `openingRateKind` du premier exercice :
  cours journalier de la veille de l'ouverture (défaut) ou cours mensuel du premier mois, au lieu du mois précédent de
  CR-1 ; tests 1er juillet / 1er avril. **R3** — champs et comptes en devise masqués dans un exercice mono-devise
  (formulaire et grilles d'ouverture), refus `MULTI_CURRENCY_DISABLED` / `laterFiscalYear` affichés. **R4** — mois
  postérieur au dernier mois publié (mois suivant à partir du 25), terminologie du manuel. AC touchés : en-tête, Story,
  AC-1, AC-2, AC-2 bis (neuf), AC-5, AC-6, Tâches. Fiche à revalider (cycle CR-2).
- 2026-11-14 — validation CR-2 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 7 LOW ; remédiés dans la fiche sauf
  35-3b-CR2-P1-2 (pour sa part dans 35-3a) et 35-3b-CR2-P1-12 (registre), inter-fiches. MEDIUM : P1-1 préalables E2E
  contradictoires (exercice mono-devise postérieur → `laterFiscalYear` ; bascule refusée avec soldes en devise) ;
  P1-2 `PUT` d'une ouverture à deux devises refusé par `FX_MULTIPLE_CURRENCIES` (exemption à la seule création) ;
  P1-3 « mois ≤ mois courant » résiduel, contraire à R4 (dernier mois publié) ; P1-4 garde G4-bis : 1090, 1021, 1022 et
  6840 absents des plans livrés (manuel, catalogues) ; P1-5 assistant *Conversion* absent de l'inventaire et de
  l'exemption de la garde des sélecteurs. LOW : P1-6 E2E « deux devises » à faire par l'API ; P1-7 tests manquants (sélecteur
  R7, masquage R3 des grilles d'ouverture, route : longueur d'`entries`, 403, clé en lecture seule) ; P1-8 refus
  `laterFiscalYear` des grilles d'ouverture ; P1-9 le formulaire de création ne charge aucun exercice (prop
  `fiscalYears`) ; P1-10 contraintes de l'assistant (deux devises distinctes, différence non modifiable, `details.entry`
  sans `details`, solde net du compte de conversion) ; P1-11 `docs/api-external.md` ; P1-12 registre C-35-9 (la route
  revalide le compte de conversion à l'usage, que C-35-9 réservait à la réévaluation).
  **Inter-fiches non appliqués — texte proposé.** *35-3a AC-4, 2 bis* : « L'exemption vaut aussi pour le `PUT`
  d'une écriture que son contenu stocké fait déjà porter plusieurs devises (ouverture, complément, état hérité) :
  `update_in_tx` refuse `FX_MULTIPLE_CURRENCIES` seulement si le corps porte plus d'une devise **et** que l'écriture
  stockée en portait au plus une (l'ensemble des devises ne s'élargit jamais) ; test : `PUT` du libellé d'une ouverture
  à deux devises → 200 ; `PUT` qui ajoute une seconde devise à une écriture à une devise → 400. » *Registre, C-35-9* :
  ajouter « la route `POST /journal-entries/conversions` (35-3b) revalide le compte de conversion à l'usage
  (`FX_ACCOUNT_INVALID`, `field: "fxTransitAccountId"`), en plus de la réévaluation (35-5) ».
  Axes exercés : citations de code (routes, dépôt, frontend, registre, plans livrés), contrat de 35-3a, 35-1, 35-2,
  comptabilité de la conversion (maquette recalculée : 1000 × 0.9470 = 947.00 ; 1078.40 × 0.8846 = 953.95 ; gain 6.95),
  registre des routes, gardes de texte, R3 à R8. Non exercés : sauvegarde (aucune table ni colonne neuve ici), MCP
  (aucune sortie touchée), E2E non joués. Un GET de la source journalière : `xmldaily?d=20231231` (dimanche) rend le
  `<datum>` 29.12.2023 valable le 31.12 — la veille d'une ouverture au 1er janvier se lit bien.
- 2026-11-14 — alignements de l'orchestrateur après CR-2 P1 : (a) 35-3a-CR2-P1-IF1 — tester le complément
  d'ouverture à deux devises **par sa route** (exemption de `FX_MULTIPLE_CURRENCIES`) ; afficher
  `ACCOUNT_REQUIRES_CURRENCY` avec `details.reason = "monoCurrencyFiscalYear"` par un message propre (clé i18n dans
  les quatre locales) ; (b) 35-3b-CR2-P1-2 porté dans 35-3a AC-4 (2 bis) : le `PUT` d'une ouverture déjà à plusieurs
  devises reste admis ; (c) 35-3b-CR2-P1-12 porté au registre de l'epic (la route de conversion revalide le compte de
  conversion à l'usage).
- 2026-11-14 — validation CR-2 P2 (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW ; remédiés dans la fiche sauf
  35-3b-CR2-P2-9 (35-3a), inter-fiche. MEDIUM : P2-1 l'AC-2 (*Une devise par écriture*) disait encore le `PUT` d'une
  ouverture à deux devises refusé et l'exemption « à la seule création », contre 35-3a AC-4 (2 bis) alignée — écriture
  relue à plusieurs devises modifiable, sélecteur limité aux devises déjà portées, Vitest ; P2-2 l'alignement (a) de
  l'orchestrateur n'était écrit qu'au Change Log — message propre de `ACCOUNT_REQUIRES_CURRENCY`
  `reason: "monoCurrencyFiscalYear"` et test du complément à deux devises par sa route portés dans l'AC-2 et l'AC-1 ;
  P2-3 route de conversion sans verrous ni ordre des refus — `companies` puis compte de conversion en
  `LOCK IN SHARE MODE` avant le premier `create_in_tx`, ordre des refus, écritures créées dans l'ordre chronologique.
  LOW : P2-4 « le `lookup` rend 400 avant 2000 » faux (35-2 AC-5 : la borne est celle du `POST /fetch`, le `lookup`
  rend 404) ; P2-5 la page des soldes d'ouverture ne charge aucun exercice (chargement et repli écrits) ; P2-6 la fiche
  d'écriture charge déjà les exercices au montage (`loadFiscalYears` :100, 29-1b), réemployés ; P2-7 Vitest du
  sélecteur R7 mal formé ; P2-8 E2E R3 : compte USD créé après la bascule ; P2-10 affichage des refus de l'assistant
  (`details.entry`, `details.line`). (P2-9, inter-fiche, ci-dessous.)
  **Inter-fiche non appliqué — texte proposé.** *35-3a AC-4, 2 bis*, remplacer « ne refuse que si le corps porte plus
  d'une devise alors que l'écriture stockée en portait au plus une » par « ne refuse que si le corps porte plus d'une
  devise **et** une devise que l'écriture stockée ne portait pas (l'ensemble des devises ne s'élargit jamais ; une
  ouverture à EUR et USD ne reçoit pas de GBP par un `PUT`) ; test : `PUT` qui ajoute GBP à une ouverture EUR + USD →
  400 ». Motif : la règle actuelle laisse un `PUT` ajouter une troisième devise à un état hérité, contre R7.
  Axes exercés : diff `b36599d0..HEAD` et alignements ; citations de code (routes et dépôt des soldes d'ouverture,
  `lib.rs` `comptable_routes` :524-570, registre des routes :1448-1502, `errors.rs` :134/:210/:224,
  `modification_blocker` :1243, `blocker-messages.test.ts` :51, `JournalEntryForm` :105-206, pages `journal-entries`
  et `[id]` :68/:100-153/:305-307/:418, page des soldes d'ouverture) ; contrats 35-1 (AC-1 à AC-3 bis), 35-2 (AC-5),
  35-3a (AC-4, AC-6, AC-7), 35-5 (AC-2, verrous) ; comptabilité de la conversion (sens de la différence) ; i18n ; E2E
  (préalables R3). Deux GET de la source journalière : `xmldaily?d=20240331` (Pâques) rend `<datum>` 28.03.2024
  (`<gueltigkeit>` 29.03 seul, non lu par 35-2) et `d=20240630` rend 28.06.2024 — la veille d'une ouverture au 1er avril
  ou au 1er juillet se lit. Non exercés : sauvegarde (aucune table ni colonne neuve ici), MCP (aucune sortie), audit
  (aucune action neuve), E2E et Vitest non joués, PDF du manuel (non écrit).
- 2026-11-14 — validation CR-2 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW ; remédiés dans la fiche. MEDIUM :
  35-3b-CR2-P3-1 l'AC-2 disait encore le `POST /fetch` borné au « dernier mois publié » (« dès le 25, le mois suivant
  se récupère ») et le cours « publié le 25 de M−1 », contre la phrase ajoutée en CR-2 P2 juste après, 35-2 AC-5 et
  R4 amendé (mois suivant à toute date). LOW : P3-2 le renvoi à 35-3a AC-4 (2 bis) disait le `PUT` refusé si l'écriture
  stockée portait au plus une devise, la règle alignée étant « plus d'une devise **et** une devise de plus » ;
  P3-3 « écriture 1, puis écriture 2 » pour les refus de la conversion, contre l'exécution chronologique écrite à la
  suite. Aucun finding inter-fiche. Axes exercés : diff `02b217ac..cb15f2cd` de la fiche ; renvois à 35-2 (AC-5,
  *Source*), 35-3a (AC-4, 2 bis), 35-1 (AC-3 : ordre `companies` → exercices → comptes), 35-5 (AC-2 (0), (3)), R4 amendé ;
  `audit_route_registry.rs:1325` (27) ; argument d'absence de cycle sur le compte de conversion. Non exercés : code
  des pages (lignes citées en CR-2 P2 non relues), i18n, E2E, PDF.
- 2026-11-14 — validation CR-2 P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW ; remédiés dans la fiche. LOW :
  35-3b-CR2-P4-1 l'AC-2 disait le cours « publié au plus tard le 25 de M−1 », ce que ni R4 (« le 25 de M−1, ou le jour
  ouvrable suivant ») ni R4 amendé (calendrier « à confirmer par Beat ») ni 35-2 *Source* n'affirment : phrase réalignée,
  sans effet sur le comportement (Kesh n'en dépend pas) ; 35-3b-CR2-P4-2 le test « refus de la seconde écriture → rien
  de créé » de l'AC-2 bis restait ambigu depuis l'exécution chronologique (CR-2 P3-3) : précisé (seconde **exécutée**,
  placée en tête du corps, `details.entry` = rang dans le corps). Aucun finding inter-fiche. Axes exercés : diff
  `cb15f2cd..dab5975d` de la fiche (trois hunks) ; renvois à 35-2 AC-5 (bornes ≥ 2000-01, ≤ mois suivant à toute date,
  rôles) et *Source*, 35-2 AC-4 (vérification quotidienne), R4 et R4 amendé, 35-3a AC-4 (2 bis : règle du `PUT`,
  mot pour mot), cohérence interne de l'AC-2 bis (`details.entry` 1/2 = colonnes, ordre d'exécution, réponse dans
  l'ordre du corps). Non exercés : code des pages et du serveur (aucune ligne de code citée par les hunks), i18n, E2E,
  Vitest, PDF.
- 2026-11-14 — révision CR-3 (revue de Beat S1–S6 ; Opus) : **S2** — ligne d'écart (AC-2, R8) sur les comptes de
  différence du **compte en devise concerné** par `fxAccountsFor` (35-1 AC-4 bis) : premier compte de bilan en devise
  de l'écriture, sinon premier compte en devise, avis si les paires diffèrent (règle de l'orchestrateur, à confirmer) ;
  props du formulaire : défaut de la société, comptes propres lus sur `AccountResponse` ; assistant *Conversion*
  (AC-2 bis) : différence sur les comptes du compte en devise de l'écriture 2 ; la route de conversion ne revalide
  toujours que le compte de conversion. **S5** — assistant TVA achat en devise (AC-2, puce neuve) : sélecteurs ouverts
  aux comptes d'**une** devise en exercice multi-devises, TVA calculée en devise par `lineVatAmount`, ligne d'impôt
  préalable en CHF au même cours (`computeChf`, règle de 35-3a), écart d'arrondi d'un centime laissé au bouton d'écart
  (aucune ligne ajoutée par l'assistant), tests USD 100.00 à 0.8846 (équilibré) et USD 12.29 à 0.8850 (TVA CHF 0.885
  → 0.89, écart d'un centime), vecteurs recalculés au décimal exact ; AC-4 (inventaire des sélecteurs) aligné.
  **S6** — l'écran de saisie n'affiche ni valeur republiée ni avertissement (35-2 AC-6) ; `FX_RATE_NOT_AFC` après une
  adoption. E2E (AC-5) et manuel (AC-6) complétés. S1, S3, S4 sans point de contact. Prémisses relues au code :
  `vat-purchase.ts:36`, `:73-99` (trois lignes CHF équilibrées par construction) ; `VatPurchaseAssistant.svelte:16-25`
  (props), `:159`, `:204` ; `JournalEntryForm.svelte:349-356` (création seulement) ; `vat_report.rs:13-19` (impôt
  préalable lu en débit CHF). AC touchés : en-tête, AC-2, AC-2 bis, AC-4, AC-5, AC-6. Fiche à revalider (cycle CR-3).
- 2026-11-14 — validation CR-3 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW ; remédiés dans la fiche, aucun
  inter-fiches. MEDIUM : 35-3b-CR3-P1-1 le refus `FX_RATE_NOT_AFC` n'était décrit que pour une adoption entre `lookup` et
  l'envoi, alors que 35-3a AC-4 (l. 222) le fait survenir sur toute écriture ancienne à lignes `afc` après adoption : action
  « Garder mon cours » ajoutée ; 35-3b-CR3-P1-2 changement de date après l'insertion de l'assistant TVA (lignes CHF
  dérivées périmées, déséquilibre muet) : avis, bouton d'écart, test. LOW : P1-3 ligne CHF dérivée arrondie à 0.00 ;
  P1-4 « deux demi-centimes » faux (trois arrondis, < 0.015) ; P1-5 « compte crédité côté USD » ambigu (relevé contre
  grand livre) ; P1-6 préalables E2E de S2/S5 ; P1-7 tâches CR-3 absentes. Axes exercés : diff `294889cc..b6f88c24` ;
  vecteurs S5 recalculés (100 @ 0.8846 : 88.46 + 7.17 = 95.63 ; 12.29 @ 0.8850 : 10.88 + 0.89 = 11.77 contre 11.76) ;
  code : `vat-purchase.ts` (`lineVatAmount`, `buildPurchaseVatLines`), `VatPurchaseAssistant.svelte` props,
  `JournalEntryForm.svelte:349-356`, `balance.ts computeBalance`, `LineDraft`, `vat_report.rs` (solde `débit − crédit`),
  `audit_route_registry.rs:1325` (27) ; renvois 35-1 AC-4 bis (`fxAccountsFor`), 35-2 (`lookup` : `afcRate` en vigueur,
  `republished`), 35-3a AC-4 ; comptabilité de la conversion et de l'écart. Non exercés : sauvegarde, MCP, audit (aucune
  table ni action neuve ici), source journalière (aucun GET : sans objet pour cette fiche), E2E, Vitest, PDF.
- 2026-11-14 — validation CR-3 P2 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW ; remédiés dans la fiche, aucun
  inter-fiches. MEDIUM : 35-3b-CR3-P2-1 (arbitrage A17) l'AC-2 faisait survenir `FX_RATE_NOT_AFC`, après adoption, sur
  « toute écriture ancienne à lignes `afc` dont on corrige seulement le libellé » — remédiation de CR-3 P1-1, contraire à
  A17 : le refus ne vise plus que les lignes `afc` nouvelles ou modifiées (cas type : montant seul corrigé, provenance
  relue gardée), les deux actions pour ces lignes seulement, Vitest d'une édition du seul libellé ; Tâches et manuel
  (AC-6) alignés ; 35-3b-CR3-P2-2 l'exemple de `RATE_UNAVAILABLE` (AC-2) était le `PUT` d'une écriture ancienne à
  lignes `afc` après restauration, que A17 fait passer pour un libellé seul : exemple restreint aux lignes nouvelles ou
  modifiées. LOW : P2-3 AC-1 « l'écriture d'ouverture ne se modifierait plus » (contraire à A17) ; P2-4 « erreurs de
  moins d'un demi-centime » faux — au plus 0.005, atteint au cas `x.xx5` de la fiche (0.885 → 0.89) ; P2-5 avis
  « cours changé » sans critère de déclenchement (cours d'insertion retenu par l'indicateur) ; P2-6 préalables E2E :
  l'absence de taux désactive par `noRates`, non `configMissing` (`VatPurchaseAssistant.svelte:55-56`) ; P2-7 « compte
  débité (côté EUR) / crédité (côté USD) » de l'AC-2 bis, sens du relevé contraire au grand livre, résidu de CR-3 P1-5
  non propagé. Axes exercés : diff `b6f88c24..d9c0018e` de la fiche ; A17 et 35-3a AC-4 (l. 221-229, texte encore
  marqué « à réécrire ») ; code : `vat-purchase.ts:36-99` (`lineVatAmount`, `buildPurchaseVatLines`, TVA nulle → deux
  lignes), `VatPurchaseAssistant.svelte:16-121` (props, `configMissing`, `noRates`, `htValid`, `canInsert`, `insert`) ;
  35-1 AC-4 bis (`CreateAccountRequest` prend `fxGainAccountId`/`fxLossAccountId` : préalable E2E réalisable) ;
  vecteurs S5 recalculés (12.29 × 0.081 = 0.99549 → 1.00 ; 10.87665 → 10.88 ; 0.885 → 0.89 ; 11.76165 → 11.76 ; écart
  0.01 ≤ 3 × 0.005) ; comptabilité de la conversion (sens au grand livre). Un GET de la source journalière :
  `xmldaily?d=20261231` (date future) rend `<datum>13.11.2026</datum>` sans erreur — la garde « veille strictement
  passée » de l'AC-1 (et 35-2, l. 85-86) est donc nécessaire, et tenue. Non exercés : sauvegarde, MCP, audit (aucune
  table ni action neuve depuis CR-2), i18n (clés neuves non énumérées), E2E, Vitest, PDF.
- 2026-11-14 — validation CR-3 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche, aucun
  inter-fiches. LOW : 35-3b-CR3-P3-1 la liste des champs d'une ligne `afc` « modifiée » (AC-2, `RATE_UNAVAILABLE`) omettait
  la provenance, que 35-3a AC-4 (4) compte (`manual` repassée `afc`). Axes exercés : diff `d9c0018e..040185ac` de la fiche,
  phrase par phrase ; A17 (epic l. 202-203) et 35-3a AC-4 (l. 221-235), AC-7 (l. 414) ; `VatPurchaseAssistant.svelte:55-56`
  (`configMissing`, `noRates`) ; sens débit/crédit de la conversion (l. 362 contre l. 375-376) ; arithmétique du seuil
  n × 0.005 et du cas `x.xx5` (l. 329-330) ; cohérence Tâches et AC-6. Non exercés : sauvegarde, MCP, audit, i18n, E2E,
  Vitest, PDF.
- 2026-11-14 — validation CR-3 P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche, aucun
  inter-fiches. LOW : 35-3b-CR3-P4-1 la liste de l'AC-2 (`RATE_UNAVAILABLE`) écrivait « côté débit/crédit » et
  « montant » là où 35-3a AC-4 (4) compare débit et crédit en valeur et le montant de `foreign`, et omettait que ni
  l'ordre ni le projet n'entrent dans la comparaison (le `PUT` d'un projet ou de lignes CHF passe aussi) : alignée mot
  pour mot. Axes exercés : diff `040185ac..53a885f4` de la fiche ; 35-3a AC-4 (l. 224-235) ; cohérence avec « Reprendre
  le cours attendu » (cours changé, ligne recontrôlée au cours attendu) et « Garder mon cours » (provenance `manual`,
  hors contrôle), l. 230-238 ; Vitest de l'édition du seul libellé ; AC-1 l. 74-78. Non exercés : sauvegarde, MCP,
  audit, i18n, E2E, PDF, code.
- 2026-11-14 — révision CR-4 (revue 2 de Beat T1–T6 ; Opus) : **T3** — la ligne ne stocke pas de devise (35-3a AC-1) :
  types TS — réponse `foreign.currency: string` lue sur le compte, requête `currency?: string` facultative ; le
  formulaire, les deux grilles d'ouverture, l'assistant TVA et l'assistant *Conversion* l'envoient toujours (garde
  contre un compte dont la devise a changé depuis le chargement) ; `LineDraft.foreign.currency` = devise du compte,
  jamais saisie ; messages de `FX_CURRENCY_MISMATCH` (recharger) et du code neuf `FX_ACCOUNT_IN_CHF` (35-3a AC-4) ;
  affichage du code par la réponse ; tests d'API des deux routes d'ouverture ; guide d'API (AC-1, AC-2, AC-3, AC-6).
  **T4** — cours d'ouverture proposé = cours de clôture de l'exercice précédent : genre `day` inchangé (veille de
  l'ouverture) ; genre `month` → cours mensuel du **dernier mois de l'exercice précédent** (mois de la veille de
  l'ouverture : décembre, juin, mars), et non plus le premier mois du nouvel exercice ; calcul partagé avec `openingDate` de
  35-2 ; tests et manuel (AC-1, AC-6). Remplace, pour le genre `month`, la phrase de CR-2 « cours mensuel du premier
  mois » (entrée ci-dessus, laissée telle quelle). T1, T2, T5, T6 sans autre point de contact. AC touchés : en-tête,
  AC-1, AC-2, AC-3, AC-6. Fiche à revalider (cycle CR-4).
- 2026-11-14 — validation CR-4 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW ; remédiés dans la fiche sauf
  35-3b-CR4-P1-2 pour sa part dans 35-2 (inter-fiche). MEDIUM : P1-1 la formule du genre `month` (« 1er du mois de
  `start_date` − 1 jour ») se lit « un jour avant le 1er du mois d'ouverture », d'où février pour une ouverture au
  15.03 (T1 permet un début quelconque ; 35-2 AC-5 dit « 1er du mois de cette veille ») : formule et test du 15.03
  écrits ; P1-3 tests manquants de T3 : corps envoyé par les deux grilles, l'assistant TVA et l'assistant *Conversion*
  portant la devise du compte ; messages et Vitest de `FX_CURRENCY_MISMATCH` et `FX_ACCOUNT_IN_CHF` (quatre locales).
  LOW : P1-2 « fonction de calcul partagée » entre la page TypeScript et l'écran *Cours de change* (Rust, 35-2 AC-5)
  irréalisable : fonction TS aux vecteurs de 35-2 ; P1-4 Tâches sans item CR-4 ; P1-5 test « dix-huit mois » redondant
  avec « 1er juillet » (la durée est sans effet sur la date d'ouverture ; T2 six mois) et E2E du genre `month` absent.
  **Inter-fiche non appliqué — texte proposé.** *35-2 AC-5, l. 525* : remplacer « et le cours proposé à l'ouverture
  (35-3b AC-1) » par « ; la page des soldes d'ouverture (35-3b AC-1), en TypeScript, recalcule la même date par une
  fonction pure TS dont les vecteurs sont ceux des tests de la fonction Rust (une fixture JSON commune, comme
  `fx_rounding.json`, est préférable) ». Axes exercés : diff `2c0f5afc..bd7d0ec7` de la fiche ; T3 contre 35-3a AC-1,
  AC-2, AC-4 (rangs 1 à 3), AC-6 ; T4 contre 35-1 AC-3 bis (l. 204), 35-2 AC-4 (l. 337-339) et AC-5 (l. 522), A22 ;
  `create_opening_complement_in_tx` (le contrôle de devise passe par `create_in_tx`, après les verrous ; aucun résidu de
  colonne `currency` de ligne dans la fiche) ; types TS ; comptabilité de l'écart et de la conversion inchangée. Non
  exercés : sauvegarde, audit, MCP (aucune table, action ou sortie neuve ici), source journalière (aucun GET), E2E,
  Vitest, PDF.
- 2026-11-14 — validation CR-4 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW ; tous remédiés dans la fiche.
  MEDIUM : 35-3b-CR4-P2-1 (AC-5, E2E du genre `month` ajouté en P1) — « ouverture au 1er juillet : juin » contredit le
  seed `with-company` qu'utilise l'E2E (exercice 01.01.2020–31.12.2030, `test_fixtures.rs:118-119` : décembre 2019) ;
  et l'enchaînement `day` puis `month` ne tient pas si l'ouverture est enregistrée en `day` (la page passe au
  complément, cours du mois de sa date) : mois du seed écrit, cours vérifiés au préremplissage, enregistrement après le
  passage en `month`. LOW : P2-2 (AC-1) vecteurs « (jour, mois, 29.02, 15.03, 1er janvier) … même fixture si 35-2 la
  fournit, sinon recopiés » — 35-2 AC-5 fournit désormais `fx_opening_dates.json` (cas réunis, 29.02 compris) : fixture
  nommée, alternative retirée ; P2-3 (AC-1) renvoi « 35-1 T1 » → 35-1 AC-11 (T1). Axes exercés : diff
  `bd7d0ec7..a8f89e04` de la fiche ; entrée CR-4 P1 ; formule du mois de la veille (15.03, 01.04, 01.07) contre epic T4
  et 35-2 AC-4/AC-5 ; début quelconque contre 35-1 AC-11 ; parité TS/Rust contre 35-2 AC-5 (fixture) ; codes
  `FX_CURRENCY_MISMATCH` / `FX_ACCOUNT_IN_CHF` contre 35-3a AC-3/AC-4 (`details.line`) ; seed E2E au code ; item
  Tâches CR-4. Non exercés : code applicatif hors seed (aucune ligne neuve citée), source factice de 35-2 AC-3 dans le
  détail, 35-4/35-5, manuel, PDF.
- 2026-11-14 — validation CR-4 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW ; aucune remédiation. Axes
  exercés : diff `a8f89e04..2f792399` de la fiche ; seed `with-company` (`test_fixtures.rs:118-119`, 2020-01-01 :
  veille 31.12.2019, décembre 2019) ; fixture `fx_opening_dates.json` contre 35-2 AC-5 (chemin, genres `day`/`month`,
  cas 1er janvier/avril/juillet, 15.03, 01.03.2024, six et dix-huit mois : tous couverts ; le 01.01.2000 de 35-2 n'est
  pas exigé ici, sans contradiction) ; renvoi 35-1 AC-11 (T1) ; enchaînement `day` puis `month` de l'E2E contre AC-1
  (complément au cours du mois de sa date) ; source factice au mois demandé (35-2 AC-3). Non exercés : code
  applicatif hors seed, 35-4/35-5, manuel, PDF.
- 2026-11-14 — révision CR-5 (revue 3 de Beat U1–U6 ; Opus) : **U1** — cours proposé = cours retenu par la société
  (`exchange_rates` de 35-2) : `lookup` rend `effectiveRate` et `source` (et `publishedRate`, non affiché ici) ; ses
  `afcRate` et `overridden` disparaissent ; `afcRate` du formulaire devient une valeur dérivée (`effectiveRate` d'un
  `lookup` `source: "afc"`), si bien que la règle de provenance (`afc` si égal, au mois de la date) et le contrôle
  `FX_RATE_NOT_AFC` de 35-3a jugent contre la même valeur ; indicateurs « AFC » / « cours saisi » lus sur `source`
  (formulaire et soldes d'ouverture) ; ↑ sous le champ vers le cours de la société, sans toucher la table ; 404 de
  `lookup` = aucune ligne de la société (le `POST /fetch` copie) ; restauration : les cours de la société reviennent ;
  « adoption » remplacée par la reprise par ↑, par société ; Vitest de provenance récrits ; E2E (`PUT
  /exchange-rates/rates`, ↑) ; manuel. **U2** — changer le compte d'une ligne : CHF → devise (champs initialisés),
  devise → CHF (`foreign` retiré), USD → EUR (montant et cours réinitialisés, cours EUR proposé), même devise (gardés) ;
  compté comme retouche ; Vitest et E2E. **U5** — A12 noté approuvé ; A8, A13, A14, A21 déjà AC (AC-2, AC-2 bis).
  **U6** — débit et crédit en devise principale (AC-1, AC-2, ligne d'écart). Aucune migration ni route neuve. AC
  touchés : en-tête, AC-1, AC-2, AC-5, AC-6, Tâches. Fiche à revalider (cycle CR-5).
- 2026-11-14 — validation CR-5 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; remédiés dans la fiche sauf
  35-3b-CR5-P1-6 (35-2, inter-fiche). MEDIUM : CR5-P1-1 `FX_RATE_NOT_AFC` à `expectedRate` `null` (cours de la société
  devenu `manual` après l'écriture, 35-3a AC-4, 4) : « Reprendre le cours attendu » ne pouvait pas s'appliquer — seule
  « Garder mon cours » est offerte, Vitest et E2E ; CR5-P1-2 le manuel (AC-6) disait « moyenne du 25 au 24 », contre la
  Terminologie de Beat (« pas des moyennes ») et R4 amendé (méthode non confirmée) : réécrit. LOW : P1-3 E2E du cours
  saisi et du ↑ irréalisables tels quels (`version` d'une ligne `afc` copiée, valeur publiée requise par `reset`, sinon
  suppression d'une ligne `manual`) ; P1-4 AC-2 « le ↑ et son dialogue » contre A23 (↑ direct) ; P1-5 `PUT /rate-kinds`
  exige les deux genres ; P1-6 (inter-fiche) 35-2 AC-3 bis. **Inter-fiche non appliqué — texte proposé.** *35-2 AC-3 bis,
  fonction de copie* : remplacer « **toutes** les devises publiées de l'unité (une devise d'un compte créé plus tard
  trouve ainsi son cours) » par « les devises **portées par un compte de la société** (A24) ; donner une devise à un
  compte déclenche la copie des cours déjà récupérés pour cette devise (35-1, 35-2) ». Axes exercés : diff
  `0ae43432..d59a1de9` de la fiche ; `lookup`, `PUT …/rates`, `…/rates/reset`, `PUT /rate-kinds` et copie de 35-2
  (AC-3 bis, AC-5, AC-7) ; 35-3a AC-4 (4) et AC-7 (U2) ; reste des anciens noms (`exchange_rate_overrides`,
  `_kesh_exchange_rates`, `republished_*`, `overridden`, `afcRate` du `lookup`) : aucun hors Change Log ; U1, U2, U3, U6,
  A23, A24 ; Terminologie. Non exercés : sauvegarde et rejeu (aucune table neuve ici), audit, MCP, source journalière
  (aucun GET), E2E, Vitest, PDF, citations de lignes de code antérieures à CR-5.
