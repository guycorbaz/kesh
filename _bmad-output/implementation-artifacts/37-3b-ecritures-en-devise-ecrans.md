# Story 37-3b : écritures en devise — soldes d'ouverture, formulaire et écrans

Status: review

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/37-multi-devises`. Epic 37, C-37-4, C-37-5. Suit **37-3a**
(découpe de l'ancienne 37-3 à la rédaction). **Aucune migration.** Révisée le 2026-11-14 pour la revue de Beat
(CR-2 : R3, R6, R7, R8), qui prime sur le registre antérieur ; elle ajoute une route serveur (assistant *Conversion*,
AC-2 bis). Révisée de nouveau pour la revue CR-3 de Beat, qui prime sur CR-2 : **S2** — la ligne d'écart et l'assistant
*Conversion* prennent les comptes de différence **propres** du compte en devise concerné (37-1 AC-4 bis) ; **S5** —
l'assistant TVA achat calcule la TVA sur le montant en devise et passe la ligne de TVA en CHF au même cours (AC-2) ;
**S6** — l'écran de saisie n'affiche pas les cours AFC republiés (c'est 37-2). Révisée enfin pour la revue 2 de Beat
(CR-4), qui prime sur tout ce qui précède : **T3** — la ligne ne stocke pas de devise (37-3a AC-1) : `currency` se lit
sur le compte, facultatif à l'envoi (AC-1, AC-2, AC-3) ; **T4** — cours d'ouverture proposé = cours de **clôture**
de l'exercice précédent : veille de l'ouverture (genre `day`) ou cours mensuel du **dernier mois de l'exercice
précédent** (genre `month`) (AC-1). Révisée en dernier pour la revue 3 de Beat (CR-5), qui prime sur tout : **U1** — le
cours proposé est le **cours retenu par la société** (`exchange_rates` de 37-2 : `effectiveRate` et `source` de
`lookup` ; ses champs `afcRate` et `overridden` n'existent plus, `afcRate` devient une valeur dérivée du formulaire,
AC-2), et les indicateurs se lisent sur `source` (AC-1,
AC-2, AC-5, AC-6) ; S6 tel qu'écrit (« adoption ») devient la reprise par ↑ d'un cours AFC corrigé, par société ;
**U2** — changer le compte d'une ligne réinitialise montant et cours, vers un compte CHF retire `foreign` (AC-2) ;
**U5** — A8, A12, A13, A14 approuvés ; **U6** — débit et crédit restent en devise principale (AC-1, AC-2).
**Révisée le 2026-11-16 pour CR-7 (A34, prime sur tout ; modèle en tête de 37-1)** : un exercice mono-devise d'une
société multi-devises n'est plus qu'un exercice clos avant l'activation puis rouvert ; l'écran ne le traite plus à
part : le masquage des champs et des comptes en devise par exercice (formulaire, grilles d'ouverture, assistants) est
retiré, le serveur refuse (409 `MULTI_CURRENCY_DISABLED`, `details.fiscalYearId`) et l'écran affiche ce refus traduit ;
les motifs `monoCurrencyFiscalYear` et `laterFiscalYear` et le cycle avec la bascule de 37-1 sont retirés.*

## Story

En tant que comptable en multi-devises, je veux saisir à l'écran une écriture en devise avec le cours du mois
proposé (cours mensuel AFC, ou le cours que ma société a saisi, D9), saisir mes propres cours (ou le montant CHF réel),
y ajouter d'un clic une ligne de différence de change, passer une conversion entre deux devises par l'assistant
*Conversion* (deux écritures par le compte de conversion, R7), et ouvrir mes comptes en devise avec leur solde en
devise au cours d'ouverture choisi pour l'exercice (R6), afin de tenir mes livres sans passer par l'API.

## Exigences

*(B26, 2026-11-15 : tirées des critères d'acceptation ci-dessous, sans en créer ni en modifier.)*

**Fonctionnelles**

- FR-37-3b-1 — Les soldes d'ouverture et leur complément acceptent, pour un compte en devise, un montant en devise et un cours (ou le CHF), serveur et écran, sous la même forme `foreign` et la même fonction d'analyse que 37-3a ; envoyés `manual` par l'écran (l'API admet aussi `afc`, contrôlé, ou l'absence de `rateSource`, qualifiée par le dépôt) ; plusieurs devises admises ; une ligne en devise incomplète n'est jamais retirée en silence ; le serveur ne fait aucun `lookup` et ne lit `exchange_rates` qu'après ses verrous (accès de 37-3a dans `create_in_tx_inner`), le complément ne faisant que des lectures verrouillantes avant les siens (34-2 AC-10) (AC-1).
- FR-37-3b-2 — Le formulaire d'écriture saisit une ligne en devise (Débit ou Crédit), calcule le CHF arrondi et alimente le solde, le cours OU le CHF se saisissant à la main (cours déduit par `rateFromChf`, `manual` ; mais si le cours `afc` du mois de la date, `afcRate`, redonne ce CHF, le cours devient `afcRate` et la ligne part `afc`, A32) ; changer le compte d'une ligne fait suivre la devise (montant et cours réinitialisés vers une autre devise, gardés vers un compte de la même devise, `foreign` retiré vers un compte CHF) ; dès qu'une ligne porte une devise, les comptes d'une autre devise sont retirés du sélecteur (R7), sauf, pour une écriture relue qui en porte déjà plusieurs (ouverture, complément, contre-passation de l'une d'elles, état hérité), les devises qu'elle porte, et un lien propose l'assistant *Conversion* (AC-2).
- FR-37-3b-3 — Le bouton « Ajouter la différence de change » **ajoute une ligne CHF à l'écriture en cours**, sans rien envoyer, sur les comptes de différence du compte en devise (`fxAccountsFor`) ; l'assistant *Conversion* passe une conversion en deux écritures par le compte de conversion, ligne de différence calculée, non modifiable et visible à l'aperçu ; « Passer les écritures » est désactivé, avec le motif, quand une date n'est pas dans un exercice ouvert, que les deux comptes ne sont pas de deux devises différentes, que la paire de différence résolue est invalide, ou qu'une ligne de frais de l'écriture 2 sur un second compte en devise **qui n'est pas un compte de charge de la même devise** donne une différence non nulle ; un frais au débit d'un compte de charge de la devise (p. ex. 6911 *Frais PayPal USD*) est admis et n'entre jamais dans la différence (A33) (AC-2, AC-2 bis).
- FR-37-3b-4 — Le détail et la liste des écritures montrent montant en devise, cours et source ; une écriture de réévaluation n'est pas modifiable (`ENTRY_IS_FX_REVALUATION`), « Supprimer » restant offert hors période verrouillée et lettrage (AC-3).
- FR-37-3b-5 — Les sélecteurs des pièces (factures, avoirs, règlements, factures fournisseur), lots, réconciliation, comptes bancaires liés, comptes par défaut et produits n'offrent pas les comptes en devise (un seul prédicat) ; le formulaire d'écriture, les soldes d'ouverture, l'assistant *Conversion* et les écrans de consultation ne filtrent pas, et l'assistant TVA achat restreint lui-même à CHF plus une devise (AC-4).
- FR-37-3b-6 — Cours proposé et provenance : le formulaire préremplit le cours retenu par la société pour le mois de la date (`lookup`, repli unique `POST /exchange-rates/fetch` sur `404 RATE_UNAVAILABLE`, pour les rôles Comptable et Administrateur et un mois à la portée du `fetch` ; sinon saisie manuelle), le repropose au changement de date pour les seules lignes proposées, envoie `afc` seulement si le cours égale le cours `afc` du mois, `manual` sinon, conserve la provenance relue d'une ligne non retouchée ; le cours d'ouverture proposé est le cours de clôture de l'exercice précédent (veille de l'ouverture, ou dernier mois de l'exercice précédent selon le genre) ; un complément daté `TODAY` propose le cours mensuel du mois de sa date, et la date recalculée par le serveur d'un autre mois ne fait pas refuser le `POST` (cours saisi, `manual`) ; quand le cours retenu par la société pour le mois de la date diffère de celui de la ligne, il s'affiche sous le champ avec ↑, qui remet la ligne à ce cours sans jamais toucher la table des cours ; les refus `FX_RATE_NOT_AFC` offrent « Reprendre le cours attendu » / « Garder mon cours », cette seule seconde action quand `expectedRate` est `null` (AC-1, AC-2).
- FR-37-3b-7 — L'assistant TVA achat passe en devise quand la charge ou la contrepartie est un compte en devise : TVA calculée en devise, ligne d'impôt préalable en CHF au même cours, écart d'arrondi éventuel soldé par le bouton d'écart ; charge et contrepartie dans deux devises différentes, ou ligne CHF dérivée arrondie à 0.00, désactivent l'insertion avec le motif ; règle d'arrondi écrite au guide d'API pour un client (AC-2).
- FR-37-3b-8 — Route `POST /api/v1/journal-entries/conversions` : deux écritures dans une transaction, tout ou rien, chacune soumise à tous les contrôles de `POST /journal-entries` ; compte de conversion revalidé (`FX_ACCOUNT_INVALID`) et soldé (`FX_CONVERSION_TRANSIT_UNBALANCED`) ; ligne de différence calculée par le serveur **sur demande seulement** (`completeDifference`, A31) sur l'écriture qui rend le compte de conversion à 0 (compte de conversion au crédit net, un seul compte en devise hors comptes de charge, au débit net, plus d'éventuels frais au débit de comptes de charge de la même devise, comptés avant la différence — A33 ; frais au crédit → 400 `Validation`, compte d'une autre devise → `FX_MULTIPLE_CURRENCIES` ; déséquilibrée sous une autre forme, elle rend `ENTRY_UNBALANCED`, jamais complétée), sur la paire `fxAccountsFor` revalidée sous verrou à l'usage seulement (`FX_ACCOUNT_INVALID`) ; une ligne de différence portée par le corps est contrôlée sur la paire résolue, avec ou sans le champ ; 400 `Validation` pour `entries` de longueur ≠ 2, une ligne portée hors de la paire ou sans compte en devise unique hors comptes de charge, `completeDifference` avec une ligne déjà portée, « trop de lignes » ; refus d'une écriture portant `details.entry` (1 ou 2 ; `FX_ACCOUNT_INVALID` et `FX_CONVERSION_TRANSIT_UNBALANCED`, qui visent la conversion entière, n'en portent pas), une seule raison dans l'ordre écrit (contrôles par écriture avant la transaction, puis sous les verrous `companies` → compte de conversion : `FX_ACCOUNT_INVALID` du compte de conversion, transit, équilibre de l'autre écriture, forme et paire de l'écriture à compléter, puis les deux créations dans l'ordre chronologique), rien de créé ; réponse 201 `{ entries }` dans l'ordre du corps ; mêmes rôles que `POST /journal-entries`, audit `journal_entry.created` ×2, route inscrite `Traced, Rejouee` (AC-2 bis).
- FR-37-3b-9 — Contrat des routes d'ouverture : à la génération, une ligne CHF dont `debit` ou `credit` est absent rend 400 et non plus 422 (au complément, une colonne absente vaut toujours 0) (Note de version, guide d'API) ; `rateSource: "revaluation"` rend 400, non le 422 d'axum, sur les deux routes ; chaque refus FX du complément rend son code en 400, contrôlé avant la transaction, jamais un 500 (AC-1).
- FR-37-3b-10 — Les refus du serveur propres aux devises (`RATE_UNAVAILABLE`, `FX_RATE_NOT_AFC`, `FX_AMOUNT_MISMATCH`, `FX_CURRENCY_MISMATCH`, `FX_ACCOUNT_IN_CHF`, `ACCOUNT_REQUIRES_CURRENCY`, `MULTI_CURRENCY_DISABLED` (exercice mono-devise, `details.fiscalYearId`), `FX_MULTIPLE_CURRENCIES`, et, pour l'assistant, `FX_ACCOUNT_INVALID` et le code neuf `FX_CONVERSION_TRANSIT_UNBALANCED`) s'affichent traduits (quatre locales), saisie conservée, au formulaire, aux deux grilles d'ouverture et dans l'assistant *Conversion* — ceux qui portent `details.line` (de même `FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO`) sur la ligne fautive, retrouvée par le rang 1-based dans le corps envoyé, qui omet les lignes vides ; dans l'assistant, sur la colonne de `details.entry` par la table qui a construit le corps, et au-dessus de l'aperçu sans `details.entry` ; en édition, ils laissent le formulaire ouvert (`stay`), `ENTRY_IS_FX_REVALUATION` recharge la fiche (`stale`) ; `RATE_UNAVAILABLE` propose « Récupérer le mois » puis le renvoi, ou le passage en `manual` (AC-1, AC-2, AC-2 bis).

**Non fonctionnelles**

- NFR-37-3b-1 — Aucune régression mono-devise : formulaire strictement inchangé dans une société mono-devise ; mode Compléter bancaire inchangé (AC-1, AC-2, AC-5).
- NFR-37-3b-2 — Motifs de blocage dans les quatre locales, test des codes à jour (AC-3).
- NFR-37-3b-3 — Inventaire clos des sélecteurs au Change Log, tenu par une garde Vitest qui lit le source (AC-4).
- NFR-37-3b-4 — E2E sans réseau : cours servis par la source factice du mode test ; refus d'une écriture à deux devises vérifié par l'API (AC-5).
- NFR-37-3b-5 — Manuel utilisateur (quatre locales, `.tex` seulement — PDF non commité, régénéré à la release, rendu contrôlable en local —, garde G4-bis des numéros de compte) et guide d'API (`docs/api-external.md`) à jour (AC-6).
- NFR-37-3b-6 — Calculs de l'écran jumeaux de ceux du serveur : `computeChf` et `rateFromChf` (`big.js`, arrondi loin de zéro explicite) tenus égaux par les fixtures partagées de 37-3a, mêmes bornes et mêmes codes d'erreur ; la date du cours d'ouverture par une fonction pure TS, dont le Vitest lit la fixture `fx_opening_dates.json` de 37-2 (aucune copie), et `lineVatAmount` jumeau de `line_vat_amount` (AC-1, AC-2).

## Critères d'acceptation

1. **AC-1 — Soldes d'ouverture** (serveur et écran).
   - **Routes.** `generate_opening_balances` (`routes/opening_balances.rs` :345 → `journal_entries::create_opening_entry`
     :655) et le complément (`complete_opening_balances` :593 → `opening_complement.rs` `create_opening_complement` :460, enveloppe en portée `Screen` de
     `create_opening_complement_in_tx` :551, qui sert aussi le mode Compléter de la 34-2 sur les comptes bancaires)
     acceptent, pour un compte en devise, le **solde en devise** et un **cours**. Un compte CHF reste inchangé.
   - **DTO.** `OpeningBalanceLineRequest` et `OpeningComplementLineRequest` prennent `foreign?: { currency?, amount, rate?, chf?,
     rateSource?, side? }` (forme de 37-3a AC-6, CR-6 : `rate` ou `chf`, `rateSource` facultatif, `side` requis si `debit`/`credit` sont absents — **même type** que le `foreign` de `CreateJournalEntryLineRequest` : champs
     `String` analysés par la même fonction, `currency` facultatif — T3 : absent, la ligne prend la devise du compte ;
     présent, il doit l'égaler, `FX_CURRENCY_MISMATCH` ; jamais stocké ; les deux grilles envoient toujours `currency`, `rate`, `rateSource` et `debit`/`credit`, comme le
     formulaire, AC-2 —, d'où un **400** et non le 422 d'axum pour `rateSource: "revaluation"`,
     statut testé sur les deux routes) ; `debit`/`credit` restent les montants en **devise principale** (aujourd'hui
     **CHF** ; U6), **fournis par le client et
     vérifiés** par la règle de 37-3a (`FX_AMOUNT_MISMATCH` sinon) — **ou complétés par la fonction d'analyse partagée
     de 37-3a AC-6 (V1, V2, CR-6)**, la seule que la route emploie : `foreign { amount, chf }` sans `rate` → cours déduit
     par `rate_from_chf` de `kesh-core` (37-3a AC-3) ; `debit`/`credit` absents → CHF posé dans la colonne de
     `foreign.side` (`OpeningBalanceLineRequest.debit`/`credit` passent en `Option<String>`, comme ceux du complément,
     `routes/opening_balances.rs:75-76` contre `:173`, `:175`). **Ligne CHF (sans `foreign`)** : à la génération, une colonne
     absente rend 400 `Validation` (« débit invalide » / « crédit invalide », le refus d'une chaîne vide aujourd'hui, `:362-368`) — jamais
     un 422 d'axum, qui était le statut d'un champ manquant (changement écrit à la Note de version et au guide d'API, tests
     existants de la route adaptés, inventaire au Change Log) ; au complément, une colonne absente vaut 0, comme aujourd'hui
     (`parse_complement_amount(None)`), sans changement ; la route ne calcule rien d'autre. `rateSource`
     `revaluation` refusé ; l'écran envoie toujours `manual` (cf. *Cours proposé*) ; l'API admet `afc`, alors contrôlé
     par `FX_RATE_NOT_AFC` de 37-3a contre le mois de la date retenue, et l'absence de `rateSource`, alors qualifiée
     par le dépôt (37-3a AC-4, 4 : `afc` seulement si le cours égale le cours `afc` retenu du mois de la date retenue,
     `manual` sinon ; **A32** : une ligne `{ amount, chf }` sans `rate`, sans `rateSource` ou à `afc`, dont
     `round(montant × cours retenu, 2)` redonne son CHF — mois dont la ligne de la société est `afc` —, prend le cours
     retenu et `afc` (à `afc` sinon : `FX_RATE_NOT_AFC` ou `RATE_UNAVAILABLE`) — la règle du cours déduit, non une égalité avec le cours déduit à 10
     décimales ; les grilles d'ouverture, qui envoient toujours `manual`, ne l'emploient pas). Le sens est celui de la colonne (`debit` ou
     `credit`) qui porte le CHF. Complément : mêmes bornes de forme que `parse_complement_amount` (`opening_balances.rs:515`) sur le CHF résultant ;
     génération : ses montants se lisent par `Decimal::from_str` (`:362-368`), sans ce parseur — les bornes y viennent du
     contrôle FX de 37-3a et de `validate`. Dans les deux cas, les bornes de 37-3a AC-3 (montant ≤ 10¹³ à 4 décimales, cours ≤ 10⁶ à 10 décimales) sur la devise ; `parse_complement_lines`
     (:535-580, « exactement une colonne non nulle », borne de `Σ(débit − crédit)`) s'applique au CHF.
   - **Contrôle FX du complément avant la transaction.** `parse_complement_lines` complète d'abord chaque ligne
     `foreign` par la fonction d'analyse partagée (V1, V2, CR-6 : cours déduit, CHF posé selon `side`), puis lui
     applique le contrôle FX pur de 37-3a AC-3 **avant** la règle « exactement une colonne non nulle » (comme 37-3a AC-3 pour
     l'exclusivité : sans cet ordre, une ligne dont le CHF s'arrondit à 0 ou manque rendrait l'erreur de montant
     générique au lieu de `FX_LINE_ROUNDS_TO_ZERO` / `FX_AMOUNT_MISMATCH`) (bornes, signe, `FX_LINE_ROUNDS_TO_ZERO`, `FX_AMOUNT_MISMATCH` ; fonction publique de
     `kesh-core`), sans lecture de la base. Dans le dépôt, `accounting::validate` s'exécute sous verrou et son erreur
     devient `DbError::Invariant("opening-complement:unbalanced:…")` (`opening_complement.rs:699-700`), rendu en 500
     `INTERNAL_ERROR` (`errors.rs:4808-4813`) : ce `map_err` reste le filet d'un défaut de calcul, jamais le refus d'une
     saisie. Test d'API : chaque refus FX du complément rend son code en 400, jamais 500. Le refus est converti par `map_core_error` (déjà importé par la route) et son `details.line` est le rang **1-based** de la ligne dans `req.lines` (celui du corps envoyé, 37-3a AC-3). (La génération appelle
     `validate` dans la route, `:448`, par `map_core_error` : rien à déplacer.)
   - **`ComplementLine`** (`opening_complement.rs`) gagne `foreign: Option<ForeignAmount>` **sans** `Default` ; chaque
     site de construction est revu et inventorié au Change Log : `bank_accounts.rs` :764 (`opening_line` : `None`
     motivé — la 34-2 ne touche jamais un compte en devise, un compte lié ne pouvant porter de devise, 37-1), le
     parseur de la route (:566), l'aide de test `line()` de la route (`opening_balances.rs:634-645`, littéral
     `OpeningComplementLineRequest`, qui gagne `foreign`), les tests de `tests/opening_complement_repository.rs` et de
     `tests/opening_complement_bank_scope.rs`. **Propagation jusqu'à la ligne écrite** (sites où 37-3a pose `None`,
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
     "manual"`) ; champs `effectiveRate` et `source` de `lookup`, 37-2 AC-5) du **genre d'ouverture de
     l'exercice** de l'écriture d'ouverture (`openingRateKind` du premier exercice, 37-1 AC-1, réglé à l'écran *Cours
     de change*, 37-2 AC-6 ; lu sur `GET /fiscal-years`) : genre `day` (défaut) → **cours journalier de la veille de
     l'ouverture** (`lookup` `kind=day`, `date` = `start_date` − 1 jour : le 31.12 pour un exercice civil, le 30.06
     pour un exercice au 1er juillet, le 31.03 pour un exercice au 1er avril), qui est le cours de clôture de
     l'exercice antérieur ; genre `month` → **cours mensuel du dernier mois de l'exercice précédent** (T4 — non plus du
     premier mois du nouvel exercice), c'est-à-dire du mois de la veille de l'ouverture (`lookup` `kind=month`,
     `date` = le **1er du mois de la veille**, la veille étant `start_date` − 1 jour — non « le 1er du mois de
     `start_date`, moins un jour » : pour une ouverture au 15.03, la veille est le 14.03 et le mois est **mars**, non
     février ; l'assistant de démarrage laisse choisir un début quelconque, 37-1 AC-11 (T1) : décembre pour un exercice civil,
     juin pour un exercice au 1er juillet, mars pour un exercice au 1er avril ou au 15.03), que l'exercice précédent soit tenu dans Kesh ou non — la
     règle se calcule sur la seule date d'ouverture, sans lire l'exercice précédent ; même date que `openingDate` du
     tableau annuel de 37-2 (AC-5) : la fonction pure de 37-2 est en Rust et la page des soldes d'ouverture est en
     TypeScript — elles ne se partagent pas ; la page calcule la date par **une** fonction pure TS (nom au
     développement), dont le Vitest lit la fixture commune de 37-2 AC-5, `crates/kesh-core/tests/fixtures/fx_opening_dates.json`
     (même fichier que le test Rust, aucune copie ; genres `day` et `month`, 1er janvier, 1er avril, 1er juillet,
     15.03, 01.03.2024 — veille 29.02 —, six et dix-huit mois). **Client de l'API (V9, CR-6)** : il ne recalcule ni la
     date ni le genre — le cours d'ouverture proposé est la case **`opening`** de `GET /fiscal-years/{id}/exchange-rates`
     (37-2 AC-5 : `effectiveRate` et `source` à `openingDate`, calculée par la fonction pure de `kesh-core` ; `null`
     sans ligne de la société ; 409 `MULTI_CURRENCY_DISABLED` pour un exercice mono-devise) ; le jumeau TS ci-dessus
     ne sert que l'écran ; renvoi écrit au guide d'API (AC-6). Ce cours est celui de la réévaluation de l'exercice précédent quand son genre de clôture est
     `month` (37-5) ; le genre d'ouverture d'un exercice qui suit un exercice Kesh reprend par défaut le genre de
     clôture de celui-ci (37-1 AC-3 bis), si bien que la concordance tient sans réglage. Hors des bornes du `POST /fetch` de 37-2 (date < 2000-01-01, AC-5 : il rendrait 400), le
     `lookup` — qui n'a pas de borne basse et rend alors `404 RATE_UNAVAILABLE` faute de cours — n'est suivi d'**aucun**
     `POST /fetch` : saisie manuelle. Pour un complément daté `TODAY` (`complementDateKind`), cours mensuel du mois de cette date, comme
     avant. Modifiable. Le `404 RATE_UNAVAILABLE` d'un jour (`details.date`) déclenche `POST /fetch { date }` si la
     veille est strictement passée (37-2 AC-5), sinon saisie manuelle. Tests : exercice civil (1er janvier), au
     1er juillet, au 1er avril, en genre `day` et `month` — genre `month` : `lookup` de décembre de l'année
     précédente, de juin, de mars (jamais du mois d'ouverture) ; ouverture au 15.03 : mars (veille 14.03) ; la durée de
     l'exercice est sans effet (six et dix-huit mois commençant le 01.07 : même date, juin, T2). **Envoyé `rate_source = manual`** sur
     l'ouverture et le complément, proposé ou saisi : le contrôle `FX_RATE_NOT_AFC` de 37-3a (AC-4, 4) compare une
     ligne `afc` au cours **mensuel** du mois de `entry_date`, à la création **et** au `PUT` pour une ligne nouvelle ou
     modifiée (arbitrage A17) — un cours journalier, ou d'un mois que la date recalculée du complément a quitté, y
     serait refusé dès la création, et toute ligne retouchée de l'écriture d'ouverture le serait aussi. L'écran affiche la provenance (« proposé : cours journalier du JJ.MM.AAAA » ou « proposé : AFC
     <mois> », ou « cours saisi » quand `source = "manual"`, « cours saisi (<source brute>) » pour toute autre source que
     `"afc"`, p. ex. « cours saisi (ecb) » (A30, comme AC-2 ; Vitest) — U1 : la provenance se lit sur `source`, sans
     comparaison de valeurs, comme au formulaire, AC-2) ; le Change Log de la story le dit. Le genre `day`
     (défaut) propose le cours de clôture de l'exercice antérieur, celui de sa réévaluation (37-5) : un solde repris d'un
     bilan réévalué tombe donc juste ; sinon il se saisit au CHF (« cours OU CHF » ci-dessous), ce qui reprend exactement
     le CHF de clôture. Sur `404 RATE_UNAVAILABLE` du `lookup`, même repli qu'à l'AC-2 (un `POST /exchange-rates/fetch`,
     puis relecture ; sinon saisie manuelle).
   - **Exercice mono-devise (CR-7)** : l'écran ne le traite pas à part. Un premier exercice ou un exercice du complément
     mono-devise (clos avant l'activation puis rouvert) fait refuser une ligne sur un compte en devise par le serveur
     (37-3a AC-4, rang 2 : 409 `MULTI_CURRENCY_DISABLED`, `details.fiscalYearId`) ; les deux grilles affichent ce refus
     traduit, classé comme tout refus FX de l'écran, saisie conservée. L'écriture d'ouverture et son complément peuvent
     porter plusieurs devises (exemption de `FX_MULTIPLE_CURRENCIES`, 37-3a AC-4, 2 bis). ⚠️ La page
     (`settings/opening-balances/+page.svelte`) ne charge aujourd'hui **aucun** exercice : elle charge `GET /fiscal-years`
     au montage (`openingRateKind` ci-dessus). Si ce chargement échoue : **aucun** cours n'est proposé (genre inconnu) —
     saisie manuelle, avec un message ; Vitest de ce cas.
   - **Saisie manuelle : cours OU CHF** (ouverture, complément et formulaire d'écriture, AC-2). L'utilisateur saisit
     soit le cours, soit le montant CHF. Si le CHF est saisi : cours = CHF ÷ montant en devise, arrondi à 10 décimales ;
     le client vérifie que `round(montant × cours, 2)` redonne le CHF saisi et, sinon, ajuste le cours au dernier chiffre
     jusqu'à ce que ce soit le cas (borné : si aucun cours à 10 décimales ne convient, message, saisie refusée). Sert à
     reprendre exactement le CHF du bilan de clôture précédent ou le CHF réel d'une conversion ; `rate_source = manual`.
     **Calcul (V1, CR-6, DRY)** : ce calcul est celui de `rate_from_chf` de `kesh-core` (37-3a AC-3), que le serveur
     applique à une ligne de l'API sans `rate` ; l'écran l'exécute par son **jumeau** `rateFromChf` (`fx.ts`, `big.js`,
     même borne de pas, mêmes codes d'erreur `FX_VALUE_OUT_OF_RANGE` / `FX_AMOUNT_MISMATCH`, message traduit), tenu égal
     par la fixture partagée `crates/kesh-core/tests/fixtures/fx_rate_from_chf.json` que lit son Vitest (même fichier,
     aucune copie) — jamais un calcul écrit à côté dans un composant. L'écran envoie le cours ainsi obtenu (`rate`) et
     le CHF, comme avant : le serveur le recontrôle (`FX_AMOUNT_MISMATCH`).
   - **Le serveur ne fait aucun `lookup`** (la résolution du cours est faite à l'écran, avant l'appel) et ne lit
     `exchange_rates` (37-2, U1) **avant aucun verrou** : `create_opening_complement_in_tx` exige que la transaction n'ait
     fait que des lectures verrouillantes avant ses verrous — AC-10 de la 34-2. Les seuls accès à la table sont ceux de 37-3a dans `create_in_tx_inner`, après les
     verrous : le contrôle `FX_RATE_NOT_AFC` d'une ligne `afc` venue de l'API et la qualification d'une ligne **sans**
     `rateSource` (V1, CR-6, AC-4, 4) ; le contrôle de devise de 37-3a AC-4 y reste aussi.
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
     du compte (T3), `foreign` sur un compte CHF → `FX_ACCOUNT_IN_CHF`, exercice mono-devise (clos avant l'activation puis rouvert ;
     fabriqué par les routes : exercice créé puis clos, activation, `POST /fiscal-years/{id}/reopen`) → `MULTI_CURRENCY_DISABLED`,
     `revaluation` refusé ; Vitest (CR-4, T3) : le corps envoyé par chacune des deux grilles porte
     `foreign.currency` = la devise du compte de la ligne ; ouverture **et** complément à lignes EUR et USD acceptés **par leurs routes** (exemption de
     `FX_MULTIPLE_CURRENCIES`, 37-3a AC-4, 2 bis), puis `PUT` du libellé de l'écriture d'ouverture ainsi créée → 200 ;
     aller-retour `.keshbackup` ; **non-régression du mode Compléter bancaire** (34-2, compte CHF inchangé) ; Vitest de
     l'écran et des totaux (`opening-balances-page.test.ts`, `opening-balances-totals.test.ts`), dont : refus
     `MULTI_CURRENCY_DISABLED` (`details.fiscalYearId`) affiché sur chacune des deux grilles, saisie conservée.
2. **AC-2 — Formulaire d'écriture** (`JournalEntryForm.svelte`), en multi-devises seulement :
   - **Sens.** Une ligne en devise se saisit dans la colonne **Débit ou Crédit en montant en devise** (le sens est
     celui de la colonne) ; le CHF est calculé (`round(montant × cours, 2)`, `Big.roundHalfUp`), affiché en lecture
     seule sous le champ, et alimente `computeBalance` et la ligne d'écart. `debit`/`credit` du payload restent les
     montants en devise principale (aujourd'hui CHF ; U6), `foreign.amount` le montant en devise. `LineDraft` (`form-helpers.ts` :13) gagne
     `foreign?: { currency, side: 'debit' | 'credit', amount, rate, rateSource }` (absent = ligne CHF ; T3, CR-4 :
     `currency` du brouillon est **la devise du compte** de la ligne, `account.currency` lue sur `AccountResponse` au
     choix du compte — jamais saisie —, et `lineResponseToDraft` la reprend de `foreign.currency` de la réponse, elle
     aussi lue sur le compte par le serveur, 37-3a AC-6) — **contrat de
     stockage** : le montant saisi en devise vit dans `foreign.amount` et le sens dans `foreign.side` (il doit survivre
     à un cours ou un CHF encore vides) ; `debit`/`credit` de la ligne ne portent **que le CHF dérivé** (l'un des deux
     vide, selon `side`), recalculé à chaque changement et seul lu par `computeBalance`, `classifyLine` et
     `isDraftLineNonEmpty` — aucun d'eux ne lit jamais `foreign.amount` comme du CHF ; une ligne en devise sans CHF
     calculable (cours ou montant vide ou invalide) est `partial` (le message de la ligne est celui du code rendu par `computeChf` — `FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO` — traduit, non un « montant invalide » générique) ; `classifyLine` (`balance.ts` :143-162),
     `computeBalance` (`balance.ts` :83), `isDraftLineNonEmpty` (`vat-purchase.ts` :106) et les fabriques de lignes
     (`JournalEntryForm` :105-106, :187 ; `vat-purchase.ts` :87-96) sont revus, inventaire au Change Log.
     **Payload (T3)** : `foreign: { currency, amount, rate, rateSource }` — le type TS de la requête porte
     `currency?: string` (facultatif, comme l'API, 37-3a AC-6), mais le formulaire, les deux grilles d'ouverture,
     l'assistant TVA et l'assistant *Conversion* l'envoient **toujours** (la devise affichée) : c'est la garde contre
     un compte dont la devise a changé depuis le chargement (`FX_CURRENCY_MISMATCH`, ci-dessous) ; le type de la
     **réponse** porte `currency: string` (toujours présent dans `foreign`). Vitest : le corps envoyé porte la devise
     du compte, y compris pour les lignes que produisent l'assistant TVA et l'assistant *Conversion* (leur Vitest).
     **Changer le compte d'une ligne (U2, CR-5)** — la devise de la ligne **suit le compte** (elle n'est verrouillée
     qu'au compte, 37-1 AC-4 ; le serveur juge la ligne sur le nouveau compte, 37-3a AC-7) : CHF → devise, champs en
     devise initialisés (montant vide, cours proposé par `lookup` de la nouvelle devise) ; devise → CHF, `foreign`
     **retiré** (la ligne redevient une ligne CHF, montant CHF vide) ; devise → **autre** devise (USD → EUR) :
     **montant et cours réinitialisés** (montant en devise vide, cours reproposé par `lookup` de la nouvelle devise,
     `rateProposed` vrai, CHF recalculé à vide) — jamais un montant USD gardé sous l'étiquette EUR ; devise → même devise
     (deux comptes USD) : montant et cours gardés, cours reproposé seulement si `rateProposed`. Le changement de compte
     compte comme une **retouche** (indicateur de l'édition, ci-dessous : la provenance relue n'est plus conservée).
     Vitest (A30) : `lookup` rendant `source: "ecb"` → `afcRate` `null`, ligne envoyée `manual`, indicateur « cours saisi (ecb) ».
     Vitest de chacun des quatre cas, dont : USD → EUR en édition d'une ligne `afc` relue → montant vide, cours EUR
     proposé, corps envoyé en EUR ; USD → CHF → aucune clé `foreign` dans le corps.
   - **Champs** : *Montant (USD)*, *Cours* (prérempli par le **cours effectif** que rend `lookup` pour le mois de la
     date, `effectiveRate` — le cours retenu par la société, U1 ; indicateur « AFC » (`source: "afc"`), « cours saisi »
     (`source: "manual"`, cours de la société, D9 ; pour toute **autre** source, p. ex. `"ecb"`, A30 : « cours saisi » suivi de la source brute entre parenthèses, « cours saisi (ecb) ») ou « manuel » (retouché à la main) ; modifié à la main → `manual`),
     *CHF* ; **provenance envoyée** (D9, U1) : on note **`afcRate`** l'`effectiveRate` d'un `lookup` dont `source`
     vaut `"afc"` (`null` si `source` ≠ `"afc"` — `"manual"` ou toute autre source, p. ex. `"ecb"`, traitée comme `manual` (A30) —, ou sans cours) — c'est exactement le cours auquel le serveur
     compare une ligne `afc` (37-3a AC-4, 4) ; `rateSource = 'afc'` si le cours de la ligne **égale** `afcRate` (non
     `null`) **du mois de la date** (`afcMonth`, puce suivante), comparaison en valeur (`Big.eq`, non en graphie),
     `manual` sinon — un cours saisi par la société, ou un mois sans cours `afc` retenu, part donc `manual` ; la
     dernière valeur publiée (`publishedRate` de `lookup`) n'entre jamais dans ce choix ; aucune autre valeur. C'est la
     règle par laquelle le serveur qualifie une ligne de l'API envoyée **sans** `rateSource` (V1, CR-6, 37-3a AC-4, 4) ;
     l'écran, lui, envoie toujours `rateSource` (sa provenance relue en édition y compris). **CHF saisi à la main (A32,
     report IF2 de 37-3a)** : si `afcRate` du mois de la date redonne ce CHF (`round(montant × afcRate, 2)`), le cours
     devient `afcRate` et la ligne part `afc` — même règle que le serveur pour une ligne au cours déduit (37-3a AC-4, 4) ;
     sinon cours de `rateFromChf`, `manual` (Vitest : CHF qui redonne `afcRate` → cours `afcRate`, `afc` ; CHF
     qui ne le redonne pas, ou mois sans `afcRate` → cours de `rateFromChf`, `manual`). Comme à
     l'AC-1, **soit le cours soit le CHF** se saisit à la
     main (CHF saisi → cours par `rateFromChf`, jumeau de `rate_from_chf`, fixture `fx_rate_from_chf.json`, AC-1 —
     CHF ÷ montant à 10 décimales, vérification `round(montant × cours, 2) = CHF`, ajustement au dernier chiffre) ; changer la date repropose le cours effectif des lignes dont le cours n'a pas été modifié à
     la main. `LineDraft.foreign` gagne pour cela un indicateur `rateProposed` (cours venu de `lookup`, non retouché),
     distinct de `rateSource` : une ligne au cours saisi de la société est `manual` **et** proposée ; en édition, une
     ligne relue `afc` est tenue pour proposée, une ligne relue `manual` pour non proposée (`rateProposed` faux : aucune
     reproposition au changement de date) — notion distincte de l'indicateur de **retouche** ci-dessous, qui gouverne
     la provenance envoyée, la provenance relue étant, elle, conservée dans `loadedRateSource`.
   - **Mois du cours AFC comparé** (37-3a AC-4, 4 : toujours le mois de `entry_date`). `LineDraft.foreign` retient
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
     (`source: "manual"` ; toute autre source, A30 : « Cours saisi (<source brute>) : <cours> ») et un bouton **↑** (« Reprendre le cours de la société ») qui remet **la ligne** à ce cours
     (`rateSource` = `afc` si `source` vaut `"afc"`, `manual` sinon, toute autre source comprise (A30) ; ligne proposée : un changement de date ultérieur
     la repropose au cours effectif du nouveau mois, comme toute ligne proposée) — action locale du formulaire ; il ne
     touche **jamais** la table des cours de la société (le ↑ de cette table, qui reprend une valeur publiée, vit à
     l'écran *Cours de change*, 37-2 AC-6). Ligne au cours de la société : indicateur « AFC » ou « cours saisi », pas
     de ↑.
   - **Appels `lookup`** : mémoïsation par `(devise, mois)` par formulaire, **résultats négatifs compris** ; appel
     seulement pour une date valide dans `dateBounds` et après débounce (`debounce.ts`) ; jeton de requête pour ignorer
     une réponse périmée ; si le nouveau mois n'a pas de cours, le cours **proposé** précédent est **effacé** (jamais
     gardé sous l'étiquette `afc` ni comme proposition) ; en édition, aucune reproposition au montage, seulement au
     changement de date et pour les lignes proposées (`rateProposed`). `GET /lookup` est **en lecture seule** (37-2) et
     ne rend 404 que si la société n'a **aucune** ligne `exchange_rates` pour ce mois (un cours saisi seul rend 200,
     `source: "manual"`, sans `POST /fetch`) : sur `404 RATE_UNAVAILABLE`, l'écran appelle **une
     fois** `POST /exchange-rates/fetch` pour ce mois — par la fonction d'API de 37-2 (seule entrée : sans bandeau
     dégradé sur 503, `timeoutMs` de 90 s, 37-2 AC-3 et AC-5), le mois tiré de `details.month` (`AAAA-MM-01`) ; les specs
     E2E pilotent la source factice et s'appuient sur le vidage des tables par `/_test/seed` (37-2 AC-7) — (rôle Comptable ou Administrateur ; mois ≥ 2000-01 et ≤ **mois suivant** le mois courant, **à toute date**, 37-2 AC-5 et R4 amendé, non « ≤ mois courant » : le mois suivant peut être publié avant le 25), puis
     relit ; `404 RATE_NOT_PUBLISHED`, `503` ou rôle Consultation → message, saisie manuelle exigée. Pour un mois
     **postérieur au mois suivant le mois courant** (Europe/Zurich ; 37-2 AC-5 : le mois suivant est admis à toute
     date, non publié → `404 RATE_NOT_PUBLISHED` ; R4 amendé, alignement de l'orchestrateur),
     **aucun** `POST /fetch` (il rendrait 400, 37-2 AC-5) : saisie manuelle directement ; un 400 du `POST /fetch` se
     traite comme `RATE_NOT_PUBLISHED`. Le cours du mois M est publié le 25 de M−1 ou le jour ouvrable suivant selon l'ESTV, mais parfois avant (relevé du 14.11.2026, à confirmer par Beat ; R4 amendé ; 37-2 *Source*), et la tâche
     automatique de 37-2 (une vérification par jour jusqu'à son arrivée, puis pause, S1) le récupère pour les exercices multi-devises ; le cas « cours indisponible » vise surtout un
     mois futur, une devise non publiée ou une source injoignable. *(Annotation de l'orchestrateur, 2026-11-14 : la
     remédiation P1 avait écrit l'inverse ; précisée par R4, CR-2.)* La
     relecture qui suit un `POST /fetch` réussi **remplace** l'entrée négative mémoïsée de (devise, mois) — elle ne la
     relit pas —, et ses `afcRate`/`afcMonth` rafraîchissent les lignes comme un `lookup` de changement de date.
   - **Formats et bornes** — celles de 37-3a AC-3, à l'identique : le cours se valide avec `AmountFormatOptions`
     `{ maxDecimals: 10, maxIntegerDigits: 7 }` (`balance.ts` :23-31) puis cours ≤ 10⁶, rejette 0 ; le montant en
     devise : 4 décimales, ≤ 10¹³ ; message d'erreur propre au cours (le « Maximum 4 décimales » existant ne vaut que
     pour les montants). `computeChf` rend l’erreur `FX_VALUE_OUT_OF_RANGE` sur ces bornes **et sur un montant ou un cours ≤ 0** (37-3a AC-3 étape 2 : même code) (les vecteurs `error` de la
     fixture partagée l'exigent), **et aussi pour un CHF arrondi > 10¹³** (borne
     de 37-3a AC-3, vecteur « CHF > 10¹³ » de la fixture) — une seule borne, au client comme au serveur, et non les
     quinze chiffres entiers de `DECIMAL(19,4)` ; un cours déduit d'un CHF saisi passe par les mêmes bornes. Le payload sérialise par `toFixed`, jamais `toString()`.
   - **Refus du serveur propres aux devises** (`POST` et `PUT`) : `RATE_UNAVAILABLE {currency, month}` — p. ex. le
     `PUT` d'une écriture ancienne dont une ligne `afc` est **nouvelle ou modifiée** (compte, débit ou crédit, devise, montant en devise, cours,
     provenance — une ligne `manual` repassée `afc` au même cours est modifiée — ou date de l'écriture ; ni l'ordre des
     lignes ni le projet), pour un mois où la société n'a aucun cours retenu (p. ex. exercice passé en multi-devises
     avant toute récupération ; depuis U1, une restauration ramène les cours de la société, 37-2 AC-7) ; le
     `PUT` du seul libellé, de lignes CHF ou d'un projet passe, ses lignes `afc` inchangées n'étant jamais recontrôlées (37-3a AC-4, arbitrage A17) — →
     l'écran propose « Récupérer le mois » (`POST /exchange-rates/fetch`, mêmes rôles et même
     règle que ci-dessus) puis le renvoi, ou le passage de la ligne en `manual` ; `FX_RATE_NOT_AFC` → le cours attendu
     (`details.expectedRate`, 37-3a AC-4) est affiché et proposé à la reprise ; `FX_AMOUNT_MISMATCH` porte
     `details.expected` / `details.provided` (37-3a AC-3) ; `FX_CURRENCY_MISMATCH` (T3 : la devise annoncée diffère
     de celle du compte — elle a changé depuis le chargement ; le message dit de recharger la page, `details.expected`
     nomme la devise du compte), `FX_ACCOUNT_IN_CHF` (ligne en devise sur un compte devenu CHF ; même conseil),
     `ACCOUNT_REQUIRES_CURRENCY`, `FX_AMOUNT_MISMATCH` → message traduit sur la ligne fautive (clés des deux messages neufs
     `FX_CURRENCY_MISMATCH` et `FX_ACCOUNT_IN_CHF` dans les quatre locales ; Vitest de leur affichage au formulaire, aux
     deux grilles d'ouverture et dans l'assistant *Conversion*, sur la bonne ligne ; numéro en `details` : le rang **1-based** de la ligne dans `lines`,
     lu dans `details.line` (37-3a AC-3), y compris pour `FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO` et
     `FX_RATE_NOT_AFC`). `MULTI_CURRENCY_DISABLED` avec `details.fiscalYearId` (exercice clos avant l'activation,
     rouvert, 37-3a AC-4, 2) a son message — aucun compte en devise n'est utilisable dans cet exercice —, clé i18n dans
     les quatre locales, au formulaire, aux deux grilles d'ouverture et dans l'assistant *Conversion* ; Vitest
     (CR-7 : motif `monoCurrencyFiscalYear` retiré). ⚠️ Ce rang est celui du **corps envoyé**, qui omet les lignes `empty` (`nonEmptyLines`,
     `JournalEntryForm.svelte:171-174`, payload :206) : l'écran le traduit en ligne du formulaire par l'`index` de
     `nonEmptyLines[line − 1]`, jamais par `lines[line − 1]` ; même règle sur les grilles d'ouverture (`nonEmptyRows`,
     `complementLines`). Vitest de chacun, dont un refus sur une ligne précédée d'une ligne vide. `editRefusalOutcome`
     (`form-helpers.ts` :108) classe ces refus en édition : `ENTRY_IS_FX_REVALUATION` → `stale` (la fiche se recharge,
     le motif s'affiche) ; `RATE_UNAVAILABLE`, `FX_*`, `ACCOUNT_REQUIRES_CURRENCY`, `MULTI_CURRENCY_DISABLED` (exercice mono-devise, `details.fiscalYearId`) → `stay` (le formulaire reste
     ouvert) ; sinon ils tomberaient en `other`. Test par code. Un `FX_RATE_NOT_AFC` sur une ligne au cours saisi
     de la société ne devrait pas se produire (elle part `manual`) : s'il survient, même traitement. **Après une
     reprise** (cours AFC corrigé **repris par ↑** à l'écran *Cours de change*, pour cette société, 37-2 AC-5 et AC-6,
     U1, U3), il survient aussi au `PUT` d'une écriture ancienne, mais **pour
     ses seules lignes `afc` nouvelles ou modifiées** (37-3a AC-4, arbitrage A17 : une ligne inchangée n'est jamais
     recontrôlée, si bien que le `PUT` du seul libellé passe) — typiquement une ligne dont seul le **montant** a été
     corrigé, qui garde sa provenance relue `afc` sans `lookup` (*Mois du cours AFC comparé*) ; une ligne dont la
     date ou le cours a changé a, elle, relu le cours retenu par la société et part `manual` s'il diffère. Pour ces lignes, le
     refus propose **deux** actions, « Reprendre le cours attendu » (`details.expectedRate`, `afc`, le CHF change) et
     « Garder mon cours » (la ligne passe en `manual` au cours qu'elle porte, son CHF n'est pas recalculé) ; aucune
     action sur les autres lignes. **`expectedRate` à `null`** (37-3a AC-4, 4 : le cours de la société pour ce mois est
     désormais **`manual`** — un `PUT …/rates` l'a saisi après l'écriture — et il n'y a pas de cours `afc` à reprendre) :
     « Reprendre le cours attendu » n'est **pas offert**, seule « Garder mon cours » l'est, avec un message qui dit que
     la société a fixé son propre cours pour ce mois (CR5-P1-1) ; Vitest de ce cas. Vitest des deux actions, et Vitest d'une édition du seul libellé d'une écriture `afc`
     (corps aux lignes inchangées, provenance relue, aucun `lookup`). **U1, CR-5 (remplace S6 de CR-3)** : l'écran de
     saisie n'affiche **ni** les corrections de l'AFC **ni** l'avertissement de connexion — le cours proposé et la
     mention sous le champ sont ceux du **cours retenu par la société** que rend `lookup` (`effectiveRate`, `source`) ;
     `publishedRate` de `lookup` n'est pas affiché ici ; le rouge « AFC corrigé », le ↑ (direct tant que l'option « toutes les sociétés » est masquée, A23 ; son dialogue
     n'apparaît qu'ensuite) et
     l'acquittement vivent à l'écran *Cours de change* et au bandeau de 37-2 (AC-6).
   - **Ligne d'écart** (R8) : écriture déséquilibrée en devise principale (aujourd'hui CHF) avec au moins une ligne en devise → bouton « Ajouter la
     différence de change », qui **ajoute une ligne à l'écriture en cours** — il ne crée **aucune** écriture et
     n'envoie rien : une ligne CHF sur le compte de gains (écart au crédit) ou de pertes (au débit) du **compte en
     devise concerné** — S2, CR-3 : `fxAccountsFor(compte, société)` (37-1 AC-4 bis), soit ses comptes **propres**, soit
     ceux de la société —, que l'utilisateur voit, modifie ou retire avant d'enregistrer. **Compte concerné** (une
     écriture ne porte qu'une devise, R7, mais peut toucher plusieurs comptes de cette devise, p. ex. PayPal USD et
     ventes USD) : le premier compte en devise **de bilan** (`Asset`/`Liability`) dans l'ordre des lignes, sinon le
     premier compte en devise ; si les comptes en devise de l'écriture ne résolvent pas tous la même paire, la ligne
     est posée sur celle du compte retenu et un avis nomme ce compte (le compte de la ligne reste modifiable) —
     arbitrage A12, approuvé par Beat (CR-5) ; au-delà de n × 0.005 CHF (n = nombre de lignes en
     devise, calculé en `Big`), le texte précise qu'il ne s'agit pas d'un arrondi ; rien n'est ajouté sans clic
     (C-37-4). Libellé accessible et aide du bouton : « Ajouter une ligne de différence de change ». Vitest : un clic
     ajoute exactement une ligne et aucun appel réseau.
   - **Une devise par écriture (R7)** : dès qu'une ligne porte une devise, les comptes d'une **autre** devise sont
     retirés du sélecteur des autres lignes (les comptes CHF restent) ; une écriture relue qui en porte déjà plusieurs —
     **l'écriture d'ouverture et son complément** (exemptés, 37-3a AC-4, 2 bis), une contre-passation de l'une d'elles,
     **ou** un état hérité — **reste modifiable** : 37-3a AC-4 (2 bis) n'oppose `FX_MULTIPLE_CURRENCIES` au `PUT` que si
     le corps porte plus d'une devise **et** une devise que l'écriture stockée ne portait pas. Pour une telle écriture, le sélecteur offre les comptes CHF et ceux
     des devises **déjà portées par l'écriture relue** (jamais une devise de plus) ; un `FX_MULTIPLE_CURRENCIES` reçu
     malgré tout (corps qui ajoute une seconde devise, écriture changée entre la lecture et l'envoi) s'affiche traduit,
     `stay`. Pour changer de devise, l'assistant *Conversion* (AC-2 bis) est proposé par un lien (Vitest : avec une
     ligne USD, le sélecteur des autres lignes propose les comptes CHF et USD, aucun compte EUR ; le lien apparaît ;
     retirer la ligne en devise rend les comptes de toutes les devises ; en édition d'une ouverture à lignes EUR et USD,
     le sélecteur propose CHF, EUR et USD, aucune autre devise, et l'enregistrement d'un libellé changé part sans
     refus). Le formulaire reçoit les props `multiCurrency`,
     `fxGainAccountId`, `fxLossAccountId` (le **défaut** de la société ; les comptes propres d'un compte en devise se
     lisent sur son `AccountResponse`, `fxGainAccountId?`/`fxLossAccountId?`, 37-1 AC-4 bis, et `fxAccountsFor` compose
     les deux), fournies par `journal-entries/+page.svelte` (:163) et
     `journal-entries/[id]/+page.svelte` (:305), qui chargent déjà `fetchCompanyCurrent`. Si cet appel échoue
     (les deux pages le tolèrent), `multiCurrency` vaut vrai dès qu'un compte de `accounts` porte une devise (37-1 : un
     compte en devise n'existe qu'en multi-devises ; `AccountResponse.currency` et `currencyLocked` sont **omis** pour
     un compte CHF, 37-1 AC-4 et C-37-9 : type TS `currency?: string`, tout test de devise lit `account.currency ?? null`,
     jamais `=== null` — alignement de l'orchestrateur sur 37-1-P5-IF1) — sans quoi l'édition d'une écriture en devise perdrait `foreign`
     au `PUT` ; les comptes de différence restent alors inconnus et le bouton d'écart l'explique. **Contrôle client seulement**
     (la ligne est une ligne CHF ordinaire envoyée au `POST` ; la revalidation serveur de C-37-2 (3) vaut pour la
     réévaluation) : le bouton revalide sur la liste des comptes la paire résolue par `fxAccountsFor` (existe, actif,
     imputable, `Expense`/`Revenue`, sans devise) ; comptes non désignés ou invalides : il explique quoi régler, en
     nommant l'origine (« comptes propres du compte 1021 » ou « réglage de la société »). Vitest : compte en devise à
     comptes propres → ligne sur ses comptes ; compte « par défaut » → comptes de la société ; deux comptes USD à paires
     différentes → compte de bilan retenu et avis ; compte propre archivé → explication, aucune ligne.
   - **Arrondi** : calcul côté client avec `big.js` dans une fonction pure (`fx.ts`, `computeChf`), `.round(2,
     Big.roundHalfUp)` **explicite** (pas `Big.RM` global), même résultat que le serveur (`MidpointAwayFromZero`) ;
     Vitest lit la fixture de vecteurs **partagée** `crates/kesh-core/tests/fixtures/fx_rounding.json` fournie par 37-3a
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
     préalable, contrepartie TTC, équilibrées par construction). Dans une société **multi-devises** (CR-7 : plus de
     règle par exercice), ses deux sélecteurs (charge, contrepartie) offrent aussi les comptes en devise
     (`excludeForeignCurrency = false`, AC-4) ; le compte d'impôt préalable reste celui des réglages de facturation, en
     CHF (37-1 : un compte désigné ne reçoit pas de devise). Dès que la charge ou la contrepartie est un compte en
     devise, l'assistant passe **en devise** : sa devise est celle de ce compte (charge et contrepartie en devise
     doivent porter **la même** devise — l'écriture reste à une seule devise étrangère, R7 ; l'autre sélecteur ne
     propose plus que les comptes CHF et ceux de cette devise), le champ *Montant HT* se lit dans cette devise
     (« Montant HT (USD) »), et un champ *Cours* prérempli par le `lookup` du mois de la date du formulaire (même
     fonction mémoïsée que les lignes, même règle `afc`/`manual`, mêmes bornes ; « cours OU CHF » non offert ici :
     le cours seul) — props neuves du composant : la date de l'écriture, le mode de son exercice, le réglage de la
     société et la fonction de `lookup` du formulaire (aucun second cache). Les lignes produites sont des `LineDraft`
     complets (`foreign` avec `side`, `rateProposed`, etc., contrat de l'AC-2), construits par la même fabrique que
     le formulaire. Calcul (fonction pure de `vat-purchase.ts`, `big.js`, arrondi `Big.roundHalfUp` explicite, la
     règle de 37-3a AC-3 par `computeChf`) : `HT` = round(montant HT, 2) et `TVA` = `lineVatAmount(HT, taux)` **en
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
     ligne d'impôt préalable. **Limite écrite (V5, CR-6)** : l'assistant TVA reste un calcul **de l'écran**, comme en CHF
     aujourd'hui (aucune route ne calcule une écriture d'achat avec TVA) ; un client de l'API passe les trois lignes
     lui-même, par `POST /journal-entries`, selon la **règle d'arrondi** ci-dessus, écrite au guide d'API (AC-6) :
     `HT` = montant HT arrondi au centime ; `TVA` = `round(HT × taux ÷ 100, 2)` en devise, loin de zéro — la règle de
     `kesh_core::accounting::vat::line_vat_amount` (`vat.rs:40`), dont `lineVatAmount` (`vat-purchase.ts:36`) est le
     jumeau ; `TTC` = `HT` + `TVA` ; ligne sur un compte en devise : `foreign` (montant, cours) et son CHF = `round(montant
     × cours, 2)` loin de zéro — ou `debit`/`credit` omis, que le serveur pose (37-3a AC-6, V2) ; ligne sur un compte CHF
     (impôt préalable toujours) : `round(montant en devise × cours, 2)` au **même** cours ; un écart d'un centime se
     solde par une ligne d'écart (37-3a AC-6, V4). Aucun calcul serveur n'est ajouté : la règle tient en une ligne et
     ses fonctions existent déjà en Rust. Libellé suggéré inchangé. **Tests** (Vitest de `vat-purchase.ts` et du composant) :
     achat **USD 100.00, TVA 8,1 %, cours 0.8846**, charge CHF et contrepartie PayPal USD → TVA USD 8.10, charge CHF
     88.46, TVA CHF 7.17 (7.16526), contrepartie USD 108.10 / CHF 95.63, équilibré ; **cas `x.xx5`** : USD 12.29,
     8,1 %, cours 0.8850 → TVA USD 1.00, TVA CHF 0.885 → **0.89** (arrondi loin de zéro), charge CHF 10.88
     (10.87665), contrepartie USD 13.29 / CHF 11.76 (11.76165) : débit 11.77, crédit 11.76, écart d'un centime
     proposé par le bouton d'écart sur les comptes de différence de la contrepartie (`fxAccountsFor`, S2) ; charge
     et contrepartie dans deux devises différentes → refusé à l'écran (insertion désactivée, motif) ; société
     mono-devise → assistant inchangé (CHF seul) ; taux exempt (TVA 0) → deux lignes, aucune ligne d'impôt
     préalable. Vecteurs relevés au calcul exact le 2026-11-14 (arrondi au centime, loin de zéro).
   - **Mono-devise** : formulaire strictement inchangé (test de non-régression) dans une société mono-devise. Le
     formulaire ne lit pas le mode des exercices (CR-7 : prop `fiscalYears` et masquage par exercice retirés) ; dans un
     exercice mono-devise d'une société multi-devises (clos avant l'activation, rouvert), le serveur rend 409
     `MULTI_CURRENCY_DISABLED`, `details.fiscalYearId`, classé `stay` par `editRefusalOutcome`, message traduit,
     saisie conservée. Vitest de ce refus.
2 bis. **AC-2 bis — Assistant *Conversion* (R7).** Une conversion entre deux devises (EUR → USD) ne tient pas dans une
   écriture (37-3a, `FX_MULTIPLE_CURRENCIES`) : elle passe par le **compte de conversion** en CHF de la société (37-1,
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
   - chaque ligne de frais est **au débit**, en CHF ou dans **la** devise de son écriture (EUR à l'écriture 1, USD à
     l'écriture 2), sur un compte choisi (les comptes en devise d'une autre devise ne sont pas offerts) ; les écritures
     peuvent donc avoir plusieurs lignes. **A33** : à l'écriture 2, un frais sur un **compte de charge** de la devise
     (`accountType` `Expense`, p. ex. 6911 *Frais PayPal USD*) est admis : il n'est pas un « second compte en devise »,
     il entre au débit avant la différence (la différence de change ne l'absorbe jamais) et l'aperçu le montre sur sa
     ligne. Une ligne de frais de l'écriture 2 sur un compte en devise **autre** que celui de la colonne *Crédit* et
     **qui n'est pas un compte de charge de la même devise** (p. ex. un second compte bancaire USD), avec une
     différence non nulle, donnerait au serveur une écriture à compléter à deux comptes en devise portant une ligne de
     différence (paire indéterminée, 400 `Validation`, *Ligne de différence calculée par le serveur* — contrôle qui
     vaut sans `completeDifference`) : « Passer les écritures » est alors désactivé avec ce motif (Vitest, avec le cas
     contraire : frais sur 6911 USD → bouton actif).
   **Écran** : depuis *Écritures*, bouton « Conversion de devises » (société multi-devises, rôles qui saisissent des
   écritures) ; page ou dialogue en deux colonnes, cours proposés et « cours OU CHF » comme au formulaire (AC-2 :
   `lookup` du mois de **chaque** date, provenance `afc`/`manual`, bornes, `computeChf` et la fixture partagée), aperçu
   des deux écritures avec la ligne de différence calculée, puis **« Passer les écritures »**. Chaque date doit tomber
   dans un exercice ouvert, sinon le motif s'affiche et le bouton est désactivé ; un exercice mono-devise (CR-7 : clos
   avant l'activation, rouvert) est refusé par le serveur (409 `MULTI_CURRENCY_DISABLED`), refus affiché. Le compte
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
   (37-3a, dont `FX_MULTIPLE_CURRENCIES` et `MULTI_CURRENCY_DISABLED` par exercice) ; en plus : compte de conversion
   de la société revalidé à l'usage (de la société, actif, imputable, en CHF, de bilan ; sinon 400 `FX_ACCOUNT_INVALID`,
   `details` de 37-1, `field: "fxTransitAccountId"`) ; il figure dans les deux écritures, au débit de l'une et au
   crédit de l'autre, pour le même CHF (sinon 400 `FX_CONVERSION_TRANSIT_UNBALANCED`, code neuf, quatre locales,
   `details: { entry1, entry2 }`) ; un refus d'une écriture rend son code avec `details.entry` (1 ou 2) en plus de ses
   `details` (un refus sans `details` en reçoit : mécanisme commun, au développement, à inventorier au Change Log), et
   rien n'est créé ; `entries` de longueur ≠ 2 → 400 `Validation` ; le compte de conversion peut figurer sur plusieurs
   lignes d'une écriture : le contrôle compare le **solde net** de ses lignes (débit − crédit) dans chaque écriture.
   **Ligne de différence calculée par le serveur, sur demande (V3, CR-6, A31)** — pour qu'une conversion se passe par l'API seule :
   - **demande (A31)** : le corps porte `completeDifference` (booléen, facultatif, `false` par défaut). Le serveur ne
     calcule **jamais** la ligne d'office : sans ce champ (ou à `false`), une écriture déséquilibrée rend
     `ENTRY_UNBALANCED` (`details.entry`), comme aujourd'hui, et **aucune ligne n'est calculée ni ajoutée** — mais
     l'identification de l'écriture à compléter, sa forme, le contrôle de la paire d'une ligne de différence **portée**
     par le corps et l'ordre des refus ci-dessous valent **dans les deux cas** (P3-1, CR-6 : l'écran n'envoie pas le
     champ et envoie la ligne qu'il montre à l'aperçu, D6 ; cette ligne-là doit rester contrôlée) ; seuls l'ajout de la
     ligne calculée et son refus propre (« trop de lignes ») dépendent du champ ; avec `completeDifference: true` **et**
     une ligne de différence déjà présente
     dans l'écriture à compléter (ligne sur un compte désigné) → 400 `Validation` (`details: { entry, line }`) : le
     client choisit entre la ligne qu'il porte et celle du serveur, jamais les deux (champ non booléen → 400 de
     l'extracteur du dépôt, testé) ;
   - **écriture à compléter** (identifiée avec ou sans la demande ; complétée seulement avec elle) : **une seule**, celle dont le **solde net du compte de conversion est un crédit** (le
     rôle de l'écriture 2 de l'assistant : elle rend le compte de conversion à 0), jamais choisie sur le seul critère
     « déséquilibrée » — sans quoi une erreur d'un client sur l'écriture 1 serait soldée en silence par un gain ou une
     perte de change. Elle est jugée en devise principale (Σ débit ≠ Σ crédit, sur le corps complété par l'analyse de
     37-3a AC-6, V1/V2) : équilibrée → rien n'est ajouté (différence nulle, ou ligne portée par le corps) ;
     déséquilibrée → la ligne ci-dessous. L'écriture à compléter est bien déterminée **après**
     `FX_CONVERSION_TRANSIT_UNBALANCED` (le compte de conversion est au débit net de l'une et au crédit net de l'autre),
     et seulement sous verrou (l'identifiant du compte de conversion est lu dans la transaction). **Forme exigée pour
     être complétée (P2, CR-6 ; A33)** : ses lignes sur des comptes en devise **qui ne sont pas des comptes de charge**
     (`account_type` ≠ `Expense`, `crates/kesh-db/src/entities/account.rs:16`) portent **un seul** compte en devise
     (« le compte en devise de cette écriture », celui qui donne la paire), dont le solde net (débit − crédit, en devise
     principale) est un **débit** — le côté opposé au compte de conversion, comme le compte USD de l'écriture 2 de
     l'assistant. **Frais (A33)** : l'écriture admet en plus des lignes **au débit** de comptes de charge de la **même
     devise** que ce compte (p. ex. 6911 *Frais PayPal USD*) ; elles ne comptent pas dans « un seul compte en devise »
     et entrent dans Σ débit **avant** le calcul de la différence — le frais reste sur son compte, jamais dans la
     différence de change (B3/B27 de l'Epic 38 : chaque sous-compte de frais se rapproche du relevé du prestataire).
     **Frais en CHF** (compte de charge sans devise) : ligne CHF ordinaire, hors de toute règle de forme (elle n'est pas
     sur un compte en devise) ; elle entre dans Σ débit avant la différence comme un frais en devise, au débit ou au
     crédit sans autre contrôle que ceux de 37-3a (l'assistant, lui, ne les offre qu'au débit) ; elle ne modifie ni
     l'identification de l'écriture à compléter ni sa paire.
     Quand la forme est examinée (écriture complétée, ou qui porte une ligne de différence), après le seul refus A31
     (champ et ligne portée) et avant tout autre contrôle de forme : comptes en devise (charges comprises) de **deux devises** → 400 `FX_MULTIPLE_CURRENCIES`
     (`details: { currencies }` du dépôt, 37-3a AC-4, 2 bis, plus `entry` posé par la route ; le code est celui du dépôt, anticipé à ce rang pour
     ne pas rendre un `ENTRY_UNBALANCED` trompeur) ; puis une ligne **au crédit** d'un compte de charge en devise →
     400 `Validation` (`details: { entry, line }`, message : « un frais de conversion se passe au débit d'un compte de
     charge de la devise <code> »). Le type et la devise des comptes se lisent par la requête de reconnaissance
     ci-dessous (mêmes comptes, même verrou), sans lecture de plus. L'autre écriture, et
     toute écriture à compléter qui n'a pas cette forme (aucun compte en devise hors comptes de charge, deux comptes en
     devise distincts de la même devise hors comptes de charge, ou
     compte en devise au crédit net — p. ex. `side: "credit"` saisi par erreur, que la ligne de différence aurait
     soldé par une perte égale à deux fois le CHF), ne sont jamais complétées : déséquilibrées, elles rendent
     `ENTRY_UNBALANCED` (`details.entry`), comme sans V3 (si les deux le sont, l'écriture qui n'est pas à compléter rend le refus : elle est validée la première, *ordre* ci-dessous) ;
     une écriture à compléter équilibrée sans ligne de différence n'y est pas soumise ; une écriture à compléter qui
     **porte** une ligne de différence sans avoir **un seul** compte en devise hors comptes de charge (paire indéterminée) → 400 `Validation`
     (`details: { entry, line }`, la ligne de différence), au rang du contrôle de la paire. **Limite écrite au guide d'API (AC-6)** :
     hors de ces cas, la ligne calculée **absorbe** tout écart du corps — montant, cours ou CHF faux sur la ligne en
     devise de l'écriture à compléter, frais oubliés, CHF du compte de conversion faux aux deux écritures — et le passe
     en gain ou en perte de change ; le client relit la ligne dans la réponse 201 (D6 : l'écran la montre à l'aperçu).
     Le consentement explicite du corps (`completeDifference`) est **retenu** (A31) : une ligne calculée n'absorbe
     jamais en silence l'erreur d'un client qui ne l'a pas demandée ; une borne de plausibilité n'est pas retenue ;
   - **paire** : `fx_accounts_for(société, compte en devise de cette écriture)` (37-1 AC-4 bis), **puis**
     `validate_fx_accounts_in_tx` sous les verrous (37-1 : `FX_ACCOUNT_INVALID`, `field`, `reason` de la liste close,
     `holderAccountId` si `origin = Own` ; `None` → `reason: "missing"`), le compte en devise relu `LOCK IN SHARE MODE`
     avec les autres comptes de l'écriture — jamais un choix écrit à côté entre `companies.fx_*` et `accounts.fx_*` ;
     la paire n'est résolue et contrôlée qu'**à l'usage** (37-1) : quand une ligne est à ajouter ou que l'écriture à
     compléter en porte une ; une écriture à compléter équilibrée sans ligne de différence ne lit aucune paire et
     n'est refusée par aucun `FX_ACCOUNT_INVALID` de paire (l'assistant, lui, désactive son bouton sur des comptes
     résolus invalides même à différence nulle : plus strict, sans contradiction) ;
   - **ligne de différence reconnue à son compte** : un compte est « désigné » s'il est le compte de gains ou de pertes
     de la société (`companies.fx_*`) ou d'un compte de la société (`accounts.fx_gain_account_id` /
     `fx_loss_account_id`, 37-1 AC-4 bis) ; la reconnaissance se lit **dans la transaction, après le verrou de
     `companies`** (ordre de 37-1 AC-3), par une requête bornée aux comptes des lignes **de l'écriture à compléter**
     (`EXISTS` sur `accounts` pour chaque identifiant de ligne, `LOCK IN SHARE MODE` ; elle rend aussi `account_type`
     et `currency` de ces comptes, que lit la forme, A33) — jamais un balayage de tous les
     comptes, ni une lecture avant les verrous ; une ligne de l'autre écriture sur un compte désigné est une ligne CHF
     ordinaire (contrôle central de 37-3a, rien de plus) ;
   - **corps avec `completeDifference: true` et sans ligne de différence** (aucune ligne **de l'écriture à compléter** sur un compte désigné comme compte
     de gains ou de pertes, de la société ou d'un compte en devise) : le serveur **ajoute en dernière ligne** de l'écriture à compléter une ligne
     CHF de |Σ débit − Σ crédit| (Σ débit **frais A33 compris** : la différence se calcule après eux) — **au crédit du compte de gains** si les débits l'emportent, **au débit du compte de
     pertes** sinon (la règle de l'assistant ci-dessus) ; si l'écriture compte déjà `MAX_LINES_PER_ENTRY` lignes, 400
     `Validation` (« trop de lignes », `details.entry`) — la limite de `prepare_new_journal_entry` se contrôle sur le
     corps **complété**, `accounting::validate` ne la portant pas ; elle figure dans la réponse 201 et dans l'audit
     `journal_entry.created` comme toute ligne ; son rang est le dernier (`details.line` d'un refus qui la viserait) ;
   - **corps qui porte une ligne de différence, sans `completeDifference`** (ligne sur un compte désigné comme compte de gains ou de pertes) : elle
     doit employer la **paire résolue** — le compte de gains au crédit, ou le compte de pertes au débit —, sinon 400
     `Validation` (`details: { entry, line }`, message : « la différence de change va sur les comptes du compte en
     devise <numéro> »), au rang du contrôle de la paire (ci-dessous) ; l'équilibre de l'écriture se juge ensuite comme d'ordinaire ;
     l'assistant envoie toujours la sienne (calculée sur la même paire par `fxAccountsFor`, aperçu D6) ;
   - **ordre** : **avant la transaction**, le handler applique, écriture par écriture dans l'ordre du corps, la longueur du libellé et la limite
     `MAX_LINES_PER_ENTRY` du corps reçu (l'ordre de `prepare_new_journal_entry`, `journal_entries.rs:525-537`), puis
     à chaque ligne l'analyse de 37-3a AC-6 (V1, V2), qui prolonge le parse des montants (`:540-554`), et le contrôle FX par ligne de 37-3a AC-3 (fonction publique), **sans**
     `accounting::validate` — `prepare_new_journal_entry` (`journal_entries.rs:515`, qui valide l'équilibre `:566`)
     n'est donc pas appelée telle quelle : sa partie sans `validate` est extraite et partagée (DRY), car savoir quelle
     écriture reçoit une ligne exige l'identifiant du compte de conversion et la devise des comptes, lus **sous
     verrou** (P2, CR-6 : choisir avant la transaction l'écriture « qui ne reçoit pas de ligne » est irréalisable).
     **Dans la transaction**, après `FX_CONVERSION_TRANSIT_UNBALANCED` : identification de l'écriture à compléter (ci-dessus),
     puis `accounting::validate` de l'**autre** écriture (`ENTRY_UNBALANCED`, `details.entry`), puis forme et paire de
     l'écriture à compléter, ajout de la ligne (refus « trop de lignes », un `AppError::Validation` du handler, non un
     refus de `kesh-core`), et `accounting::validate` de l'écriture à compléter **après** l'ajout — son refus éventuel
     (aucun quand une ligne a été ajoutée : elle équilibre par construction ; `ENTRY_UNBALANCED` quand l'écriture n'a
     pas la forme exigée ou porte une ligne de différence qui ne l'équilibre pas) rend un 400 par `map_core_error`,
     jamais le `DbError::Invariant` d'un 500 ; tout cela avant les deux `create_in_tx`.
   **Verrous et ordre des refus** (une seule raison) : `entries` de longueur ≠ 2 (avant la transaction) ; puis, avant la
   transaction, pour chaque écriture dans l'ordre du corps (`details.entry`) : longueur du libellé, limite
   `MAX_LINES_PER_ENTRY` du corps reçu, analyse V1/V2 (37-3a AC-6), contrôle FX par ligne (37-3a AC-3) ; puis, dans
   la transaction et **avant** le premier `create_in_tx` : la ligne `companies` (`fx_transit_account_id`) lue
   `LOCK IN SHARE MODE` (jamais `FOR SHARE`), puis le compte de conversion lu `LOCK IN SHARE MODE` — ordre `companies` →
   réglages → `accounts` de 37-1 AC-3, comme la réévaluation (37-5 AC-2, étapes (0) et (3)) ; `create_in_tx_inner`
   ne prend plus de verrou de devise sur `companies` (37-3a AC-4, D-CR7-1), mais son étape 0 prend toujours la
   sentinelle `companies` **`FOR UPDATE`** quand une ligne porte un projet (`projects::validate_taggable_in_tx`,
   `projects.rs:101`, `journal_entries.rs:308`) : la transaction de l'assistant, qui tient déjà le partagé, monte alors
   de S à X sur la même ligne, et deux conversions concurrentes de la même société portant chacune un projet
   s'interbloquent — cycle défendu par le rejeu (`retry_on_deadlock`, route `Rejouee`), argument écrit au Change Log
   avec celui des comptes ci-dessous → refus
   `FX_ACCOUNT_INVALID` ; → `FX_CONVERSION_TRANSIT_UNBALANCED` (calculé sur les corps, identifiant relu sous verrou) ;
   → équilibre de l'écriture qui n'est pas à compléter (`ENTRY_UNBALANCED`, `details.entry` — avec ou sans
   `completeDifference`, de sorte que deux écritures déséquilibrées rendent toujours le refus de celle-ci, jamais
   « la première exécutée ») ; → forme et paire de
   l'écriture à compléter (V3 : `completeDifference: true` avec une ligne de différence portée → 400 `Validation`, A31 ;
   comptes en devise de deux devises → `FX_MULTIPLE_CURRENCIES`, A33 ; frais au crédit d'un compte de charge en devise
   → 400 `Validation`, A33 ; ligne portée sans compte en devise unique hors comptes de charge → 400 `Validation` ; `FX_ACCOUNT_INVALID` de
   `validate_fx_accounts_in_tx`, puis ligne portée hors de la paire → 400 `Validation`, puis « trop de lignes », puis
   ajout de la ligne, puis équilibre de l'écriture complétée, `ENTRY_UNBALANCED`) ; → les deux écritures dans l'ordre d'exécution ci-après, chronologique (refus de chacune dans l'ordre de 37-3a AC-4, avec `details.entry` = son rang dans le corps). Les deux
   `create_in_tx` s'exécutent dans l'ordre **chronologique** de leurs dates (corps dans l'ordre à date égale), pour
   prendre les exercices `FOR UPDATE` dans l'ordre de `close` (antérieurs d'abord) ; la réponse garde
   l'ordre du corps et `details.entry` le rang dans le corps. Le compte de conversion, les comptes des lignes de
   l'écriture à compléter (V3) et ses comptes de différence, lus en partage avant l'exercice (l'écriture ordinaire prend
   l'exercice puis les comptes en partage), ne forment pas de cycle connu : aucun chemin ne prend un exercice puis un
   compte **en exclusif** depuis le retrait de la bascule `force` de 37-1 (CR-7) — argument écrit au Change Log,
   défendu de toute façon par le rejeu (`retry_on_deadlock`, route `Rejouee`). Réponse 201 `{ entries: [JournalEntryResponse, JournalEntryResponse] }` ; l'audit
   est celui des deux créations (`journal_entry.created` ×2), les libellés se renvoient (« (1/2) », « (2/2) »).
   Registre des routes : `LIB_ROUTES` +1, `Traced, Rejouee` (elle écrit au journal comptable ; enveloppe exigée par
   `every_replayed_route_calls_an_envelope`) — **+1 route, +1 tracée, total +1, `rejouees` +1, `tout.len()` +1**, en
   deltas cumulés avec 37-1, 37-2 et 37-5 (recomptés depuis la source après chaque reprise de `main`) ; l'assertion `examinees` de `every_replayed_route_calls_an_envelope`
   (`audit_route_registry.rs:1325`, 27) prend +1, chiffre et toutes lettres (alignement de l'orchestrateur sur
   37-5-CR2-P2-IF2) ; phrase du manuel d'administration recomptée.
   **Tests** : dépôt et API — conversion sans frais (gain), avec frais aux deux écritures (perte), différence nulle
   (aucune ligne d'écart), compte de conversion à 0 après, refus de la seconde écriture **exécutée** (la plus tardive, placée en tête du corps pour exercer l'ordre chronologique) → rien de créé, `details.entry` = son rang dans le corps, compte de
   conversion non désigné ou invalide, transit déséquilibré, date dans un exercice mono-devise (fabriqué comme à l'AC-1 : clos, activation, réouverture), `details.entry`,
   `entries` de longueur 0, 1 ou 3, rôle Consultation → 403, clé d'API en lecture seule refusée ; **V3 (CR-6)** :
   corps sans ligne de différence **avec `completeDifference: true`** → ligne ajoutée en dernière ligne de l'écriture à compléter (compte de conversion au crédit net), au crédit du compte de
   gains (gain) ou au débit du compte de pertes (perte), relue dans la réponse et à l'audit, compte de conversion à 0 ;
   compte USD à comptes propres → ligne sur ses comptes, compte « par défaut » → comptes de la société ; corps sans
   ligne de différence et différence nulle → aucune ligne ; **A31** : même corps déséquilibré **sans** le champ (ou à
   `false`) → `ENTRY_UNBALANCED` (`details.entry`), rien de créé ; sans le champ, une ligne de différence portée hors
   de la paire → 400 `Validation` (le contrôle ne dépend pas du champ), portée sur la paire → 201 ; `completeDifference:
   true` sur des écritures équilibrées sans ligne de différence → 201, aucune ligne ajoutée ; champ non booléen → 400 ; `completeDifference: true` **avec** une ligne de
   différence déjà présente → 400 `Validation` (`details: { entry, line }`) ; ligne de différence portée sur la paire résolue → acceptée
   telle quelle ; portée sur le compte de gains de la société alors que le compte USD a ses comptes propres → 400
   `Validation` (`details.entry`, `details.line`) ; gain porté au débit → 400 (fixture à comptes de gains et de pertes **distincts** : 37-1 admet le même compte, qui rendrait ce cas indécidable) ; comptes de différence non désignés ou
   compte propre archivé (par SQL) → `FX_ACCOUNT_INVALID` (`holderAccountId`), rien de créé ; deux écritures
   déséquilibrées, l'écriture à compléter sans compte en devise → `ENTRY_UNBALANCED` de l'écriture qui n'est pas à compléter, même placée en second dans le corps et datée après ; écriture 1 déséquilibrée (erreur du client) →
   `ENTRY_UNBALANCED`, jamais complétée ; écriture à compléter sans compte en devise → `ENTRY_UNBALANCED` ; compte en devise de l'écriture à compléter au crédit net (`side: "credit"` par erreur) → `ENTRY_UNBALANCED`, aucune ligne ajoutée ; deux comptes en devise distincts de la même devise, hors comptes de charge, dans l'écriture à compléter → `ENTRY_UNBALANCED` ; même cas avec une ligne de différence portée → 400 `Validation` ; écriture à compléter équilibrée et comptes de différence invalides → 201 (paire lue à l'usage seulement) ; écriture à
   compléter à `MAX_LINES_PER_ENTRY` lignes → 400 `Validation` ; écritures passées sans `rate` ni `debit`/`credit`
   (V1, V2 : `{ amount, chf, side }`) → 201 ; **A33** (fixture : compte de charge USD 6911 *Frais PayPal USD*, compte
   de charge EUR) : écriture à compléter avec un frais au débit de 6911 USD et `completeDifference: true` → 201, frais
   sur 6911 tel quel, différence = CHF du compte de conversion − (CHF₂ + CHF du frais), relue dans la réponse (le
   frais n'est pas dans la ligne de différence) ; même corps portant la ligne de différence de l'assistant sans le
   champ → 201 ; frais sur 6911 USD **au crédit** → 400 `Validation` (`details: { entry, line }` = la ligne du frais),
   aucune ligne ajoutée, rien de créé ; frais sur le compte de charge **EUR** dans l'écriture USD → 400
   `FX_MULTIPLE_CURRENCIES` (`details: { entry, currencies: ["EUR", "USD"] }`), rien de créé — avec le champ et
   avec une ligne de différence portée sans le champ ; écriture à compléter dont le seul compte en devise est 6911
   (aucun compte en devise hors charges) → `ENTRY_UNBALANCED` ; frais en CHF (compte de charge CHF, au débit) avec un
   frais sur 6911 USD → 201, les deux frais tels quels, différence calculée après eux, l'écriture n'étant ni refusée
   ni re-déterminée par la ligne CHF ;
   Vitest de l'assistant (calcul de la différence sur la fixture partagée, frais, cours OU CHF, bouton désactivé hors
   exercice ouvert, et désactivé avec le motif de la paire indéterminée pour une ligne de frais de l'écriture 2 sur un second compte en devise qui n'est pas un compte de charge de la devise, avec une différence non nulle (AC-2 bis) ; frais de l'écriture 2 sur 6911 USD (compte de charge) → bouton actif, différence calculée après le frais, aperçu avec la ligne du frais (A33) ; compte USD à comptes propres → ligne de différence sur ces comptes, compte USD « par
   défaut » → comptes de la société, S2) ; E2E (AC-5). L'écriture 2 reste modifiable après coup comme toute écriture (le lien entre
   les deux n'est pas gardé en base : limite écrite au manuel).
3. **AC-3 — Affichage et édition** : détail et liste des écritures montrent montant en devise (avec son code), cours
   **sans zéros terminaux** et source (`AFC`, `manuel`, `réévaluation`) à côté du CHF ; `lineResponseToDraft` recharge
   `foreign`. Une écriture portant une ligne `revaluation` **n'est pas modifiable** : le champ existant
   `modificationBlockedBy` (`journal-entries.types.ts:84`, union `ModificationBlocker` :117 ; serveur
   `ModificationBlocker`/`ModificationGuard`, `kesh-db/src/errors.rs:134,210`) reçoit la **valeur** neuve
   `ENTRY_IS_FX_REVALUATION`, rendue par le serveur (37-3a AC-7) ; 37-3b l'ajoute à l'union TS et à
   `modificationBlockerLabel` (quatre locales ; `blocker-messages.test.ts`, « couvre exactement douze codes »
   `:51-54`, passe à treize), et la fiche affiche le motif. Les types TS `JournalEntryLineResponse` et
   `CreateJournalEntryLineRequest` (`journal-entries.types.ts`) gagnent `foreign` (T3 : `currency: string` dans la
   réponse, lue sur le compte ; `currency?: string` dans la requête, AC-2). Le code affiché à côté du montant est celui
   de `foreign.currency` de la réponse (la devise du compte), jamais une valeur gardée par l'écran. « Modifier » disparaît ; **« Supprimer »
   reste** pour ce motif — la suppression et la contre-passation sont les corrections prévues (37-5 AC-3) — alors
   qu'aujourd'hui les deux boutons partagent la condition `entry.modifiable` (`journal-entries/[id]/+page.svelte`
   :418-432) : la condition de « Supprimer » devient `modifiable || (modificationBlockedBy === 'ENTRY_IS_FX_REVALUATION' &&
   !periodLocked && !lettered)`. ⚠️ Les deux termes sont nécessaires : le motif `ENTRY_IS_FX_REVALUATION` se rend
   **avant** `PERIOD_LOCKED` et la marque de lettrage (37-3a AC-7 ; `modification_blocker`, `journal_entries.rs:1243`),
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
   - Inventaire **clos** au Change Log, à partir de l'inventaire serveur de 37-3a et du grep des composants qui
     importent `fetchAccounts`/`AccountAutocomplete` : au moins `InvoiceForm`, `invoices/[id]`, `invoices/due-dates`,
     `credit-notes/[id]`, `SettleInvoiceDialog`, `SupplierInvoiceForm` et les pages fournisseur
     (`new`, `[id]`, `[id]/edit`), `settings/invoicing`, `products/+page.svelte` (:697), `ManualMatchModal`,
     `TransactionSplitModal`, `ReconciliationProposals`, `reconciliation/rules` (`RuleFormModal`),
     `bank-accounts/+page.svelte` (`<select>` :860, :1116) et `BankAccountJournalLinkForm` (:67) — tous trois par
     `linkableLedgerAccounts` (`features/bank-accounts/format.ts:180`), où le filtre se pose une fois —,
     `VatPurchaseAssistant` (:159, :204 — reçoit les comptes **non filtrés** du formulaire,
     `JournalEntryForm.svelte:350-355` ; **S5, CR-3** : dans une société multi-devises, ses deux `AccountAutocomplete`
     passent `excludeForeignCurrency = false` et le composant restreint lui-même à CHF plus **une** devise (AC-2,
     assistant TVA achat en devise) ; en mono-devise, le défaut `true` reste — ne pas filtrer dans le
     formulaire, qui a besoin des comptes en devise ; exemption de la garde d'inventaire justifiée par ce motif).
   - **À ne pas filtrer** (libellés, consultation, ou saisie en devise) : `reports`, `OpenItemsScreen`,
     `accounts/+page.svelte`, `journal-entries/[id]`, `journal-entries/+page.svelte` (liste, et comptes passés au
     formulaire), `RulesList`, et **`settings/opening-balances/+page.svelte`** (`fetchAccounts` :8, :77 : les comptes en
     devise y figurent, exception C-37-5), et **l'assistant *Conversion*** (AC-2 bis : composant neuf, exemption
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
5. **AC-5 — E2E** : écriture USD au cours AFC (cours servis par la source factice du mode test de 37-2 AC-3, pilotée
   par `/api/v1/_test/exchange-rates/source`), écriture USD avec ligne d'écart ajoutée par le bouton (R8) — une fois
   sur un compte « par défaut » (comptes de la société), une fois sur un compte USD à **comptes propres** (S2 : la
   ligne va sur ses comptes) —, **achat USD avec TVA 8,1 % par l'assistant TVA** (S5 : ligne d'impôt préalable en CHF
   au cours de l'achat, écart d'un centime complété par le bouton d'écart, écriture enregistrée), conversion
   EUR → USD **par l'assistant** (deux écritures, compte de conversion à 0 après, R7), écriture à deux devises refusée
   (`FX_MULTIPLE_CURRENCIES`) **par l'API** (`authedApiContext`) — le formulaire ne la laisse pas composer (R7) ; à
   l'écran, on vérifie que le sélecteur ne propose pas le compte de l'autre devise, saisie du CHF au
   lieu du cours, soldes d'ouverture d'un compte USD au cours journalier de la veille de l'ouverture (genre `day`, R6), puis, l'exercice passé en genre `month` par `PUT /fiscal-years/{id}/rate-kinds` (37-2 ; les deux champs `openingRateKind` et `closingRateKind` sont obligatoires : le second reste inchangé ; `applyToNextOpening` absent, sans exercice suivant) et la page rechargée, au cours mensuel du dernier mois de l'exercice précédent (T4 ; l'exercice du seed `with-company` s'ouvre le 01.01.2020, `crates/kesh-db/src/test_fixtures.rs:118-119` : veille 31.12.2019, mois décembre 2019 ; ce jour et ce mois servis par la source factice) — les deux cours se vérifient **au préremplissage**, l'ouverture ne s'enregistre qu'après le passage en `month` : enregistrée en genre `day`, elle ferait passer la page au complément, dont le cours est celui du mois de sa propre date (AC-1) (l'E2E repart d'une société vierge par `seedTestState('with-company')`,
   précédent `opening-balances.spec.ts:21, :80` : l'ouverture exige une société sans écriture, `ALREADY_HAS_ENTRIES`),
   mono-devise inchangé ; **cours saisi par la société** (D9) : un cours posé par `PUT /exchange-rates/rates` (37-2,
   `source: "manual"` ; si le mois a déjà été récupéré, une ligne `afc` copiée existe et le `PUT` doit porter sa `version`,
   lue par `GET /exchange-rates`, sinon 409) pour le mois est proposé, affiché « cours saisi », et la ligne part `rateSource: "manual"`
   (relue telle) ; après ↑ à l'écran *Cours de change* (`POST /exchange-rates/rates/reset`, avec la `version` de la ligne et le `publishedRate` affiché ; le mois doit avoir une
   valeur publiée, servie par la source factice et récupérée, sans quoi une ligne `manual` serait **supprimée** et non
   reprise ; la ligne de la société repasse `afc`), le cours proposé part `afc` ; **cours `manual` apparu après l'écriture** : édition du montant
   seul d'une ligne `afc` relue, le mois étant passé en `manual` par `PUT …/rates` → `FX_RATE_NOT_AFC`, `expectedRate`
   `null`, seule « Garder mon cours » offerte ; après elle, la ligne part `manual` à son cours et l'écriture s'enregistre ; **changement de compte (U2)** : une ligne USD passée sur un compte EUR
   repart montant vide et cours EUR proposé, l'écriture s'enregistre en EUR ;
   préalables posés par l'API (Administrateur) : mode multi-devises activé **avec les comptes de gain, de perte et de
      conversion désignés** (37-1 AC-3, sans quoi le bouton d'écart n'ajoute rien et l'assistant est désactivé),
      exercices passés en multi-devises par l'activation ; **S2/S5** : un compte USD à comptes propres (gains et pertes
      désignés par l'API à sa création) en plus d'un compte USD « par défaut », et, pour l'assistant TVA, le compte
      d'impôt préalable des réglages de facturation et un taux de TVA 8,1 % (sans quoi l'assistant est désactivé :
      `configMissing` sans compte d'impôt préalable, `noRates` sans taux, `VatPurchaseAssistant.svelte:55-56`) ; *(CR-7 : le scénario de l'exercice mono-devise est retiré ; le refus
      `MULTI_CURRENCY_DISABLED` d'un exercice mono-devise est couvert par Vitest et par les tests d'API de 37-3a)* ;
      compte USD (et EUR) créé, cours des mois et jours utiles servis par la source factice — dont la **veille** de
      l'ouverture ; un scénario au moins part
      d'une table des cours vide et exerce le repli de l'écran (`404 RATE_UNAVAILABLE` → `POST /exchange-rates/fetch`,
      qui copie le cours publié dans les cours de la société, 37-2 AC-3 bis → relecture), les autres récupèrent leurs
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
   exercice clos avant l'activation : aucun compte en devise n'y est utilisable, même rouvert (CR-7), soldes d'ouverture en devise (« solde repris au cours de clôture » : cours journalier de
   la veille de l'ouverture, ou cours mensuel du dernier mois de l'exercice précédent, selon le choix de l'exercice,
   R6 et T4 : le cours d'ouverture est le cours de clôture précédent) ; i18n quatre
   locales (catalogues : bloc propre à l'epic, compteurs `sitesTotal` recomptés depuis la source après chaque reprise de `main`) ; `.tex` seulement, PDF non commité (régénéré à la release). ⚠️ **Garde G4-bis**
   (`crates/kesh-api/tests/textes_coherents.rs`) : tout numéro de compte cité dans le manuel, le guide de démarrage ou
   un catalogue i18n doit exister dans l'un des trois plans livrés, sous son nom — or ni 1090, ni 1021, ni 1022, ni
   6840 n'y figurent (la maquette de l'AC-2 bis est une illustration de la fiche, pas un texte à recopier) : les exemples du
   manuel et des catalogues n'emploient que des numéros livrés (p. ex. 1020 *Banque*, 6940 *Différences d'arrondi*) ou
   parlent de « votre compte de conversion » sans numéro ; contrôle : `cargo test -p kesh-api --test textes_coherents`.
   **Guide d'API** (`docs/api-external.md`) : la ligne « Écritures comptables » de son tableau de routes (l'assistant,
   `POST /journal-entries/conversions`, ouvert aux clés d'API comme `POST /journal-entries`) et le paragraphe de
   `LATER_FISCAL_YEAR_CLOSED` (toute création d'écriture, la conversion comprise) ; les champs `foreign` sont ceux de
   37-3a — le guide dit que la devise d'une ligne est **celle de son compte** (T3), que `foreign.currency` est
   facultatif à l'envoi et, s'il est donné, doit égaler la devise du compte (`FX_CURRENCY_MISMATCH`), et qu'un client
   qui l'omet renonce à cette garde ; une ligne en devise sur un compte CHF rend `FX_ACCOUNT_IN_CHF`. **CR-6 (couverture
   de l'API)** — le guide dit aussi : une ligne en devise se passe **sans calcul du client** (37-3a AC-6) : `rate` ou
   `chf` (cours déduit par le serveur, V1 ; A32, ligne sans `rate` et sans `rateSource` ou à `rateSource: "afc"` : si
   le CHF est celui que donne le cours `afc` retenu du mois, la ligne prend ce cours et `afc` ; sinon, sans
   `rateSource`, le cours déduit et `manual`, à `rateSource: "afc"`, `FX_RATE_NOT_AFC` ou `RATE_UNAVAILABLE` ; à
   `rateSource: "manual"`, toujours le cours déduit et `manual` — 37-3a AC-4, 4), `debit`/`credit` ou `side` (CHF posé par le serveur, V2), `rateSource`
   facultatif (qualifié `afc` ou `manual` par le serveur, V1) — mêmes champs aux routes d'ouverture et de complément
   (AC-1) ; une ligne CHF sans `debit` rend désormais 400 et non 422 (route des écritures et génération des soldes d'ouverture ; le complément, lui, tient toujours une colonne absente pour 0) ; la ligne d'écart (comptes du compte en devise,
   sinon de la société, lus sur `AccountResponse` et `CompanyJson` ; V4) ; la conversion sans ligne de différence,
   calculée par le serveur **sur demande** (AC-2 bis, V3, A31 : corps `completeDifference: true`, sans lui `ENTRY_UNBALANCED` ; avec lui et une ligne de différence déjà présente, 400 `Validation` ; seule l'écriture où le compte de conversion est au **crédit** net est
   complétée, et seulement si elle porte un seul compte en devise hors comptes de charge, au débit net ; des frais au
   débit de comptes de charge de la même devise y sont admis et comptés avant la différence, jamais dans elle — un
   frais au crédit rend 400 `Validation`, un compte d'une autre devise `FX_MULTIPLE_CURRENCIES` (A33) ; le CHF du compte de conversion est
   donc celui de l'autre écriture, qui doit s'équilibrer seule ; la ligne calculée absorbe tout écart de l'écriture
   complétée — montant, cours, frais oubliés — en gain ou perte de change : la relire dans la réponse 201) ; l'achat avec TVA en devise, calcul du client selon la règle d'arrondi de
   l'AC-2 (V5) ; le cours d'ouverture proposé, case `opening` de `GET /fiscal-years/{id}/exchange-rates` (V9) ; la
   réévaluation de livres importés, repassée par `POST /fiscal-years/{id}/fx-revaluation` avec `rates` (37-5, V8) ;
   l'activation du multi-devises par une session d'Administrateur en `Authorization: Bearer`, jamais par une clé
   d'API (37-1 AC-3, V6). Contrôle du guide contre les routes au développement (Change Log).

## Tâches

- [x] Soldes d'ouverture : serveur (DTO, `ComplementLine`, statut), écran, totaux, tests (AC-1).
- [x] `fx.ts` (calcul CHF, cours depuis CHF), formulaire, `balance.ts`, `form-helpers.ts`, détail, liste, Vitest sur la
      fixture partagée (AC-2, AC-3).
- [x] Sélecteurs, prédicat commun, garde d'inventaire (AC-4).
- [x] E2E, captures, manuel (`.tex` seulement), i18n, section *Note de version* de la fiche (AC-5, AC-6).
- [x] CR-3 : ligne d'écart et assistant *Conversion* sur les comptes de différence du compte en devise (S2,
      `fxAccountsFor`) ; assistant TVA achat en devise (S5 : `vat-purchase.ts`, `chfFromForeign`, ligne CHF à zéro,
      changement de date) ; `FX_RATE_NOT_AFC` après la reprise par ↑ d'un cours corrigé (ex-« adoption », U1), pour
      les seules lignes nouvelles ou modifiées (A17, « Garder mon cours ») ; E2E et manuel complétés.
- [x] CR-5 : U1 — `lookup` à `effectiveRate` + `source` (+ `publishedRate` non affiché) : type TS, `afcRate` dérivé
      (`source === 'afc' ? effectiveRate : null`), indicateurs « AFC » / « cours saisi » sur `source`, ↑ sous le champ
      vers le cours de la société, soldes d'ouverture (provenance affichée) ; U2 — changement de compte d'une ligne
      (USD → EUR : montant et cours réinitialisés ; vers CHF : `foreign` retiré ; CHF → devise ; même devise) au
      formulaire, Vitest et E2E ; manuel (AC-6).
- [x] CR5-P1 : `FX_RATE_NOT_AFC` à `expectedRate` `null` (action « Reprendre » absente, Vitest) ; manuel sans « moyenne » ;
      préalables E2E du cours saisi et du ↑ (`version`, valeur publiée) ; genre `month` par `PUT /rate-kinds` complet.
- [x] CR-4 : T3 — `currency` facultatif des DTO d'ouverture, types TS (réponse `currency: string`, requête
      `currency?`), devise du compte dans `LineDraft`, envoi systématique par formulaire, grilles et assistants,
      messages `FX_CURRENCY_MISMATCH` / `FX_ACCOUNT_IN_CHF`, guide d'API ; T4 — cours d'ouverture de genre `month` au
      mois de la veille de l'ouverture (fonction pure TS, vecteurs dont le 15.03), tests Vitest et E2E, manuel.
- [x] CR-6 : **V1** — `rateFromChf` dans `fx.ts`, jumeau de `rate_from_chf` (37-3a AC-3), seul calcul « cours OU
      CHF » de l'écran (formulaire, deux grilles d'ouverture, assistant *Conversion*) ; Vitest sur la fixture partagée
      `crates/kesh-core/tests/fixtures/fx_rate_from_chf.json` (accepts et `error`, aucune copie) ; DTO d'ouverture
      (`OpeningBalanceLineRequest.debit`/`credit` en `Option<String>`) et complément passés par la fonction d'analyse
      partagée de 37-3a AC-6 (**V2**) ; tests d'API sur les deux routes : ligne `{ amount, chf, side }` sans `rate`, ni
      `debit`/`credit`, ni `rateSource` → 201, cours de la fixture, provenance `manual` (date d'ouverture hors mois à
      ligne `afc`) ; refus sur les deux routes : `debit`/`credit` absents sans `side` → 400, `chf` ≠ `debit` + `credit`
      → `FX_AMOUNT_MISMATCH`, ni `rate` ni `chf` → 400, ligne CHF d'ouverture sans `credit` → 400 (non 422) ; complément
      de même, avant la règle « exactement une colonne non nulle » ; **V3** — route de
      conversion : ligne de différence calculée **sur demande** (`completeDifference`, A31 ; sans le champ
      `ENTRY_UNBALANCED`, champ + ligne présente → 400) ou contrôlée (AC-2 bis), frais de l'écriture à compléter sur un
      compte de charge de la devise (A33 : hors « un seul compte en devise », avant la différence ; crédit → 400
      `Validation`, autre devise → `FX_MULTIPLE_CURRENCIES` ; garde de l'assistant restreinte), absence de cycle écrite au Change Log (CR-7),
      tests de l'AC-2 bis ; **V5** — limite et règle d'arrondi de l'assistant TVA (AC-2), au guide d'API ; **V9** —
      renvoi à la case `opening` (AC-1), au guide d'API ; guide d'API (AC-6) complété (V1–V9) ; **A32 (IF1, IF2)** — tests
      d'API, routes de génération et de complément : ligne `{ amount, chf, side }` sans `rateSource` dans un mois retenu
      `afc` dont le cours redonne le CHF → provenance `afc`, cours retenu (37-3a AC-4, 4, *Cours déduit*) ; sinon
      `manual` ; `None` jamais écrit ; Vitest du CHF saisi à la main (AC-2).
- [x] CR-2 : cours d'ouverture selon le genre de l'exercice (AC-1, R6) ; refus `MULTI_CURRENCY_DISABLED` d'un
      exercice mono-devise affiché à l'ouverture et au formulaire (AC-1, AC-2, CR-7) ; bouton d'écart qui ajoute une ligne (R8) ; une devise par écriture au formulaire
      (R7) ; assistant *Conversion* — route `POST /journal-entries/conversions`, code `FX_CONVERSION_TRANSIT_UNBALANCED`,
      registre des routes (+1), écran et maquette (AC-2 bis).

## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release ; la fiche n'écrit jamais au `CHANGELOG.md`.)*

- **Changed (contrat de l'API)** — routes d'ouverture : à la génération, une ligne CHF dont `debit` ou `credit` est absent rend 400 `Validation` au lieu de 422 ; `rateSource: "revaluation"` rend 400 sur les deux routes (guide d'API `docs/api-external.md` à jour).
- **Added** — écrans de saisie en devise, assistant *Conversion*, ligne d'écart sur les comptes de différence de change.
- **Added (API)** — `POST /journal-entries/conversions` : deux écritures en une transaction par le compte de conversion, `completeDifference` optionnel (ligne de différence calculée), rejouée sur interblocage ; nouveau code `FX_CONVERSION_TRANSIT_UNBALANCED` (400) ; deux écritures dans la même devise, CHF comprise, refusées en `VALIDATION_ERROR`.
- **Added (API)** — `GET /opening-balances/status` : `currency` sur les comptes complétables ; `foreign` admis sur les lignes des deux routes d'ouverture.
- **Compteurs** : recomptés depuis la source après chaque reprise de `main` (`LIB_ROUTES`, partitions, `sitesTotal`, catalogues i18n).

## Notes de développement

- `frontend/src/lib/features/journal-entries/` : `JournalEntryForm.svelte` (575 lignes), `balance.ts` (`parseAmount`
  :68, `computeBalance` :83, `formatSwissAmount` :129), `form-helpers.ts`, `vat-purchase.ts`,
  `journal-entries.types.ts`.
- Écrans des soldes d'ouverture : `frontend/src/routes/(app)/settings/opening-balances/` ;
  `frontend/src/lib/features/opening-balances/` (`opening-balances-totals.ts`, types).
- `kesh-db` : `repositories/opening_complement.rs` (`COMPLETABLE_SQL` :319, `create_opening_complement_in_tx` :551,
  `check_lines`), `routes/bank_accounts.rs` :764.
- Dépendances sur d'autres fiches (portées ailleurs) : 37-1 (devise refusée sur un compte lié, désigné par défaut ou
  portant un rôle ; lier/désigner un compte en devise refusé) ; 37-3a (produit en entiers `i128`, jamais `Decimal::checked_mul`, `FX_VALUE_OUT_OF_RANGE`,
  fixture `fx_rounding.json`, valeur `ENTRY_IS_FX_REVALUATION` de `modificationBlockedBy` rendue par le serveur,
  contrôle FX par ligne exposé en fonction publique de `kesh-core` ; CR-6 : `rate_from_chf` et fixture
  `fx_rate_from_chf.json`, fonction d'analyse de `foreign` partagée, qualification d'une ligne sans `rateSource`) ;
  C-37-2 (3) amendé.

## Change Log

- 2026-11-14 — fiche rédigée (découpe de 37-3).
- 2026-11-14 — validation P1 (Sonnet) : 0 CRITICAL, 3 HIGH, 8 MEDIUM, 3 LOW ; remédiés ; P1-4 (37-1), P1-7 (37-3a) et
  l'amendement de C-37-2 portés par l'orchestrateur.
- 2026-11-14 — validation P2 (Opus) : 0 CRITICAL, 3 HIGH, 6 MEDIUM, 3 LOW ; remédiés dans la fiche sauf
  37-3b-P2-4 et 37-3b-P2-2 pour leur part dans 37-3a (inter-fiches, portés par l'orchestrateur).
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
  le **cours effectif** de 37-2 (`effectiveRate` de `lookup`, cours saisi par la société sinon cours mensuel AFC) ;
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
  écritures par le compte de conversion de 37-1, la différence de change dans la seconde pour que le compte revienne à
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
  37-3b-CR2-P1-2 (pour sa part dans 37-3a) et 37-3b-CR2-P1-12 (registre), inter-fiches. MEDIUM : P1-1 préalables E2E
  contradictoires (exercice mono-devise postérieur → `laterFiscalYear` ; bascule refusée avec soldes en devise) ;
  P1-2 `PUT` d'une ouverture à deux devises refusé par `FX_MULTIPLE_CURRENCIES` (exemption à la seule création) ;
  P1-3 « mois ≤ mois courant » résiduel, contraire à R4 (dernier mois publié) ; P1-4 garde G4-bis : 1090, 1021, 1022 et
  6840 absents des plans livrés (manuel, catalogues) ; P1-5 assistant *Conversion* absent de l'inventaire et de
  l'exemption de la garde des sélecteurs. LOW : P1-6 E2E « deux devises » à faire par l'API ; P1-7 tests manquants (sélecteur
  R7, masquage R3 des grilles d'ouverture, route : longueur d'`entries`, 403, clé en lecture seule) ; P1-8 refus
  `laterFiscalYear` des grilles d'ouverture ; P1-9 le formulaire de création ne charge aucun exercice (prop
  `fiscalYears`) ; P1-10 contraintes de l'assistant (deux devises distinctes, différence non modifiable, `details.entry`
  sans `details`, solde net du compte de conversion) ; P1-11 `docs/api-external.md` ; P1-12 registre C-37-9 (la route
  revalide le compte de conversion à l'usage, que C-37-9 réservait à la réévaluation).
  **Inter-fiches non appliqués — texte proposé.** *37-3a AC-4, 2 bis* : « L'exemption vaut aussi pour le `PUT`
  d'une écriture que son contenu stocké fait déjà porter plusieurs devises (ouverture, complément, état hérité) :
  `update_in_tx` refuse `FX_MULTIPLE_CURRENCIES` seulement si le corps porte plus d'une devise **et** que l'écriture
  stockée en portait au plus une (l'ensemble des devises ne s'élargit jamais) ; test : `PUT` du libellé d'une ouverture
  à deux devises → 200 ; `PUT` qui ajoute une seconde devise à une écriture à une devise → 400. » *Registre, C-37-9* :
  ajouter « la route `POST /journal-entries/conversions` (37-3b) revalide le compte de conversion à l'usage
  (`FX_ACCOUNT_INVALID`, `field: "fxTransitAccountId"`), en plus de la réévaluation (37-5) ».
  Axes exercés : citations de code (routes, dépôt, frontend, registre, plans livrés), contrat de 37-3a, 37-1, 37-2,
  comptabilité de la conversion (maquette recalculée : 1000 × 0.9470 = 947.00 ; 1078.40 × 0.8846 = 953.95 ; gain 6.95),
  registre des routes, gardes de texte, R3 à R8. Non exercés : sauvegarde (aucune table ni colonne neuve ici), MCP
  (aucune sortie touchée), E2E non joués. Un GET de la source journalière : `xmldaily?d=20231231` (dimanche) rend le
  `<datum>` 29.12.2023 valable le 31.12 — la veille d'une ouverture au 1er janvier se lit bien.
- 2026-11-14 — alignements de l'orchestrateur après CR-2 P1 : (a) 37-3a-CR2-P1-IF1 — tester le complément
  d'ouverture à deux devises **par sa route** (exemption de `FX_MULTIPLE_CURRENCIES`) ; afficher
  `ACCOUNT_REQUIRES_CURRENCY` avec `details.reason = "monoCurrencyFiscalYear"` par un message propre (clé i18n dans
  les quatre locales) ; (b) 37-3b-CR2-P1-2 porté dans 37-3a AC-4 (2 bis) : le `PUT` d'une ouverture déjà à plusieurs
  devises reste admis ; (c) 37-3b-CR2-P1-12 porté au registre de l'epic (la route de conversion revalide le compte de
  conversion à l'usage).
- 2026-11-14 — validation CR-2 P2 (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW ; remédiés dans la fiche sauf
  37-3b-CR2-P2-9 (37-3a), inter-fiche. MEDIUM : P2-1 l'AC-2 (*Une devise par écriture*) disait encore le `PUT` d'une
  ouverture à deux devises refusé et l'exemption « à la seule création », contre 37-3a AC-4 (2 bis) alignée — écriture
  relue à plusieurs devises modifiable, sélecteur limité aux devises déjà portées, Vitest ; P2-2 l'alignement (a) de
  l'orchestrateur n'était écrit qu'au Change Log — message propre de `ACCOUNT_REQUIRES_CURRENCY`
  `reason: "monoCurrencyFiscalYear"` et test du complément à deux devises par sa route portés dans l'AC-2 et l'AC-1 ;
  P2-3 route de conversion sans verrous ni ordre des refus — `companies` puis compte de conversion en
  `LOCK IN SHARE MODE` avant le premier `create_in_tx`, ordre des refus, écritures créées dans l'ordre chronologique.
  LOW : P2-4 « le `lookup` rend 400 avant 2000 » faux (37-2 AC-5 : la borne est celle du `POST /fetch`, le `lookup`
  rend 404) ; P2-5 la page des soldes d'ouverture ne charge aucun exercice (chargement et repli écrits) ; P2-6 la fiche
  d'écriture charge déjà les exercices au montage (`loadFiscalYears` :100, 29-1b), réemployés ; P2-7 Vitest du
  sélecteur R7 mal formé ; P2-8 E2E R3 : compte USD créé après la bascule ; P2-10 affichage des refus de l'assistant
  (`details.entry`, `details.line`). (P2-9, inter-fiche, ci-dessous.)
  **Inter-fiche non appliqué — texte proposé.** *37-3a AC-4, 2 bis*, remplacer « ne refuse que si le corps porte plus
  d'une devise alors que l'écriture stockée en portait au plus une » par « ne refuse que si le corps porte plus d'une
  devise **et** une devise que l'écriture stockée ne portait pas (l'ensemble des devises ne s'élargit jamais ; une
  ouverture à EUR et USD ne reçoit pas de GBP par un `PUT`) ; test : `PUT` qui ajoute GBP à une ouverture EUR + USD →
  400 ». Motif : la règle actuelle laisse un `PUT` ajouter une troisième devise à un état hérité, contre R7.
  Axes exercés : diff `b36599d0..HEAD` et alignements ; citations de code (routes et dépôt des soldes d'ouverture,
  `lib.rs` `comptable_routes` :524-570, registre des routes :1448-1502, `errors.rs` :134/:210/:224,
  `modification_blocker` :1243, `blocker-messages.test.ts` :51, `JournalEntryForm` :105-206, pages `journal-entries`
  et `[id]` :68/:100-153/:305-307/:418, page des soldes d'ouverture) ; contrats 37-1 (AC-1 à AC-3 bis), 37-2 (AC-5),
  37-3a (AC-4, AC-6, AC-7), 37-5 (AC-2, verrous) ; comptabilité de la conversion (sens de la différence) ; i18n ; E2E
  (préalables R3). Deux GET de la source journalière : `xmldaily?d=20240331` (Pâques) rend `<datum>` 28.03.2024
  (`<gueltigkeit>` 29.03 seul, non lu par 37-2) et `d=20240630` rend 28.06.2024 — la veille d'une ouverture au 1er avril
  ou au 1er juillet se lit. Non exercés : sauvegarde (aucune table ni colonne neuve ici), MCP (aucune sortie), audit
  (aucune action neuve), E2E et Vitest non joués, PDF du manuel (non écrit).
- 2026-11-14 — validation CR-2 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW ; remédiés dans la fiche. MEDIUM :
  37-3b-CR2-P3-1 l'AC-2 disait encore le `POST /fetch` borné au « dernier mois publié » (« dès le 25, le mois suivant
  se récupère ») et le cours « publié le 25 de M−1 », contre la phrase ajoutée en CR-2 P2 juste après, 37-2 AC-5 et
  R4 amendé (mois suivant à toute date). LOW : P3-2 le renvoi à 37-3a AC-4 (2 bis) disait le `PUT` refusé si l'écriture
  stockée portait au plus une devise, la règle alignée étant « plus d'une devise **et** une devise de plus » ;
  P3-3 « écriture 1, puis écriture 2 » pour les refus de la conversion, contre l'exécution chronologique écrite à la
  suite. Aucun finding inter-fiche. Axes exercés : diff `02b217ac..cb15f2cd` de la fiche ; renvois à 37-2 (AC-5,
  *Source*), 37-3a (AC-4, 2 bis), 37-1 (AC-3 : ordre `companies` → exercices → comptes), 37-5 (AC-2 (0), (3)), R4 amendé ;
  `audit_route_registry.rs:1325` (27) ; argument d'absence de cycle sur le compte de conversion. Non exercés : code
  des pages (lignes citées en CR-2 P2 non relues), i18n, E2E, PDF.
- 2026-11-14 — validation CR-2 P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW ; remédiés dans la fiche. LOW :
  37-3b-CR2-P4-1 l'AC-2 disait le cours « publié au plus tard le 25 de M−1 », ce que ni R4 (« le 25 de M−1, ou le jour
  ouvrable suivant ») ni R4 amendé (calendrier « à confirmer par Beat ») ni 37-2 *Source* n'affirment : phrase réalignée,
  sans effet sur le comportement (Kesh n'en dépend pas) ; 37-3b-CR2-P4-2 le test « refus de la seconde écriture → rien
  de créé » de l'AC-2 bis restait ambigu depuis l'exécution chronologique (CR-2 P3-3) : précisé (seconde **exécutée**,
  placée en tête du corps, `details.entry` = rang dans le corps). Aucun finding inter-fiche. Axes exercés : diff
  `cb15f2cd..dab5975d` de la fiche (trois hunks) ; renvois à 37-2 AC-5 (bornes ≥ 2000-01, ≤ mois suivant à toute date,
  rôles) et *Source*, 37-2 AC-4 (vérification quotidienne), R4 et R4 amendé, 37-3a AC-4 (2 bis : règle du `PUT`,
  mot pour mot), cohérence interne de l'AC-2 bis (`details.entry` 1/2 = colonnes, ordre d'exécution, réponse dans
  l'ordre du corps). Non exercés : code des pages et du serveur (aucune ligne de code citée par les hunks), i18n, E2E,
  Vitest, PDF.
- 2026-11-14 — révision CR-3 (revue de Beat S1–S6 ; Opus) : **S2** — ligne d'écart (AC-2, R8) sur les comptes de
  différence du **compte en devise concerné** par `fxAccountsFor` (37-1 AC-4 bis) : premier compte de bilan en devise
  de l'écriture, sinon premier compte en devise, avis si les paires diffèrent (règle de l'orchestrateur, à confirmer) ;
  props du formulaire : défaut de la société, comptes propres lus sur `AccountResponse` ; assistant *Conversion*
  (AC-2 bis) : différence sur les comptes du compte en devise de l'écriture 2 ; la route de conversion ne revalide
  toujours que le compte de conversion. **S5** — assistant TVA achat en devise (AC-2, puce neuve) : sélecteurs ouverts
  aux comptes d'**une** devise en exercice multi-devises, TVA calculée en devise par `lineVatAmount`, ligne d'impôt
  préalable en CHF au même cours (`computeChf`, règle de 37-3a), écart d'arrondi d'un centime laissé au bouton d'écart
  (aucune ligne ajoutée par l'assistant), tests USD 100.00 à 0.8846 (équilibré) et USD 12.29 à 0.8850 (TVA CHF 0.885
  → 0.89, écart d'un centime), vecteurs recalculés au décimal exact ; AC-4 (inventaire des sélecteurs) aligné.
  **S6** — l'écran de saisie n'affiche ni valeur republiée ni avertissement (37-2 AC-6) ; `FX_RATE_NOT_AFC` après une
  adoption. E2E (AC-5) et manuel (AC-6) complétés. S1, S3, S4 sans point de contact. Prémisses relues au code :
  `vat-purchase.ts:36`, `:73-99` (trois lignes CHF équilibrées par construction) ; `VatPurchaseAssistant.svelte:16-25`
  (props), `:159`, `:204` ; `JournalEntryForm.svelte:349-356` (création seulement) ; `vat_report.rs:13-19` (impôt
  préalable lu en débit CHF). AC touchés : en-tête, AC-2, AC-2 bis, AC-4, AC-5, AC-6. Fiche à revalider (cycle CR-3).
- 2026-11-14 — validation CR-3 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW ; remédiés dans la fiche, aucun
  inter-fiches. MEDIUM : 37-3b-CR3-P1-1 le refus `FX_RATE_NOT_AFC` n'était décrit que pour une adoption entre `lookup` et
  l'envoi, alors que 37-3a AC-4 (l. 222) le fait survenir sur toute écriture ancienne à lignes `afc` après adoption : action
  « Garder mon cours » ajoutée ; 37-3b-CR3-P1-2 changement de date après l'insertion de l'assistant TVA (lignes CHF
  dérivées périmées, déséquilibre muet) : avis, bouton d'écart, test. LOW : P1-3 ligne CHF dérivée arrondie à 0.00 ;
  P1-4 « deux demi-centimes » faux (trois arrondis, < 0.015) ; P1-5 « compte crédité côté USD » ambigu (relevé contre
  grand livre) ; P1-6 préalables E2E de S2/S5 ; P1-7 tâches CR-3 absentes. Axes exercés : diff `294889cc..b6f88c24` ;
  vecteurs S5 recalculés (100 @ 0.8846 : 88.46 + 7.17 = 95.63 ; 12.29 @ 0.8850 : 10.88 + 0.89 = 11.77 contre 11.76) ;
  code : `vat-purchase.ts` (`lineVatAmount`, `buildPurchaseVatLines`), `VatPurchaseAssistant.svelte` props,
  `JournalEntryForm.svelte:349-356`, `balance.ts computeBalance`, `LineDraft`, `vat_report.rs` (solde `débit − crédit`),
  `audit_route_registry.rs:1325` (27) ; renvois 37-1 AC-4 bis (`fxAccountsFor`), 37-2 (`lookup` : `afcRate` en vigueur,
  `republished`), 37-3a AC-4 ; comptabilité de la conversion et de l'écart. Non exercés : sauvegarde, MCP, audit (aucune
  table ni action neuve ici), source journalière (aucun GET : sans objet pour cette fiche), E2E, Vitest, PDF.
- 2026-11-14 — validation CR-3 P2 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW ; remédiés dans la fiche, aucun
  inter-fiches. MEDIUM : 37-3b-CR3-P2-1 (arbitrage A17) l'AC-2 faisait survenir `FX_RATE_NOT_AFC`, après adoption, sur
  « toute écriture ancienne à lignes `afc` dont on corrige seulement le libellé » — remédiation de CR-3 P1-1, contraire à
  A17 : le refus ne vise plus que les lignes `afc` nouvelles ou modifiées (cas type : montant seul corrigé, provenance
  relue gardée), les deux actions pour ces lignes seulement, Vitest d'une édition du seul libellé ; Tâches et manuel
  (AC-6) alignés ; 37-3b-CR3-P2-2 l'exemple de `RATE_UNAVAILABLE` (AC-2) était le `PUT` d'une écriture ancienne à
  lignes `afc` après restauration, que A17 fait passer pour un libellé seul : exemple restreint aux lignes nouvelles ou
  modifiées. LOW : P2-3 AC-1 « l'écriture d'ouverture ne se modifierait plus » (contraire à A17) ; P2-4 « erreurs de
  moins d'un demi-centime » faux — au plus 0.005, atteint au cas `x.xx5` de la fiche (0.885 → 0.89) ; P2-5 avis
  « cours changé » sans critère de déclenchement (cours d'insertion retenu par l'indicateur) ; P2-6 préalables E2E :
  l'absence de taux désactive par `noRates`, non `configMissing` (`VatPurchaseAssistant.svelte:55-56`) ; P2-7 « compte
  débité (côté EUR) / crédité (côté USD) » de l'AC-2 bis, sens du relevé contraire au grand livre, résidu de CR-3 P1-5
  non propagé. Axes exercés : diff `b6f88c24..d9c0018e` de la fiche ; A17 et 37-3a AC-4 (l. 221-229, texte encore
  marqué « à réécrire ») ; code : `vat-purchase.ts:36-99` (`lineVatAmount`, `buildPurchaseVatLines`, TVA nulle → deux
  lignes), `VatPurchaseAssistant.svelte:16-121` (props, `configMissing`, `noRates`, `htValid`, `canInsert`, `insert`) ;
  37-1 AC-4 bis (`CreateAccountRequest` prend `fxGainAccountId`/`fxLossAccountId` : préalable E2E réalisable) ;
  vecteurs S5 recalculés (12.29 × 0.081 = 0.99549 → 1.00 ; 10.87665 → 10.88 ; 0.885 → 0.89 ; 11.76165 → 11.76 ; écart
  0.01 ≤ 3 × 0.005) ; comptabilité de la conversion (sens au grand livre). Un GET de la source journalière :
  `xmldaily?d=20261231` (date future) rend `<datum>13.11.2026</datum>` sans erreur — la garde « veille strictement
  passée » de l'AC-1 (et 37-2, l. 85-86) est donc nécessaire, et tenue. Non exercés : sauvegarde, MCP, audit (aucune
  table ni action neuve depuis CR-2), i18n (clés neuves non énumérées), E2E, Vitest, PDF.
- 2026-11-14 — validation CR-3 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche, aucun
  inter-fiches. LOW : 37-3b-CR3-P3-1 la liste des champs d'une ligne `afc` « modifiée » (AC-2, `RATE_UNAVAILABLE`) omettait
  la provenance, que 37-3a AC-4 (4) compte (`manual` repassée `afc`). Axes exercés : diff `d9c0018e..040185ac` de la fiche,
  phrase par phrase ; A17 (epic l. 202-203) et 37-3a AC-4 (l. 221-235), AC-7 (l. 414) ; `VatPurchaseAssistant.svelte:55-56`
  (`configMissing`, `noRates`) ; sens débit/crédit de la conversion (l. 362 contre l. 375-376) ; arithmétique du seuil
  n × 0.005 et du cas `x.xx5` (l. 329-330) ; cohérence Tâches et AC-6. Non exercés : sauvegarde, MCP, audit, i18n, E2E,
  Vitest, PDF.
- 2026-11-14 — validation CR-3 P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche, aucun
  inter-fiches. LOW : 37-3b-CR3-P4-1 la liste de l'AC-2 (`RATE_UNAVAILABLE`) écrivait « côté débit/crédit » et
  « montant » là où 37-3a AC-4 (4) compare débit et crédit en valeur et le montant de `foreign`, et omettait que ni
  l'ordre ni le projet n'entrent dans la comparaison (le `PUT` d'un projet ou de lignes CHF passe aussi) : alignée mot
  pour mot. Axes exercés : diff `040185ac..53a885f4` de la fiche ; 37-3a AC-4 (l. 224-235) ; cohérence avec « Reprendre
  le cours attendu » (cours changé, ligne recontrôlée au cours attendu) et « Garder mon cours » (provenance `manual`,
  hors contrôle), l. 230-238 ; Vitest de l'édition du seul libellé ; AC-1 l. 74-78. Non exercés : sauvegarde, MCP,
  audit, i18n, E2E, PDF, code.
- 2026-11-14 — révision CR-4 (revue 2 de Beat T1–T6 ; Opus) : **T3** — la ligne ne stocke pas de devise (37-3a AC-1) :
  types TS — réponse `foreign.currency: string` lue sur le compte, requête `currency?: string` facultative ; le
  formulaire, les deux grilles d'ouverture, l'assistant TVA et l'assistant *Conversion* l'envoient toujours (garde
  contre un compte dont la devise a changé depuis le chargement) ; `LineDraft.foreign.currency` = devise du compte,
  jamais saisie ; messages de `FX_CURRENCY_MISMATCH` (recharger) et du code neuf `FX_ACCOUNT_IN_CHF` (37-3a AC-4) ;
  affichage du code par la réponse ; tests d'API des deux routes d'ouverture ; guide d'API (AC-1, AC-2, AC-3, AC-6).
  **T4** — cours d'ouverture proposé = cours de clôture de l'exercice précédent : genre `day` inchangé (veille de
  l'ouverture) ; genre `month` → cours mensuel du **dernier mois de l'exercice précédent** (mois de la veille de
  l'ouverture : décembre, juin, mars), et non plus le premier mois du nouvel exercice ; calcul partagé avec `openingDate` de
  37-2 ; tests et manuel (AC-1, AC-6). Remplace, pour le genre `month`, la phrase de CR-2 « cours mensuel du premier
  mois » (entrée ci-dessus, laissée telle quelle). T1, T2, T5, T6 sans autre point de contact. AC touchés : en-tête,
  AC-1, AC-2, AC-3, AC-6. Fiche à revalider (cycle CR-4).
- 2026-11-14 — validation CR-4 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW ; remédiés dans la fiche sauf
  37-3b-CR4-P1-2 pour sa part dans 37-2 (inter-fiche). MEDIUM : P1-1 la formule du genre `month` (« 1er du mois de
  `start_date` − 1 jour ») se lit « un jour avant le 1er du mois d'ouverture », d'où février pour une ouverture au
  15.03 (T1 permet un début quelconque ; 37-2 AC-5 dit « 1er du mois de cette veille ») : formule et test du 15.03
  écrits ; P1-3 tests manquants de T3 : corps envoyé par les deux grilles, l'assistant TVA et l'assistant *Conversion*
  portant la devise du compte ; messages et Vitest de `FX_CURRENCY_MISMATCH` et `FX_ACCOUNT_IN_CHF` (quatre locales).
  LOW : P1-2 « fonction de calcul partagée » entre la page TypeScript et l'écran *Cours de change* (Rust, 37-2 AC-5)
  irréalisable : fonction TS aux vecteurs de 37-2 ; P1-4 Tâches sans item CR-4 ; P1-5 test « dix-huit mois » redondant
  avec « 1er juillet » (la durée est sans effet sur la date d'ouverture ; T2 six mois) et E2E du genre `month` absent.
  **Inter-fiche non appliqué — texte proposé.** *37-2 AC-5, l. 525* : remplacer « et le cours proposé à l'ouverture
  (37-3b AC-1) » par « ; la page des soldes d'ouverture (37-3b AC-1), en TypeScript, recalcule la même date par une
  fonction pure TS dont les vecteurs sont ceux des tests de la fonction Rust (une fixture JSON commune, comme
  `fx_rounding.json`, est préférable) ». Axes exercés : diff `2c0f5afc..bd7d0ec7` de la fiche ; T3 contre 37-3a AC-1,
  AC-2, AC-4 (rangs 1 à 3), AC-6 ; T4 contre 37-1 AC-3 bis (l. 204), 37-2 AC-4 (l. 337-339) et AC-5 (l. 522), A22 ;
  `create_opening_complement_in_tx` (le contrôle de devise passe par `create_in_tx`, après les verrous ; aucun résidu de
  colonne `currency` de ligne dans la fiche) ; types TS ; comptabilité de l'écart et de la conversion inchangée. Non
  exercés : sauvegarde, audit, MCP (aucune table, action ou sortie neuve ici), source journalière (aucun GET), E2E,
  Vitest, PDF.
- 2026-11-14 — validation CR-4 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW ; tous remédiés dans la fiche.
  MEDIUM : 37-3b-CR4-P2-1 (AC-5, E2E du genre `month` ajouté en P1) — « ouverture au 1er juillet : juin » contredit le
  seed `with-company` qu'utilise l'E2E (exercice 01.01.2020–31.12.2030, `test_fixtures.rs:118-119` : décembre 2019) ;
  et l'enchaînement `day` puis `month` ne tient pas si l'ouverture est enregistrée en `day` (la page passe au
  complément, cours du mois de sa date) : mois du seed écrit, cours vérifiés au préremplissage, enregistrement après le
  passage en `month`. LOW : P2-2 (AC-1) vecteurs « (jour, mois, 29.02, 15.03, 1er janvier) … même fixture si 37-2 la
  fournit, sinon recopiés » — 37-2 AC-5 fournit désormais `fx_opening_dates.json` (cas réunis, 29.02 compris) : fixture
  nommée, alternative retirée ; P2-3 (AC-1) renvoi « 37-1 T1 » → 37-1 AC-11 (T1). Axes exercés : diff
  `bd7d0ec7..a8f89e04` de la fiche ; entrée CR-4 P1 ; formule du mois de la veille (15.03, 01.04, 01.07) contre epic T4
  et 37-2 AC-4/AC-5 ; début quelconque contre 37-1 AC-11 ; parité TS/Rust contre 37-2 AC-5 (fixture) ; codes
  `FX_CURRENCY_MISMATCH` / `FX_ACCOUNT_IN_CHF` contre 37-3a AC-3/AC-4 (`details.line`) ; seed E2E au code ; item
  Tâches CR-4. Non exercés : code applicatif hors seed (aucune ligne neuve citée), source factice de 37-2 AC-3 dans le
  détail, 37-4/37-5, manuel, PDF.
- 2026-11-14 — validation CR-4 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW ; aucune remédiation. Axes
  exercés : diff `a8f89e04..2f792399` de la fiche ; seed `with-company` (`test_fixtures.rs:118-119`, 2020-01-01 :
  veille 31.12.2019, décembre 2019) ; fixture `fx_opening_dates.json` contre 37-2 AC-5 (chemin, genres `day`/`month`,
  cas 1er janvier/avril/juillet, 15.03, 01.03.2024, six et dix-huit mois : tous couverts ; le 01.01.2000 de 37-2 n'est
  pas exigé ici, sans contradiction) ; renvoi 37-1 AC-11 (T1) ; enchaînement `day` puis `month` de l'E2E contre AC-1
  (complément au cours du mois de sa date) ; source factice au mois demandé (37-2 AC-3). Non exercés : code
  applicatif hors seed, 37-4/37-5, manuel, PDF.
- 2026-11-14 — révision CR-5 (revue 3 de Beat U1–U6 ; Opus) : **U1** — cours proposé = cours retenu par la société
  (`exchange_rates` de 37-2) : `lookup` rend `effectiveRate` et `source` (et `publishedRate`, non affiché ici) ; ses
  `afcRate` et `overridden` disparaissent ; `afcRate` du formulaire devient une valeur dérivée (`effectiveRate` d'un
  `lookup` `source: "afc"`), si bien que la règle de provenance (`afc` si égal, au mois de la date) et le contrôle
  `FX_RATE_NOT_AFC` de 37-3a jugent contre la même valeur ; indicateurs « AFC » / « cours saisi » lus sur `source`
  (formulaire et soldes d'ouverture) ; ↑ sous le champ vers le cours de la société, sans toucher la table ; 404 de
  `lookup` = aucune ligne de la société (le `POST /fetch` copie) ; restauration : les cours de la société reviennent ;
  « adoption » remplacée par la reprise par ↑, par société ; Vitest de provenance récrits ; E2E (`PUT
  /exchange-rates/rates`, ↑) ; manuel. **U2** — changer le compte d'une ligne : CHF → devise (champs initialisés),
  devise → CHF (`foreign` retiré), USD → EUR (montant et cours réinitialisés, cours EUR proposé), même devise (gardés) ;
  compté comme retouche ; Vitest et E2E. **U5** — A12 noté approuvé ; A8, A13, A14, A21 déjà AC (AC-2, AC-2 bis).
  **U6** — débit et crédit en devise principale (AC-1, AC-2, ligne d'écart). Aucune migration ni route neuve. AC
  touchés : en-tête, AC-1, AC-2, AC-5, AC-6, Tâches. Fiche à revalider (cycle CR-5).
- 2026-11-14 — validation CR-5 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; remédiés dans la fiche sauf
  37-3b-CR5-P1-6 (37-2, inter-fiche). MEDIUM : CR5-P1-1 `FX_RATE_NOT_AFC` à `expectedRate` `null` (cours de la société
  devenu `manual` après l'écriture, 37-3a AC-4, 4) : « Reprendre le cours attendu » ne pouvait pas s'appliquer — seule
  « Garder mon cours » est offerte, Vitest et E2E ; CR5-P1-2 le manuel (AC-6) disait « moyenne du 25 au 24 », contre la
  Terminologie de Beat (« pas des moyennes ») et R4 amendé (méthode non confirmée) : réécrit. LOW : P1-3 E2E du cours
  saisi et du ↑ irréalisables tels quels (`version` d'une ligne `afc` copiée, valeur publiée requise par `reset`, sinon
  suppression d'une ligne `manual`) ; P1-4 AC-2 « le ↑ et son dialogue » contre A23 (↑ direct) ; P1-5 `PUT /rate-kinds`
  exige les deux genres ; P1-6 (inter-fiche) 37-2 AC-3 bis. **Inter-fiche non appliqué — texte proposé.** *37-2 AC-3 bis,
  fonction de copie* : remplacer « **toutes** les devises publiées de l'unité (une devise d'un compte créé plus tard
  trouve ainsi son cours) » par « les devises **portées par un compte de la société** (A24) ; donner une devise à un
  compte déclenche la copie des cours déjà récupérés pour cette devise (37-1, 37-2) ». Axes exercés : diff
  `0ae43432..d59a1de9` de la fiche ; `lookup`, `PUT …/rates`, `…/rates/reset`, `PUT /rate-kinds` et copie de 37-2
  (AC-3 bis, AC-5, AC-7) ; 37-3a AC-4 (4) et AC-7 (U2) ; reste des anciens noms (`exchange_rate_overrides`,
  `_kesh_exchange_rates`, `republished_*`, `overridden`, `afcRate` du `lookup`) : aucun hors Change Log ; U1, U2, U3, U6,
  A23, A24 ; Terminologie. Non exercés : sauvegarde et rejeu (aucune table neuve ici), audit, MCP, source journalière
  (aucun GET), E2E, Vitest, PDF, citations de lignes de code antérieures à CR-5.
- 2026-11-14 — validation CR-5 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW ; remédié. MEDIUM :
  37-3b-CR5-P2-1 (AC-5, E2E) — l'insertion de CR5-P1-3 avait déplacé « le cours proposé part `afc` », conclusion du ↑,
  à la fin du cas « cours `manual` apparu après l'écriture », où il contredisait `expectedRate` `null` (aucun cours
  `afc` pour ce mois, 37-3a AC-4) et laissait le ↑ sans vérification : remis au ↑ ; le cas nouveau se conclut par
  « Garder mon cours » → ligne `manual`, écriture enregistrée. Axes exercés : diff `d59a1de9..3b058509` de la fiche ;
  37-3a AC-4 (`expectedRate` `null`, ligne `manual` de la société) ; 37-2 AC-5 (`PUT …/rates` et `version`, `…/rates/reset`
  et `publishedRate`, suppression sans valeur publiée, `PUT /rate-kinds` : deux genres obligatoires,
  `applyToNextOpening` absent = `false`), `Cell.row.version` du `GET` ; A23 ; Terminologie et R4 amendé/S1 ; seed
  `with-company` (`test_fixtures.rs:118-119`, un seul exercice : pas d'exercice suivant). Non exercés : reste de la
  fiche hors diff, Vitest, PDF, audit, sauvegarde. Aucun finding inter-fiche.
- 2026-11-14 — validation CR-5 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW. Axes exercés : diff
  `3b058509..dc6c1c5a` de la fiche (deux phrases de l'AC-5, E2E) ; conclusion du ↑ (`afc`) placée au ↑, cohérente avec
  `expectedRate` `null` de 37-3a AC-4 (l. 266) et avec l. 298-301 de la fiche (« Garder mon cours » → `manual`,
  CHF non recalculé) ; 37-3a l. 823 ; A30 sans objet ici (37-2 seulement). Non exercés : reste de la fiche hors diff,
  Vitest, PDF, audit, sauvegarde. Aucun finding inter-fiche.
- 2026-11-14 — alignement A30 (orchestrateur, Sonnet) : une `source` de cours inconnue (`RateOrigin::Other(String)`, p. ex. `ecb`, 37-2) se traite comme `manual` à l'AC-2 : `afcRate` est `null` si `source` ≠ `"afc"` ; `rateSource` du ↑ `manual` ; indicateur « cours saisi » suivi de la source brute entre parenthèses ; Vitest ajouté (`lookup` `source: "ecb"` → `afcRate` `null`, ligne `manual`, indicateur). Aligné sur 37-3a AC-4 (`expectedRate` `null`) et 37-2 (`lookup`).
- 2026-11-14 — validation CR-5 P5 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW ; remédié dans la fiche.
  MEDIUM 37-3b-CR5-P5-1 (AC-1, ouverture et complément) la provenance affichée (« proposé : AFC <mois> », ou « cours
  saisi » quand `source = "manual"`) aurait étiqueté « AFC » un cours `ecb` — « cours saisi (<source brute>) » pour
  toute autre source, comme à l'AC-2, Vitest. Vérifié : AC-2 (`afcRate` `null` si `source` ≠ `"afc"`, ↑ → `manual`,
  indicateur), tâche `source === 'afc' ? effectiveRate : null` cohérents avec 37-2 et 37-3a. Non exercés : reste de la
  fiche hors diff, Vitest, rendu.
- 2026-11-14 — validation CR-5 P6 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW ; aucune modification. Phrases A30 du diff P5 vérifiées contre 37-2 (AC-1, AC-5, AC-6) ; aucun site restant. Non exercés : reste de la fiche hors diff, compilation, rendu.
- 2026-11-14 — révision CR-6 (couverture API et MCP, Opus) : **V1** — « cours OU CHF » par `rateFromChf` (`fx.ts`),
  jumeau de `rate_from_chf` de `kesh-core` (37-3a AC-3), fixture partagée `fx_rate_from_chf.json` (AC-1, AC-2) ; la
  provenance de l'écran est la règle de qualification du serveur (AC-2) ; DTO d'ouverture et complément passés par la
  fonction d'analyse partagée de 37-3a AC-6 (`rate` ou `chf`, `rateSource` facultatif ; AC-1) ; **V2** — `debit`/
  `credit` facultatifs avec `side` sur les deux routes d'ouverture (AC-1) ; **V3** — route de conversion : ligne de
  différence calculée par le serveur si le corps ne la porte pas, paire contrôlée sinon, `fx_accounts_for` puis
  `validate_fx_accounts_in_tx`, cycle neuf avec la bascule de 37-1 défendu par le rejeu (AC-2 bis) ; **V5** — limite
  écrite de l'assistant TVA et règle d'arrondi pour un client de l'API (`line_vat_amount`, AC-2) ; **V9** — renvoi des
  clients de l'API à la case `opening` de `GET /fiscal-years/{id}/exchange-rates` (AC-1) ; guide d'API complété (AC-6 :
  V1–V9, 422 → 400) ; tâche CR-6. Prémisses relues au code : `routes/opening_balances.rs:73-76` (`debit`/`credit` en
  `String`), `:170-175` (complément, `Option<String>`) ; `kesh-core/src/accounting/vat.rs:40` (`line_vat_amount`,
  arrondi au centime par `round_to_centimes`) ; `frontend/.../vat-purchase.ts:36` (`lineVatAmount`, jumeau) ;
  `kesh-core/src/errors.rs:125` (`ENTRY_UNBALANCED`) ; 37-2 AC-5 (case `opening` du tableau annuel, `Cell`) ; 37-1 AC-4
  bis (`fx_accounts_for`, `validate_fx_accounts_in_tx`). AC touchés : AC-1, AC-2, AC-2 bis, AC-6, Tâches. Fiche à
  revalider (cycle CR-6).
- 2026-11-14 — validation CR-6 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 3 LOW ; remédiés dans la fiche (aucun finding inter-fiches). MEDIUM : 37-3b-CR6-P1-1 (AC-1) le DTO d'ouverture et de complément restait écrit `{ currency?, amount, rate, rateSource }`, sans `chf`, `side`, ni `rate`/`rateSource` facultatifs, contre 37-3a AC-6 ; 37-3b-CR6-P1-2 (AC-1) « le seul accès à `exchange_rates` est le contrôle `FX_RATE_NOT_AFC` » contredisait la qualification d'une ligne sans `rateSource` (37-3a AC-4, 4) ; 37-3b-CR6-P1-3 (AC-2 bis, V3) « l'écriture 1 l'est par construction » (déséquilibrée) était faux, et compléter « celle qui est déséquilibrée » aurait soldé en silence, par un gain ou une perte de change, une erreur du client sur l'écriture 1 — l'écriture à compléter est désormais celle dont le solde net du compte de conversion est un crédit, les autres cas rendent `ENTRY_UNBALANCED` ; 37-3b-CR6-P1-4 (AC-2 bis, V3) la reconnaissance d'une « ligne sur un compte désigné » (comptes de gains/pertes de la société ou de tout compte en devise) n'avait ni requête ni ordre de verrous — requête bornée aux comptes du corps, `LOCK IN SHARE MODE`, après le verrou de `companies`. LOW : 37-3b-CR6-P1-5 (V3) « aucun refus » faux : la ligne ajoutée peut dépasser `MAX_LINES_PER_ENTRY` (500, contrôlé seulement par `prepare_new_journal_entry`, `journal_entries.rs:533`) — 400 `Validation` ; 37-3b-CR6-P1-6 (AC-1, e) ligne CHF d'ouverture sans `debit`/`credit` non spécifiée (`Decimal::from_str(&line.debit)`, `opening_balances.rs:362-368` — 400 comme la chaîne vide) et phrase du guide trop large (le complément tient déjà une colonne absente pour 0) ; 37-3b-CR6-P1-7 (Tâches) tests de refus V1/V2/V3 manquants (sans `side`, `chf` ≠ débit + crédit, ni `rate` ni `chf`, écriture 1 déséquilibrée, 500 lignes). Choix de l'auteur : (a) `FX_AMOUNT_MISMATCH` avec `expected` : correct ; (b) 400 `Validation` pour la mauvaise paire : correct (aucun code de plus, `details { entry, line }` suffisent) ; (c) repérage par le compte : correct une fois la requête et les verrous écrits (P1-4) ; (d) extension aux routes d'ouverture et de complément : correcte ; (e) 422 → 400 : correct, borné aux routes dont le champ devient `Option`. Prémisses relues au code : `opening_balances.rs:75-76`, `:173`, `:175`, `:362-368`, `:515-580` ; `vat.rs:40` ; `vat-purchase.ts:36` ; `journal_entries.rs:395`, `:533` ; 37-2 AC-5 (`Cell`, `opening`). Axes exercés : diff `b6e73cc3..d01ad946` de la fiche ; contrat de l'API seule (combinaisons `rate`/`chf`/`debit`/`credit`/`rateSource` contre 37-3a AC-3, AC-4 4, AC-6) ; choix (a) à (e) ; V5 (arrondi `line_vat_amount` / `lineVatAmount`) ; V9 ; guide d'API ; verrous de V3. Non exercés : MCP (la fiche n'étend aucun outil), activation en Bearer (37-1, 37-5), compilation, rendu.
- 2026-11-14 — validation CR-6 P2 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW ; remédiés dans la fiche (aucun finding inter-fiches). MEDIUM : 37-3b-CR6-P2-1 (AC-2 bis, V3, *ordre*) la remédiation P1-3 choisit l'écriture à compléter par le compte de conversion, dont l'identifiant n'est lu que sous verrou, alors que l'*ordre* exigeait `accounting::validate` « avant la transaction » pour l'écriture qui ne reçoit pas de ligne — irréalisable ; avant la transaction : analyse V1/V2, limite de lignes, contrôle FX par ligne (partie de `prepare_new_journal_entry` sans `validate`, extraite) ; dans la transaction, après `FX_CONVERSION_TRANSIT_UNBALANCED` : identification, équilibre de l'autre écriture, forme et paire, ajout, équilibre de l'écriture complétée ; ordre des refus et test « deux écritures déséquilibrées » alignés ; 37-3b-CR6-P2-2 (AC-2 bis, V3, comptabilité) la ligne calculée soldait une erreur de sens du client (compte USD au crédit, `side: "credit"`) par une perte de deux fois le CHF, et « le compte en devise de cette écriture » était indéterminé avec deux comptes en devise — forme exigée (un seul compte en devise, au débit net), sinon `ENTRY_UNBALANCED` ; ligne portée sans compte en devise unique → 400 ; l'absorption résiduelle (montant, cours, frais oubliés, CHF du compte de conversion) écrite comme limite au guide d'API. **Question à Guy** (non appliquée, V3 de l'epic dit « un client qui l'omet la délègue au serveur ») : exiger un consentement explicite du corps (champ qui demande le calcul) ou une borne de plausibilité de la différence. LOW : P2-3 reconnaissance des comptes désignés bornée à l'écriture à compléter (portée « du corps » ambiguë, verrous plus larges que le paragraphe du cycle) ; paragraphe du cycle élargi ; P2-4 paire lue à l'usage seulement (écriture à compléter équilibrée sans ligne de différence : aucun `FX_ACCOUNT_INVALID`) ; P2-5 test « ligne ajoutée à l'écriture déséquilibrée » (résidu d'avant P1-3) → « à compléter » ; « gain au débit → 400 » exige une fixture à comptes de gains et de pertes distincts (37-1 admet le même compte) ; tests de la forme ajoutés ; P2-6 guide d'API sans la règle de l'écriture complétée ; P2-7 le refus « trop de lignes » est un `AppError::Validation` du handler, non un refus de `map_core_error`. Prémisses relues au code : `opening_balances.rs:70-76`, `:166-176`, `:345-375` ; `journal_entries.rs:395`, `:533-535` (route) ; `opening_complement.rs:690-719` (complément par `create_in_tx`, donc `create_in_tx_inner`) ; `errors.rs:153` (`Validation(String)`, sans `details` : mécanisme commun de `details.entry` requis) ; `middleware/rbac.rs:42-67` et `auth/api_key.rs:179-219` (PAT à portée) ; écran `opening-balances/+page.svelte:199-200`, `:245-246` (colonne vide envoyée `"0"`). Axes exercés : diff P1 `16f1cdb2..19039295` et révision `b6e73cc3..d01ad946` de la fiche ; cohérence 37-3a AC-3, AC-4 4, AC-6 (DTO, trois états de `debit`/`credit`, `rate_from_chf`, qualification) ↔ AC-1 ; comptabilité de V3 (détermination, sens, absorption) ; contrat de l'API seule de la conversion ; ordre des refus et des verrous ; tests. Non exercés : MCP au-delà du constat (la fiche n'étend aucun outil ; les écritures de conversion se lisent par `kesh_ecritures`, 37-3a AC-8), activation en Bearer (37-1, 37-5), i18n, compilation. Écart de protocole déclaré : six remplacements de texte de la fiche faits par un script Python ad hoc au lieu de l'outil Edit (contenu identique à des Edit, seule la fiche touchée).
- 2026-11-14 — arbitrages A31/A32 et reports IF1/IF2 (orchestrateur, Sonnet) : **A31** — AC-2 bis : la ligne de différence
  n'est calculée que sur `completeDifference: true` (sans lui `ENTRY_UNBALANCED` ; avec lui et une ligne de différence
  présente, 400 `Validation`), la question à Guy (borne ou consentement) tranchée pour le consentement ; guide d'API,
  tests, tâche CR-6 ; l'écran n'envoie pas le champ. **A32/IF2** — AC-2 : CHF saisi qui redonne `afcRate` du mois →
  cours `afcRate`, `afc`, sinon `rateFromChf`, `manual` ; Vitest. **IF1** — tâche CR-6 : tests d'API de génération et de
  complément (`afc` si le cours redonne le CHF, sinon `manual`). Fiche à revalider (cycle CR-6).
- 2026-11-14 — validation CR-6 P3 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW ; remédiés dans la fiche (aucun finding inter-fiches). MEDIUM : 37-3b-CR6-P3-1 (AC-2 bis, V3, A31) « sans ce champ, tout ce qui suit (écriture à compléter, paire, ligne calculée) ne s'applique pas » laissait sans contrôle de paire la ligne de différence que l'écran envoie sans le champ (D6), contredisait le bullet « corps qui porte une ligne de différence, sans `completeDifference` », et ne disait pas qui valide (`accounting::validate`, hors `prepare_new_journal_entry` depuis P2-1) les deux écritures sans le champ — l'ordre vaut désormais dans les deux modes (autre écriture, puis écriture à compléter ; deux écritures déséquilibrées rendent toujours le refus de l'écriture qui n'est pas à compléter), seuls l'ajout et « trop de lignes » dépendent du champ. LOW : P3-2 le refus `completeDifference: true` + ligne présente n'avait pas de rang dans l'ordre des refus (placé en tête de « forme et paire ») et le champ non booléen pas de test ; P3-3 les refus d'avant la transaction (V1/V2, limite de lignes, contrôle FX par ligne) absents de la liste d'ordre ; P3-4 A32 non propagé à l'AC-1 (qualification d'une ligne sans `rateSource`) ni au guide d'API (AC-6) ; tests ajoutés (sans le champ : ligne portée hors paire → 400, sur la paire → 201 ; `true` sur écritures équilibrées → aucune ligne). Prémisses relues au code : `journal_entries.rs:507-570` (`prepare_new_journal_entry` valide à `:566`, route et non dépôt), `extractors.rs:32-50` (rejets JSON). Cohérence A32 écran ↔ 37-3a AC-4, 4 : sens et arrondi identiques, `afcRate` `null` si source ≠ `afc` : correcte. Comptabilité de V3 relue : écriture 2 (compte en devise au débit net, compte de conversion au crédit net), gain au crédit si les débits l'emportent, perte au débit sinon : correct ; l'absorption résiduelle reste la limite écrite, désormais consentie (A31). Axes exercés : diff `ff3ea962..f46de99c`, AC-1, AC-2 (A32), AC-2 bis (détermination, forme, consentement, ordre, verrous), AC-6, Tâches, tests ; contrat de l'API seule de la conversion ; cohérence avec 37-3a AC-4 (4), AC-6 et son renvoi (AC-5) ; RBAC inchangé. Non exercés : MCP (aucun outil touché), activation en Bearer, i18n, compilation, rendu.
- 2026-11-14 — validation CR-6 P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; remédiés dans la fiche ; une passe de plus est due (MEDIUM). Périmètre : diff `7537d144..7d80c9bd` (remédiation P3). MEDIUM : 37-3b-CR6-P4-1 (AC-6, guide d'API) « si le CHF est celui du cours `afc` retenu, `afc`, sinon le cours déduit et `manual` » était faux pour une provenance explicite — 37-3a AC-4 (4) et ses tests (`rateSource: "afc"` non redonné → `FX_RATE_NOT_AFC` ; `manual` explicite reste `manual`) : phrase réécrite par provenance. 37-3b-CR6-P4-2 (AC-2 bis, assistant) : en étendant forme et paire au mode sans `completeDifference` (P3-1), la fiche rend refusable un corps que l'écran construit — frais de l'écriture 2 sur un second compte de la devise (offert : seuls les comptes d'une autre devise sont exclus) avec une différence non nulle → paire indéterminée, 400 ; l'assistant désactive désormais « Passer les écritures » avec ce motif (Vitest). LOW : P4-3 intitulé « écriture à compléter (avec la demande) » contraire à « dans les deux cas » ; P4-4 ordre d'avant la transaction (analyse avant la limite de lignes) contraire à `prepare_new_journal_entry` (libellé `:526`, limite `:533`, parse `:541`) — corrigé aux deux sites (puce *ordre*, *Verrous et ordre des refus*) ; P4-5 parenthèse A32 de l'AC-1 sans la provenance `afc` ni la condition « ligne de la société `afc` » ; P4-6 test « `true` sur écritures équilibrées » sans préciser « sans ligne de différence » (avec, 400 A31). Prémisses relues : `journal_entries.rs:515-566`, `extractors.rs:32-50` (`JsonDataError` → 400 `Validation`), 37-3a AC-4 (4) l. 345-355 et tests l. 730-745, epic A31/A32. Inter-fiches (non appliqué) : 37-3a AC-6 cite le parse des montants à `:539-551`, la boucle est à `:541-554` (LOW). Axes exercés : chaque phrase ajoutée par P3 (AC-1 A32, AC-2 bis demande/ordre/verrous/tests, AC-6 A32) contre le code, l'epic et 37-3a ; cohérence écran ↔ serveur de la conversion sans le champ. Non exercés : 37-5 (aucune phrase de P3 n'y touche), reste de la fiche, MCP, i18n, compilation.
- 2026-11-14 — validation CR-6 P5 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW ; remédiés dans la fiche ; la boucle peut se clore (aucun finding au-dessus de LOW, remédiation sans code de production). LOW : 37-3b-CR6-P5-1 (AC-2 bis, puce *ordre*) « à chaque écriture … puis à chaque ligne » pouvait se lire libellés de toutes les écritures d'abord, contre l'ordre écriture par écriture de `prepare_new_journal_entry` et de *Verrous et ordre des refus* : « écriture par écriture dans l'ordre du corps » ajouté ; 37-3b-CR6-P5-2 (Tâches, Vitest de l'assistant) le Vitest du bouton désactivé pour la paire indéterminée (P4-2), écrit à l'AC-2 bis, manquait à la liste des tests : ajouté. Relu au code et vrai : `journal_entries.rs:526` (libellé), `:533` (limite), `:540-554` (parse), `:566` (`validate`) ; AC-1 et AC-6 (A32) conformes à 37-3a AC-4 (4) l. 345-355 (provenance absente ou `afc`, ligne de la société `afc`, `manual` explicite jamais touché) ; paire indéterminée de l'assistant cohérente avec la forme exigée (deux comptes en devise distincts, ligne de différence portée → 400) et avec le test déjà listé. Axes exercés : chaque phrase du diff `0bcdaeac..19ec8732` (AC-1, AC-2 bis frais, écriture à compléter, ordre, Verrous, tests, AC-6) ; cohérence avec 37-3a AC-4 (4) et AC-6 (ordre de l'analyse) ; comptabilité de l'écriture 2 avec frais. Non exercés : 37-5, reste de la fiche, MCP, i18n, compilation.
- 2026-11-15 — exigences FR/NFR ajoutées (B26 ; orchestrateur, Sonnet) : 5 FR, 5 NFR.
- 2026-11-15 — validation ciblée des exigences (Opus) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 6 LOW ; remédiés (9 FR, 6 NFR). MEDIUM : FR-3 fausse — le bouton d'écart « ajoute une ligne à l'écriture en cours », « ne crée aucune écriture et n'envoie rien » (AC-2, *Ligne d'écart*) — réécrite ; route `POST /journal-entries/conversions` (transaction, `completeDifference` A31, `FX_CONVERSION_TRANSIT_UNBALANCED`, rôles, audit, registre) sans exigence (AC-2 bis → FR-8) ; assistant TVA achat en devise (S5) sans exigence (AC-2 → FR-7) ; cours proposé, provenance `afc`/`manual`, cours d'ouverture de clôture et actions sur `FX_RATE_NOT_AFC` sans exigence (AC-1, AC-2 → FR-6) ; contrat des routes d'ouverture (422 → 400 à la génération, refus FX du complément en 400 jamais 500) sans exigence (AC-1 → FR-9). LOW : FR-1 sans forme partagée, `manual`, mono-devise ni ligne incomplète ; FR-2 sans changement de compte (U2) ni sélecteur R7 ; FR-4 sans « Supprimer » ; FR-5 « comptes » ambigu, exceptions absentes ; NFR-5 sans guide d'API ni G4-bis ; jumeaux client/serveur sans exigence (NFR-6). Récapitulatif de l'epic aligné.
- 2026-11-15 — validation ciblée P2 des exigences (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 5 LOW ; remédiés (10 FR, 6 NFR). MEDIUM : 37-3b-B26-P2-1 affichage des refus FX du serveur sur la ligne fautive (rang 1-based dans le corps sans lignes vides, messages `FX_CURRENCY_MISMATCH`, `FX_ACCOUNT_IN_CHF`, `monoCurrencyFiscalYear`, `laterFiscalYear`, classement `stay`/`stale`, « Récupérer le mois ») sans exigence (AC-1, AC-2, AC-2 bis → FR-10 neuve). LOW : -P2-2 FR-1 « envoyés `manual` » vrai de l'écran seulement (l'API admet `afc` et l'absence, AC-1) ; -P2-3 FR-2 sans « cours OU CHF », avec changement de compte vers la même devise (montant gardé) et sélecteur R7 sans l'exception des écritures relues multi-devises ni le masquage en exercice mono-devise ; -P2-4 FR-5 sans l'exception de l'assistant TVA achat (AC-4, S5) ; -P2-5 FR-6 sans le complément `TODAY` ni le cas `expectedRate` `null` (seule « Garder mon cours ») ; -P2-6 FR-8 sans la forme exigée de l'écriture complétée. Récapitulatif de l'epic aligné (-10 ajouté). Une passe de plus est due (MEDIUM, `CLAUDE.md`).
- 2026-11-15 — validation ciblée P3 des exigences (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 7 LOW ; remédiés (10 FR, 6 NFR, décompte inchangé). MEDIUM : 37-3b-B26-P3-1 FR-2 « CHF saisi → cours déduit par `rateFromChf` » faux dans le cas A32 de l'AC-2 (*Champs*, « CHF saisi à la main ») : si `afcRate` du mois redonne le CHF, le cours devient `afcRate` et la ligne part `afc` — FR-2 corrigée ; -P3-2 verrous et ordre des refus sans exigence : AC-1 (« le serveur ne fait aucun `lookup` … ne lit `exchange_rates` avant aucun verrou », 34-2 AC-10) et AC-2 bis (*Verrous et ordre des refus* : une seule raison, `companies` → compte de conversion, créations dans l'ordre chronologique) — portés par FR-1 et FR-8 (rangées en FR pour ne pas changer le décompte de l'epic ; un NFR de concurrence serait plus juste, à arbitrer). LOW : -P3-3 FR-8 « toute autre forme reste `ENTRY_UNBALANCED` » trop large (équilibrée : rien ; ligne portée sans compte unique : 400 `Validation`) et sans les 400 `Validation` (`entries` ≠ 2, A31 avec ligne portée, trop de lignes), la paire `FX_ACCOUNT_INVALID` à l'usage, la réponse `{ entries }` ; -P3-4 FR-9 « sans `debit`/`credit` » ambigu (une colonne absente suffit), sans le complément inchangé ni `revaluation` → 400 sur les deux routes ; -P3-5 FR-10 « sur la ligne fautive » pour des refus sans `details.line` (`MULTI_CURRENCY_DISABLED`, `FX_MULTIPLE_CURRENCIES`) et règle du corps sans lignes vides appliquée à l'assistant, qui traduit par `details.entry` et sa table ; -P3-6 FR-2 exception R7 réduite à ouverture et complément (AC-2 : contre-passation, état hérité), lien vers l'assistant ; FR-6 sans le cours de la société sous le champ et ↑ (ne touche jamais la table) ; -P3-7 NFR-6 sans le jumeau TS de la date d'ouverture (fixture `fx_opening_dates.json` de 37-2) ni `lineVatAmount` ; -P3-8 FR-5 « pièces » sans le détail de l'AC-4 (factures, avoirs, règlements, factures fournisseur) ; -P3-9 FR-7 sans les deux refus de l'assistant (deux devises, ligne CHF dérivée à 0.00). Axes exercés : chaque FR/NFR contre l'AC entier cité (AC-1 à AC-6 relus) ; points majeurs (contrat d'API, refus, verrous, audit, RBAC, MCP — la fiche n'en touche aucun outil —, sauvegarde — aucune table neuve) ; récapitulatif de l'epic. Non exercés : les AC eux-mêmes, Tâches, code. Récapitulatif de l'epic aligné (-1, -8). Une passe de plus est due (MEDIUM).
- 2026-11-15 — validation ciblée P4 des exigences (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 5 LOW ; remédiés (10 FR, 6 NFR, décompte inchangé). LOW : 37-3b-B26-P4-1 FR-8 « refus portant `details.entry` » trop large (AC-2 bis : `FX_ACCOUNT_INVALID` et `FX_CONVERSION_TRANSIT_UNBALANCED` n'en portent pas, `details: { entry1, entry2 }` pour le second) et ordre sous verrous sans le `FX_ACCOUNT_INVALID` du compte de conversion, premier ; -P4-2 FR-3 sans les gardes de « Passer les écritures » (exercice multi-devises, deux devises différentes, paire invalide, frais de l'écriture 2 sur un second compte en devise) ; -P4-3 FR-10 sans `FX_ACCOUNT_INVALID` ni `FX_CONVERSION_TRANSIT_UNBALANCED` (message neuf, quatre locales, affichés au-dessus de l'aperçu) ; -P4-4 FR-6 repli `POST /fetch` écrit sans condition (AC-2 : une fois, Comptable ou Administrateur, mois à la portée du `fetch`, sinon saisie manuelle), sans la reproposition au changement de date ni la date du complément recalculée qui ne refuse pas le `POST` (AC-1) ; -P4-5 NFR-6 « fonction pure qui lit la fixture » : c'est son Vitest qui la lit (AC-1). Axes exercés : chaque FR/NFR contre l'AC entier cité (AC-1 à AC-6 relus), diff P3, points majeurs (contrat d'API, refus, verrous, audit, RBAC, sauvegarde), récapitulatif de l'epic. Non exercés : les AC eux-mêmes, Tâches, code. Récapitulatif de l'epic aligné (-3).
- 2026-11-15 — A33 frais de conversion (Opus) : décision de Beat du 2026-11-15 (epic, A33 ; B3/B27 de l'Epic 38).
  AC-2 bis : l'écriture à compléter admet, en plus de son compte en devise au débit net, des frais **au débit** de
  comptes de charge (`Expense`, `crates/kesh-db/src/entities/account.rs:16` ; `accountType`,
  `frontend/src/lib/features/accounts/accounts.types.ts:50`) de la même devise ; hors règle « un seul compte en
  devise » (le compte en devise et la paire se prennent hors comptes de charge) ; différence calculée après eux.
  Refus retenus, au rang de la forme, quand elle est examinée : comptes de deux devises → `FX_MULTIPLE_CURRENCIES`
  (`details: { entry, currencies }`, code du dépôt 37-3a AC-4, 2 bis, anticipé pour ne pas rendre un
  `ENTRY_UNBALANCED` trompeur), puis frais au crédit d'un compte de charge en devise → 400 `Validation`
  (`details: { entry, line }`). Type et devise lus par la requête de reconnaissance (mêmes comptes, même verrou).
  Assistant : frais au débit ; garde « second compte en devise » restreinte aux comptes qui ne sont pas des comptes de
  charge de la devise. Touchés : FR-3, FR-8, AC-2 bis (assistant, forme, reconnaissance, ligne calculée, ordre des
  refus, tests API et Vitest), AC-6 (guide d'API), Tâches (CR-6 V3) ; récapitulatif de l'epic (-3, -8). Tests
  ajoutés : frais sur 6911 USD → 201, différence après le frais (avec le champ, et ligne portée sans lui) ; frais au
  crédit → 400 `Validation` ; frais sur un compte de charge EUR dans l'écriture USD → `FX_MULTIPLE_CURRENCIES` ; seul
  compte en devise de charge → `ENTRY_UNBALANCED` ; Vitest : frais sur 6911 → bouton actif. Fiche à revalider
  (passe ciblée sur ce diff).
- 2026-11-15 — validation ciblée A33 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW ; remédiés. Périmètre : diff
  `c6274a58..3c98a3dc`. LOW : 37-3b-A33-1 (AC-2 bis, *forme*) sort d'un frais en CHF (compte de charge sans devise) non
  écrit : ligne CHF ordinaire, hors forme, comptée dans Σ débit avant la différence, sans effet sur l'identification ni
  la paire — phrase et test API ajoutés (frais CHF + frais 6911 USD → 201), A33 de l'epic aligné ; -2 (AC-2 bis,
  *forme*) « le code et les `details` du dépôt » contredisait `details: { entry, currencies }` (37-3a AC-4, 2 bis :
  `{ currencies }` seul) — dit : `{ currencies }` du dépôt plus `entry` posé par la route. Axes exercés : chaque phrase
  ajoutée contre le diff entier ; citations de code (`account.rs:16`, `AccountType::Expense` ; `accounts.types.ts:50`,
  `accountType`) vérifiées au code ; sens gain/perte (Σ débit frais compris > Σ crédit → crédit du compte de gains,
  sinon débit du compte de pertes ; formule du test = CHF conversion − (CHF₂ + frais)) ; cohérence avec A31 (refus
  A31 d'abord), A32, 37-3a AC-4 rang 2 bis ; ordre des refus (`FX_MULTIPLE_CURRENCIES` avant le refus du frais au crédit,
  avant `ENTRY_UNBALANCED` ; tests « deux comptes distincts » restreints à la même devise, aucun résidu à deux devises
  attendant `ENTRY_UNBALANCED` par grep) ; 37-1 (rien n'interdit une devise sur un compte `Expense` non désigné) ;
  requête de reconnaissance (type et devise, sans lecture de plus) ; guide d'API, Tâches, FR-3, FR-8, récapitulatif.
  Non exercés : le reste de la fiche ; code (aucun n'existe).
- 2026-11-16 — révision CR-7 (A34 multi-devises sans retour, Opus) : masquage par exercice retiré — grilles
  d'ouverture (comptes en devise non offerts, motif) et formulaire (prop `fiscalYears`, champs masqués, enregistrement
  désactivé au changement de date) — ; le serveur tranche (409 `MULTI_CURRENCY_DISABLED`, `details.fiscalYearId`,
  message traduit) ; messages `monoCurrencyFiscalYear` et `laterFiscalYear` retirés ; assistant *Conversion* : « exercice
  ouvert » au lieu de « multi-devises ouvert » ; assistant TVA : règle par société ; cycle de la conversion avec la
  bascule `force` retiré ; scénario E2E R3 retiré ; manuel. FR-1, FR-2, FR-3, FR-5, FR-10, NFR-1 réécrites. Décompte :
  10 FR, 6 NFR (inchangé).
- 2026-11-16 — validation CR-7 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW ; remédiés (D-CR7-1 appliquée : `create_in_tx_inner` ne prend plus `companies`). MEDIUM : 37-3b-CR7-P1-1 (AC-2 bis, verrous : « le `companies` en partage que `create_in_tx_inner` reprend … est alors déjà tenu » prémisse périmée par D-CR7-1 — la route de conversion garde seule sa lecture de `companies`, sans rien à rendre compatible). LOW : -2 (AC-2, `editRefusalOutcome` : `MULTI_CURRENCY_DISABLED` « réglage retiré entre le chargement et l'envoi », résidu de la désactivation — réécrit sur l'exercice mono-devise) ; -3 (AC-1 et AC-2 bis, tests : l'exercice mono-devise d'une société multi-devises n'a aucune route qui le fabrique — fixture posée en SQL, écrite). Résidus CR-7 : aucun autre (`currency-mode`, `force`, `MULTI_CURRENCY_IN_USE`, `FISCAL_YEAR_*`, `laterFiscalYear`, `monoCurrencyFiscalYear`, `fiscalYears`) hors Change Logs datés. Aucun inter-fiches propre à 37-3b ; inter-fiches de la passe 37-3a signalés au rapport (37-1, 37-5, epic).
- 2026-11-16 — validation CR-7 P2 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 0 LOW ; remédiés. MEDIUM : 37-3b-CR7-P2-1
  (AC-1 et AC-2 bis, tests : « aucune route ne le fabrique : posé … en SQL », écrit par la remédiation P1 (-3), est
  faux au code — `POST /fiscal-years/{id}/close`, l'activation de 37-1 et `POST /fiscal-years/{id}/reopen`
  (`routes/fiscal_years.rs:10-11`) fabriquent un exercice mono-devise d'une société multi-devises ; la fixture passe
  par les routes) ; 37-3b-CR7-P2-2 (AC-2 bis, verrous : « `create_in_tx_inner` ne prend plus aucun verrou sur
  `companies` : rien à reprendre ni à tenir compatible », écrit par la remédiation P1 (-1), est faux — son étape 0
  prend la sentinelle `companies` `FOR UPDATE` dès qu'une ligne porte un projet (`journal_entries.rs:308` →
  `projects.rs:101` → `bank_accounts.rs:1729`) ; la route, qui tient déjà le partagé, monte de S à X : deux
  conversions concurrentes portant chacune un projet s'interbloquent ; écrit, défendu par le rejeu de la route
  `Rejouee`, argument au Change Log du développement ; lecture partagée gardée, comme la réévaluation). D-CR7-2 : sans
  effet propre sur 37-3b (aucune mention de la limite). Axes exercés : diff P1 (882d865b..e99364d2) phrase par phrase
  contre le code (`fiscal_years` routes et dépôt, `validate_taggable_in_tx`, `acquire_company_sentinel_lock`) ;
  résidus du modèle réversible (`currency-mode`, `force`, bascule, désactivation, `laterFiscalYear`,
  `monoCurrencyFiscalYear`, masquage par exercice, limite du trou) hors Change Logs datés ; KISS (aucun mécanisme
  restant propre à la réversibilité trouvé). Non exercés : AC-3 à AC-6 hors grep, calculs de l'assistant TVA et de la
  conversion, i18n, E2E, compilation. Inter-fiches : 37-3a (CR7-P2, D-CR7-2 cité) ; aucun autre.
- 2026-11-16 — validation CR-7 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW ; aucune modification. Vérifié au code : `journal_entries.rs:308` (`validate_taggable_in_tx` sous `enforce_project_taggable`) → `projects.rs:101` (`acquire_company_sentinel_lock`) → `bank_accounts.rs:1729` (`companies` `FOR UPDATE`) : cycle S → X de la conversion avec projet exact, défendu par le rejeu (`Rejouee`) ; fixtures par les routes (création, clôture, activation, réouverture) cohérentes avec 37-1 (exercice clos à l'activation, mono-devise même rouvert). Résidus du modèle réversible : aucun hors Change Logs datés. Non exercés : reste de la fiche hors diff, compilation, rendu. Inter-fiches : 37-5 (CR7-P3-1, renvoi au cas de l'AC-2 bis).
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : PDF du manuel non commité, `.tex` seulement (2 remplacements) ; CHANGELOG remplacé par la section *Note de version* (4 remplacements, section ajoutée) ; compteurs « recomptés depuis la source après chaque reprise de `main` » (3 ajouts). Fond des AC inchangé.
- 2026-11-16 — étape 0 (développement, Opus 5.5), base `6f66e775` (pointe de `story/37-3a-ecritures-en-devise-modele`,
  37-1 + 37-2 + 37-3a) : relevé contre 37-2 (`kesh-gate-logs/etape0/37-3b.md`) repris, et les prémisses « attend
  37-3a » relues à cette base, toutes tenues : `ForeignAmount`, `check_foreign_line` (`kesh-core/src/accounting/fx.rs:216`,
  publique), `rate_from_chf` (`:266`), `fx_chf` (`:176`), `fx_line_error` (`balance.rs:19`) ; la fonction d'analyse
  partagée s'appelle `parse_line_amounts` (`routes/journal_entries.rs:704`, `pub(crate)`) ; `prepare_new_journal_entry`
  `:598`, `map_core_error` `:335` ; `create_opening_in_tx` (`repositories/journal_entries.rs:408`, gardée lexicalement
  par `chf_line_lexical.rs`) ; `ModificationBlocker::FxRevaluation` → `ENTRY_IS_FX_REVALUATION` (`kesh-db/src/errors.rs:372`,
  `:384`), rendu juste après la garde d'écriture, avant `PERIOD_LOCKED` (`modification_blocker` `:1441`) ;
  `fx_failed_proposal` (`routes/reconciliation.rs:754` : `MULTI_CURRENCY_DISABLED`, `RATE_UNAVAILABLE` et la table de
  `fx_db_refusal`) ; fixtures `fx_rounding.json`, `fx_rate_from_chf.json`, `fx_opening_dates.json`. Écarts de l'étape 0
  repris au développement : `ModificationBlocker` à `errors.rs:358` (non `:134,210`) ; `CurrencySettingsSection.svelte`
  hors inventaire (exempté, ci-dessous) ; manuel utilisateur en français seul (`docs/manual/{de,en,it}/` ne portent qu'un
  `README.md`) ; `lookup` exige une société multi-devises (le formulaire ne l'appelle qu'en multi-devises).
- 2026-11-16 — développement (Opus 5.5) : AC-1 à AC-6 livrés, détail au *Dev Agent Record* ; statut `review`.
- 2026-11-16 — revue de code P1 (Opus 5.5, couches Blind Hunter backend et frontend, Edge Case Hunter backend et
  frontend, Acceptance Auditor, en parallèle ; diff `story/37-3a-ecritures-en-devise-modele...6e0f39e3`) : **0 CRITICAL,
  3 HIGH, 13 MEDIUM, 22 LOW** après dédoublonnage (5 écartés). Une passe de plus est due (HIGH et MEDIUM, `CLAUDE.md`).
  - Corrigés, avec leurs tests : HIGH CR-P1-1 un CHF saisi refusé (`chfError`) laissait partir le CHF de l'ancien cours
    (formulaire, assistant ; `foreignChf` rend `null`, ligne incomplète ; l'assistant affiche le refus) ; HIGH CR-P1-2 le
    champ CHF se réécrivait au CHF dérivé à chaque frappe (« 9 » → « 9.00 », « 950 » impossible) : la saisie brute reste
    jusqu'à la sortie du champ (`commitChfEntry`, formulaire, assistant, grilles) ; HIGH CR-P1-3 une écriture relue au
    montant normalisé à trois décimales (« 1.234 ») était refusée comme ambiguë et ne s'enregistrait plus
    (`foreignFromResponse`, quatre décimales). MEDIUM : CR-P1-4 frais resté sur un compte d'une autre devise que sa
    colonne ; CR-P1-5 envoi dans les 300 ms d'un changement de date (`debounce.flush`) ; CR-P1-6 l'assistant attend ses
    `lookup` ; CR-P1-7 échec passager du `lookup` non mémoïsé ; CR-P1-8 refus par ligne décalés à la suppression d'une
    ligne ou à l'assistant TVA ; CR-P1-9 frais CHF arrondi en silence (« 12.345 », « 0.004 ») ; CR-P1-10 changement de
    compte de même devise en édition (provenance jugée sur le cours `afc` lu) ; CR-P1-11 un `lookup` réécrivait le
    cours des lignes relues non retouchées ; CR-P1-13 page *Conversion* sans état d'erreur de chargement (motifs faux ;
    deux clés, quatre locales) ; CR-P1-14 refus des grilles d'ouverture indexés par position (par compte désormais) ;
    CR-P1-17, -18, -19 tests qui ne prouvaient pas leur clause (complément mono-devise, refus de la seconde création,
    trop de lignes : témoin et code). LOW : CR-P1-12 erreur d'un frais sur le mauvais frais ; CR-P1-15 champ réellement
    manquant de la paire de la société (test neuf `conversion_paire_de_la_societe_manquante`) ; CR-P1-16 cycles de
    verrous et `EXISTS` non verrouillé écrits ; CR-P1-36 `expectedRate` numérique ; CR-P1-37 Vitest qui acceptait deux
    cours ; CR-P1-38 ligne morte d'un test. Message du motif « frais » précisé (quatre locales).
  - Non traités, une issue chacun (CR-P1-20 à -35, liste au journal `kesh-gate-logs/s37-3b/review-p1/issues-a-ouvrir-p1.md`) :
    conversion de même devise admise par le serveur, différence hors `DECIMAL(19,4)` en 500, `debit`/`credit` « 0 » sur
    une ligne en devise, sélecteur mono-devise, actions `RATE_UNAVAILABLE`/`FX_RATE_NOT_AFC` dans l'assistant, motif
    `FX_VALUE_OUT_OF_RANGE` de l'assistant TVA, envoi muet après un `lookup`, refus des grilles non vidés, noms
    accessibles, lien vers l'assistant sans confirmation, garde de rôle de la page, `details.entry` de la paire (à
    arbitrer), message d'une société mono-devise, tests manquants, duplications, message de longueur en dur.
  - Écartés : `invalid` et `invalid_amount` du complément identiques ; `openingRateKind ?? 'day'` mort (type non
    facultatif) ; idempotence (comme `POST /journal-entries`) ; A33 sans `completeDifference` (voulu, AC-2 bis) ; refus
    de la ligne de différence au-dessus de l'aperçu (la ligne n'est pas un champ).
  - Décompte du *File List* : 82 fichiers, 19 ajoutés (`git diff --name-status 6f66e775..01b3bf0c`), non 81 et 17.
  - Écarts du Dev Agent Record jugés acceptables (Acceptance Auditor) ; défaut du code de la 37-3a : aucun relevé.
  - Gates au commit de la revue : backend complet (`scripts/test-fast.sh --ci`, base `kesh_s37_3b` recréée, migrée,
    seedée) **5748 exécutés, 5748 passés, 13 ignorés** ; `npm run check` 0 erreur (29 avertissements, comme avant),
    `lint-i18n-ownership` PASS, `npm run build` vert ; Vitest complet **4049 passés (257 fichiers)** (`front-vitest-3.log` ; un passage lancé pendant le
    gate backend, `front-vitest-2.log`, rendait cinq délais de 5 s dépassés dans des fichiers hors du diff, verts au passage
    seul). E2E non rejoués.

## Dev Agent Record

### Choix (C-37-3b-*)

- **C-37-3b-1 — `details.entry` de la conversion** : mécanisme commun dans le handler (`conversion_refusal`,
  `routes/journal_entries.rs`) : le refus d'une écriture est porté par `AppError::ConversionEntry { entry, error }` ; la
  réponse de `error` est relue, `details` complété de `entry` (créé `{}` s'il manque ou n'est pas un objet), réécrite
  (statut et en-têtes gardés, `Content-Length` retiré). Un interblocage n'est jamais étiqueté (`AppError::Database` nu)
  pour que `retry_app_on_deadlock` le reconnaisse. Inventaire : toutes les variantes de `ConversionErrorKind` qui visent
  une écriture (`Db`, `Core`, `Validation`, `NoFiscalYear`, `FiscalYearClosed`) reçoivent `entry` ; `TransitUnbalanced`
  et le `FX_ACCOUNT_INVALID` du compte de conversion n'en reçoivent pas.
- **C-37-3b-2 — cœur de la conversion au dépôt** : `repositories/journal_entry_conversions.rs` (`create_conversion`),
  erreur typée `ConversionError { entry, kind }`, la route ne faisant que la forme avant la transaction (libellé, nombre
  de lignes, `parse_line_amounts`, `check_foreign_line`) par `parse_entry_body`, partie de `prepare_new_journal_entry`
  sans `accounting::validate`, extraite et partagée (DRY ; `prepare_new_journal_entry` = `parse_entry_body` + `validate`
  + `draft_to_new_entry`).
- **C-37-3b-3 — paire résolue sur le réglage verrouillé** : `accounts::fx_accounts_for_settings(gain, loss, &Account)` ;
  `fx_accounts_for(&Company, &Account)` n'en est plus que l'enveloppe (aucun choix écrit à côté). Reconnaissance des
  lignes de différence : `accounts::conversion_line_accounts_in_tx` (comptes des lignes de l'écriture à compléter,
  `EXISTS` sur `accounts.fx_gain/loss_account_id` par identifiant, `LOCK IN SHARE MODE`, triés par `id`), la
  désignation par la société se lisant sur `lock_company_currency_in_tx`.
- **C-37-3b-4 — société mono-devise à la conversion** : `409 MULTI_CURRENCY_DISABLED` (sans `fiscalYearId`) avant le
  compte de conversion ; non écrit à la fiche (l'écran n'offre pas l'assistant en mono-devise).
- **C-37-3b-5 — exercice de chaque écriture de la conversion** : lu dans la transaction par `find_covering_date_in_tx`
  au moment de chaque création (ordre chronologique), `NO_FISCAL_YEAR` / `FISCAL_YEAR_CLOSED` étiquetés `details.entry` ;
  la création reverrouille l'exercice `FOR UPDATE`.
- **C-37-3b-6 — journal des deux écritures de l'assistant** : `Banque` (deux mouvements de comptes de trésorerie) ;
  libellés « Conversion EUR → USD (1/2) » / « (2/2) » ; l'assistant porte la ligne de différence qu'il montre et
  n'envoie jamais `completeDifference`.
- **C-37-3b-7 — complément : une colonne vide d'une ligne en devise vaut absente** (`parse_complement_foreign_line`) ;
  bornes de forme de `parse_complement_amount` appliquées au CHF complété (`check_complement_bounds`, extraite).
- **C-37-3b-8 — cache des cours en fermeture** (`createRateLookupCache`) et non en classe : la garde des libellés
  (`i18n-libelle-en-dur.test.ts`) refuse les classes hors liste admise.
- **C-37-3b-9 — provenance affichée à l'ouverture** : « proposé : … » quand la ligne porte le cours de la société
  (source `afc`), « cours saisi » / « cours saisi (ecb) » selon `source`, « manuel » dès que le cours diffère ; l'envoi
  est toujours `manual`.
- **C-37-3b-10 — libellés des codes de lot** (reçu de la 37-3a) : `failedProposalLabel` couvre les huit codes de
  `fx_failed_proposal` (trois clés `reconciliation-failed-*` neuves, cinq messages partagés de `fx-refusals.ts`).

### Inventaires

- **`ComplementLine`** (sans `Default`) : `routes/bank_accounts.rs` `opening_line` (`foreign: None`, motivé : un compte lié
  ne porte pas de devise) ; parseur de la route `parse_complement_lines` (la partie analysée) ; aide de test `line()`
  (`OpeningComplementLineRequest.foreign: None`) ; `tests/opening_complement_repository.rs` 5 sites et
  `tests/opening_complement_bank_scope.rs` 6 sites (`foreign: None`). Propagation : génération — `JournalEntryLineDraft`
  du handler (`foreign` analysé), `NewJournalEntryLine` (recopie du brouillon validé) ; complément —
  `JournalEntryLineDraft` depuis `ComplementLine` (`l.foreign.clone()`), contrepartie sur le report (`None`, CHF),
  `NewJournalEntryLine` (recopie).
- **Tests existants des routes d'ouverture adaptés au 400** : aucun ne testait le 422 d'un champ manquant
  (`grep 422|UNPROCESSABLE` sur `opening_balances_e2e.rs`, `bank_account_opening_balance_e2e.rs` : rien).
- **Sélecteurs (AC-4), inventaire clos tenu par `fx-account-filter-inventory.test.ts`** (fichiers qui importent
  `fetchAccounts`, `AccountAutocomplete` ou `linkableLedgerAccounts`, ou déclarent une prop `AccountResponse[]`) —
  *filtrés* : `AccountAutocomplete` (`excludeForeignCurrency`, défaut `true`) et donc `InvoiceForm`, `products/+page`,
  `ManualMatchModal`, `TransactionSplitModal` (aussi `counterpartyAccounts`) ; `linkableLedgerAccounts` (filtre posé une
  fois, `bank-accounts/format.ts`) et donc `bank-accounts/+page`, `BankAccountJournalLinkForm`, `onboarding/+page` ;
  `counterpartyAccounts` (`reconciliation-counterparties.ts`) ; à l'entrée du composant : `SettleInvoiceDialog`,
  `SupplierInvoiceForm`, `RuleFormModal` ; dans la page : `supplier-invoices/[id]` (compte interne du paiement),
  `settings/invoicing` (`designatable`, valeur en place gardée par `withCurrentAccount` avec la mention « en devise : à
  remplacer ») ; `isAccountUnusable` prend `excludeForeignCurrency` (défaut `true`). *Exemptés, avec leur raison écrite
  dans le fichier* (`fx-filtre-comptes : exempté —`) : `JournalEntryForm`, `VatPurchaseAssistant`, `ConversionAssistant`,
  `journal-entries/conversion/+page`, `journal-entries/+page`, `journal-entries/[id]/+page`,
  `settings/opening-balances/+page` (C-37-5), `CurrencySettingsSection` et `AccountCurrencyFields` (comptes de
  différence et de conversion déjà en CHF par `fxDifferenceAccountOptions` / `fxTransitAccountOptions`), `accounts/+page`,
  `reports/+page`, `OpenItemsScreen`, `RulesList`, `CounterpartyClaimNotice`, `credit-notes/[id]`, `invoices/[id]` et
  `invoices/due-dates` (le règlement filtre dans `SettleInvoiceDialog`), `supplier-invoices/new` et `[id]/edit`
  (`SupplierInvoiceForm` filtre), `reconciliation/rules/+page` (`RuleFormModal` filtre), `ReconciliationProposals`
  (modales filtrées par `counterpartyAccounts`). Aides `.ts` à paramètre `AccountResponse[]` hors garde (vérifié par le test).
- **Cycles de verrous de la conversion (argument écrit, CR-7)** : `companies` puis compte de conversion en partage,
  puis comptes de l'écriture à compléter et sa paire en partage, avant les exercices (`FOR UPDATE` dans
  `create_in_tx`) ; aucun chemin ne prend un exercice puis un compte en exclusif depuis le retrait de la bascule `force`
  de 37-1 : pas de cycle connu. Seule promotion : une ligne à projet fait prendre à `create_in_tx` la sentinelle
  `companies` `FOR UPDATE` alors que la transaction la tient en partage (S → X) ; deux conversions concurrentes à projet
  s'interbloquent, défendues par le rejeu (route `Rejouee`, `retry_app_on_deadlock`). Aucun test de course ajouté.

### Écarts et limites

- **Manuel utilisateur en français seul** (étape 0) ; catalogues i18n dans les quatre locales.
- **Assistant TVA en devise, Vitest du composant** : le `Select` de bits-ui ne s'ouvre pas sous jsdom ; le test du
  composant vérifie le passage en devise (libellé, cours proposé, sélecteur restreint, cours invalide), les lignes
  insérées sont tenues par le Vitest de `foreignPurchaseVatEntry` (vecteurs de la fiche) et par l'E2E (scénario 5).
- **Aller-retour `.keshbackup` d'une ligne d'ouverture en devise** : non écrit ici ; une ligne d'ouverture est une ligne
  d'écriture ordinaire, dont l'aller-retour est couvert par 37-3a (`admin_full_import_e2e.rs`).
- **Mono-devise inchangé** : couvert par les Vitest existants du formulaire (`JournalEntryForm.create/edit.test.ts`) et
  la suite E2E existante, sans test neuf.
- **E2E** : le scénario « exercice mono-devise » est retiré par CR-7 (Vitest et API) ; « Supprimer » d'une réévaluation
  n'a pas d'E2E (la réévaluation est 37-5), Vitest de `canDeleteEntry`.

### Debug Log

- Clippy `explicit_auto_deref` sur `&mut **tx` (conversion) corrigé ; un cas de test « ni cours ni CHF » corrigé (le
  CHF de référence vient aussi de `debit`/`credit`, règle de 37-3a).
- Gardes i18n recomptées depuis la source : `sitesTotal` 3185 → 3292, `CANDIDATES_ATTENDUES` 106 → 112 (ventilation
  écrite aux deux tests) ; garde des dates : `previousDay` de `fx.ts` sans `toISOString`.

### Completion Notes

**Gates, au dernier commit de code `bcf2461e`** (journaux `kesh-gate-logs/s37-3b/`) :

- **Backend complet** (`scripts/test-fast.sh --ci` : fmt, clippy `-D warnings`, nextest profil `ci`, base
  `kesh_s37_3b` remise à zéro, migrée et seedée avant) : **5747 exécutés, 5747 passés, 13 ignorés** (1307 s,
  `gate-final-2.log`). Le premier passage complet (`gate-complet-1.log`, commit précédent) rendait 5746/5747 : seul
  `services_motif::ac12_iii_inventaire_des_sites_non_resolus` (inventaire des handlers de `routes/journal_entries.rs`,
  + `create_conversion` et l'aide `conversion_refusal`), corrigé par `bcf2461e`. Un passage intermédiaire
  (`gate-final.log`, 3186 échecs « No more room in record file ») est **nul** : tmpfs du MariaDB partagé plein, sans
  rapport avec la story ; conteneur redémarré par l'orchestrateur, bases recréées, gate rejoué.
- **Frontend complet** (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) : check 0 erreur, lint PASS,
  **Vitest 4041 passés (257 fichiers)**, build vert (`front-final.log`). Le premier passage (`front-complet-1.log`)
  rendait trois gardes rouges, corrigées par `cdf9926a` : sélecteurs E2E figés sur un libellé traduit (passés en
  `data-testid` : `journal-entry-submit`, `vat-purchase-rate`, `vat-purchase-insert`), bouton rouge de l'assistant
  *Conversion* inscrit à la table fermée (14 → 15), et la garde des sélecteurs de comptes resserrée (une prop
  `AccountResponse[]` ne se déclare que dans un `.svelte` : `conversion.ts` n'en déclare pas).
- **E2E** : `fx-entries.spec.ts` seule, **9/9 passés** (run `-fx2`, avant les sélecteurs en `data-testid`).
  **E2E complet** au commit `bcf2461e` (`LANG=fr_CH.UTF-8`, base `kesh_e2e_s37_3b` recréée, backend et frontend
  construits à ce commit ; `e2e-run-final2.log`, 41,5 min) : **440 passés, 10 échecs, 18 ignorés** ; les neuf de
  `fx-entries.spec.ts` passés (sélecteurs en `data-testid`). Les dix échecs, fichier par fichier, sont tous connus et
  hors story : les **sept de la KF-029 (#97)** (`mode-expert.spec.ts:26`, `:41` ; `onboarding-path-b.spec.ts:65`,
  `:92` ; `onboarding.spec.ts:57`, `:77`, `:150`) ; `backup-restore-new-install.spec.ts:262` et `:381` (délai dépassé
  sur un clic, déjà relevés au registre `issues-a-ouvrir-epic-37.md`) ; `opening-balances.spec.ts:156` (un toast
  recouvre le bouton *Compléter* : le clic échoue sur « element intercepts », relevé depuis la 37-1 au même registre,
  « à confirmer sur `main` »). Un premier E2E complet (`e2e-run-final.log`) a été **arrêté** en cours : tmpfs du
  MariaDB partagé de nouveau plein ; rejoué sur le conteneur redémarré, réservé à cette story.

**Tests ajoutés** (périmètre `6f66e775..HEAD`) : Rust **23** `#[sqlx::test]`
(`journal_entries_fx_ecrans_e2e.rs`) ; Vitest **90** déclarations `it(` ajoutées ; E2E **9** tests.

**Captures** (thème clair, base de démonstration) : onze, `kesh-gate-logs/s37-3b/captures/01…11*.png`.

**PDF** : manuel utilisateur compilé hors dépôt (`kesh-gate-logs/s37-3b/pdf/user-manual.pdf`), section
« Convertir d'une devise à une autre » contrôlée par `pdftotext` ; aucun PDF commité.

### File List

*(Relevé par `git diff --name-status 6f66e775..HEAD` : 81 fichiers, 17 ajoutés, 64 modifiés, la fiche comprise.)*

**Serveur**
- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/lib.rs` (route `POST /journal-entries/conversions`)
- `crates/kesh-api/src/routes/journal_entries.rs`, `routes/opening_balances.rs`, `routes/bank_accounts.rs`
- `crates/kesh-db/src/repositories/journal_entry_conversions.rs` *(neuf)*, `repositories/accounts.rs`,
  `repositories/opening_complement.rs`, `repositories/mod.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`

**Tests serveur**
- `crates/kesh-api/tests/journal_entries_fx_ecrans_e2e.rs` *(neuf, 23 tests)*, `tests/audit_route_registry.rs`,
  `tests/services_motif.rs`
- `crates/kesh-db/tests/opening_complement_repository.rs`, `tests/opening_complement_bank_scope.rs`

**Frontend — neufs**
- `frontend/src/lib/features/journal-entries/` : `fx.ts`, `fx-lines.ts`, `rate-lookup.ts`, `fx-refusals.ts`,
  `fx-display.ts`, `conversion.ts`, `ConversionAssistant.svelte`
- `frontend/src/lib/features/opening-balances/opening-fx.ts`
- `frontend/src/routes/(app)/journal-entries/conversion/+page.svelte`

**Frontend — modifiés**
- `journal-entries/` : `JournalEntryForm.svelte`, `VatPurchaseAssistant.svelte`, `AccountAutocomplete.svelte`,
  `form-helpers.ts`, `vat-purchase.ts`, `blocker-messages.ts`, `journal-entries.api.ts`, `journal-entries.types.ts`
- `accounts/` : `accounts.types.ts`, `account-validity.ts`, `AccountCurrencyFields.svelte`
- `bank-accounts/format.ts`, `invoices/SettleInvoiceDialog.svelte`, `supplier-invoices/SupplierInvoiceForm.svelte`,
  `open-items/OpenItemsScreen.svelte`, `opening-balances/opening-balances.types.ts`,
  `settings/CurrencySettingsSection.svelte`
- `reconciliation/` : `reconciliation-counterparties.ts`, `failed-proposal-label.ts`, `CounterpartyClaimNotice.svelte`,
  `ReconciliationProposals.svelte`, `rules/RuleFormModal.svelte`, `rules/RulesList.svelte`
- routes `(app)/` : `journal-entries/+page`, `journal-entries/[id]/+page`, `settings/opening-balances/+page`,
  `settings/invoicing/+page`, `accounts/+page`, `reports/+page`, `credit-notes/[id]/+page`, `invoices/[id]/+page`,
  `invoices/due-dates/+page`, `reconciliation/rules/+page`, `supplier-invoices/[id]/+page`,
  `supplier-invoices/[id]/edit/+page`, `supplier-invoices/new/+page`

**Tests frontend**
- neufs : `fx.test.ts`, `fx-lines.test.ts`, `fx-support.test.ts`, `JournalEntryForm.fx.test.ts`,
  `ConversionAssistant.test.ts`, `VatPurchaseAssistant.fx.test.ts`, `accounts/fx-account-filter-inventory.test.ts`,
  `frontend/tests/e2e/fx-entries.spec.ts` (9 tests)
- modifiés : `blocker-messages.test.ts`, `failed-proposal-label.test.ts`, `opening-balances-page.test.ts`,
  `shared/i18n-keys.test.ts`, `shared/i18n-libelle-en-dur.test.ts`,
  `components/ui/button/boutons-rouges-inventory.test.ts`

**Documentation** (`.tex` seulement, aucun PDF)
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex`, `docs/api-external.md`

**Artefacts**
- `_bmad-output/implementation-artifacts/37-3b-ecritures-en-devise-ecrans.md`, `sprint-status.yaml`
- 2026-11-16 — revue de code P2 (Sonnet 5.5, couches Blind Hunter, Edge Case Hunter, Acceptance Auditor en parallèle) : 0 CRITICAL, 0 HIGH, 5 MEDIUM (cache : échec passager du `POST /fetch` mémoïsé ; reprise du même compte qui remplaçait cours et CHF d'une ligne relue ; tests de non-régression mono-devise et de `openingProposal` manquants ; refus de conversion en même devise absent du serveur, ex CR-P1-20), une vingtaine de LOW. Corrigés avec tests : les cinq MEDIUM ; conversion en même devise refusée (400, frais exclus de la devise de l'écriture) ; différence calculée au-delà de `DECIMAL(19,4)` en 400 `FX_VALUE_OUT_OF_RANGE` ; sélecteur mono-devise sans compte en devise ; refus de frais vidés à l'ajout ou au retrait d'un frais ; sortie de champ CHF inchangée n'efface plus un refus ; 408 et 429 passagers ; `commitForeignChf` gardé ; guide d'API (route de conversion rejouée) et Note de version. Non corrigés, au registre `issues-a-ouvrir-p1.md` (19 points). Gates (au dernier commit de code) : `scripts/test-fast.sh --ci` 5750 passés, 13 ignorés (fmt, clippy, nextest) ; Vitest 4055 (258 fichiers) ; `npm run check` 0 erreur ; `npm run build` vert ; E2E complet non rejoué ici (clôture).
- 2026-11-16 — revue de code P3 ciblée (Opus 5.5, une lentille, prompt `37-3b-review-prompt-p3-ciblee.md`, sur `9e2e7df6`) : **0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW**. Une passe de plus est due (MEDIUM, et CR-P3-1 touche la production). MEDIUM : CR-P3-1 un échec passager n'étant plus mémoïsé, un `503 RATE_SOURCE_UNAVAILABLE` durable du `POST /fetch` (AFC injoignable) refaisait `lookup` puis `POST /fetch` à chaque frappe du cours ou du CHF (`setForeignRate`, `setForeignChf` → `refreshRates`), dans le formulaire, l'assistant et les grilles : l'échec passager reste mémoïsé `TRANSIENT_RETRY_MS` (15 s), « Récupérer le mois » (`refresh`) réessaie toujours (`rate-lookup.ts` ; tests du délai et du 503 durable, rouges sur `9e2e7df6` ; ferme le point 16 du registre P1) ; CR-P3-4 CR-P2-4 sans test : test de composant écrit (ligne relue `afc` à 0.95, compte repris, cours, CHF et provenance gardés, aucun `lookup` ; rouge sans la garde : cours remplacé par 0.8846). LOW : CR-P3-2 un compte inconnu était lu comme un compte CHF par le refus « même devise », qui masquait `INACTIVE_OR_INVALID_ACCOUNTS` (test `conversion_compte_inconnu_pas_lu_comme_chf`) ; CR-P3-3 borne de la ligne calculée `>=` alignée sur `> FX_CHF_MAX` ; CR-P3-5 guide d'API : refus en même devise et `FX_VALUE_OUT_OF_RANGE` de la ligne calculée (`details.line` = dernière + 1) ; CR-P3-6 en-tête du module (ordre des refus) et commentaire d'`account-validity.ts` périmés ; CR-P3-7 (non corrigé, registre `review-p3/issues-a-ouvrir-p3.md`) : l'assistant efface son refus quand on reprend le même compte. Axes exercés : `SameCurrency` (frais seuls, lignes CHF de transit, ligne de différence portée gains/pertes, compte inconnu), borne de la différence et rang rendu à l'assistant (`conversionField` → `difference`), cache (`get`, `refresh`, appelants par frappe), `selectAccount` inchangé (le rafraîchissement de date passe par `refreshOnDate`, non touché), filtre mono-devise (compte choisi résolu sur la liste complète, aucun compte en devise possible sans le multi-devises), `opening-fx.test.ts`, `ConversionAssistant` (refus gardés, frais), i18n quatre locales (`error-conversion-same-currency` présent), grep des résidus (`excludeForeignCurrency={false}`, `memo.delete`). Non exercés : E2E Playwright, PDF du manuel (aucun `.tex` touché), concurrence. Gates (au commit de la passe) : `scripts/test-fast.sh --ci` 5751 passés, 13 ignorés (fmt, clippy, nextest) ; Vitest 4057 (258 fichiers) ; `npm run check` 0 erreur ; `npm run build` vert ; E2E complet non rejoué.
