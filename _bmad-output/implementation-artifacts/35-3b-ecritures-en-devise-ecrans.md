# Story 35-3b : écritures en devise — soldes d'ouverture, formulaire et écrans

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, C-35-4, C-35-5. Suit **35-3a**
(découpe de l'ancienne 35-3 à la rédaction). **Aucune migration.** Révisée le 2026-11-14 pour la revue de Beat
(CR-2 : R3, R6, R7, R8), qui prime sur le registre antérieur ; elle ajoute une route serveur (assistant *Conversion*,
AC-2 bis).*

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
   - **Cours proposé** (R6, CR-2 — remplace « mois précédent » de CR-1) : cours **effectif** (D9 : cours saisi par la
     société s'il existe, sinon cours AFC ; champ `effectiveRate` de `lookup`, 35-2 AC-5) du **genre d'ouverture de
     l'exercice** de l'écriture d'ouverture (`openingRateKind` du premier exercice, 35-1 AC-1, réglé à l'écran *Cours
     de change*, 35-2 AC-6 ; lu sur `GET /fiscal-years`) : genre `day` (défaut) → **cours journalier de la veille de
     l'ouverture** (`lookup` `kind=day`, `date` = `start_date` − 1 jour : le 31.12 pour un exercice civil, le 30.06
     pour un exercice au 1er juillet, le 31.03 pour un exercice au 1er avril), qui est le cours de clôture de
     l'exercice antérieur ; genre `month` → **cours mensuel du premier mois** de l'exercice (`lookup` `kind=month`,
     `date` = `start_date`). Hors des bornes du `POST /fetch` de 35-2 (date < 2000-01-01, AC-5 : il rendrait 400), le
     `lookup` — qui n'a pas de borne basse et rend alors `404 RATE_UNAVAILABLE` faute de cours — n'est suivi d'**aucun**
     `POST /fetch` : saisie manuelle. Pour un complément daté `TODAY` (`complementDateKind`), cours mensuel du mois de cette date, comme
     avant. Modifiable. Le `404 RATE_UNAVAILABLE` d'un jour (`details.date`) déclenche `POST /fetch { date }` si la
     veille est strictement passée (35-2 AC-5), sinon saisie manuelle. Tests : exercice civil, au 1er juillet, au
     1er avril, en genre `day` et `month`. **Envoyé `rate_source = manual`** sur
     l'ouverture et le complément, proposé ou saisi : le contrôle `FX_RATE_NOT_AFC` de 35-3a (AC-4, 4) compare une
     ligne `afc` au cours **mensuel** du mois de `entry_date`, à la création **et** au `PUT` — un cours journalier, ou
     d'un mois que la date recalculée du complément a quitté, y serait refusé, et l'écriture d'ouverture ne se
     modifierait plus. L'écran affiche la provenance (« proposé : cours journalier du JJ.MM.AAAA » ou « proposé : AFC
     <mois> », ou « cours saisi » quand `overridden` et `effectiveRate` ≠ `afcRate` — un cours saisi égal au cours AFC
     s'affiche comme le cours AFC, comme au formulaire, AC-2) ; le Change Log de la story le dit. Le genre `day`
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
     ouverture **et** complément à lignes EUR et USD acceptés **par leurs routes** (exemption de
     `FX_MULTIPLE_CURRENCIES`, 35-3a AC-4, 2 bis), puis `PUT` du libellé de l'écriture d'ouverture ainsi créée → 200 ;
     aller-retour `.keshbackup` ; **non-régression du mode Compléter bancaire** (34-2, compte CHF inchangé) ; Vitest de
     l'écran et des totaux (`opening-balances-page.test.ts`, `opening-balances-totals.test.ts`), dont : premier
     exercice (ou exercice du complément) mono-devise → aucun compte en devise offert, motif affiché ; refus
     `laterFiscalYear` affiché sur chacune des deux grilles.
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
   - **Champs** : *Montant (USD)*, *Cours* (prérempli par le **cours effectif** que rend `lookup` pour le mois de la
     date, `effectiveRate` ; indicateur « AFC », « cours saisi » (cours de la société, D9) ou « manuel » ; modifié à
     la main → `manual`), *CHF* ; **provenance envoyée** (D9) : `rateSource = 'afc'` si le cours de la ligne **égale**
     `afcRate` (non `null`) **du mois de la date** (`afcMonth`, puce suivante), comparaison en valeur (`Big.eq`, non en graphie), `manual` sinon — un cours saisi par la
     société qui diffère du cours AFC, ou un mois sans cours AFC, part donc `manual` ; aucune autre valeur. Comme à
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
   - **Cours AFC sous le champ** (D9) : quand le `lookup` rend un `afcRate` qui diffère du cours de la ligne, **pour le
     mois de la date courante** (`afcMonth` égal, aucun `lookup` de ce mois en cours — jamais le cours AFC d'un mois
     quitté), le formulaire
     affiche « AFC : <cours> » et un bouton **↑** (« Reprendre le cours AFC ») qui remet **la ligne** au cours AFC
     (`rateSource = 'afc'`, ligne proposée : un changement de date ultérieur la repropose au cours effectif du nouveau
     mois, comme toute ligne proposée) — action locale du formulaire, **sans** `DELETE` du cours saisi, qui se
     retire à l'écran *Cours de change* (35-2 AC-6). Un cours saisi égal au cours AFC : indicateur « AFC »
     (provenance `afc`), pas de ↑.
   - **Appels `lookup`** : mémoïsation par `(devise, mois)` par formulaire, **résultats négatifs compris** ; appel
     seulement pour une date valide dans `dateBounds` et après débounce (`debounce.ts`) ; jeton de requête pour ignorer
     une réponse périmée ; si le nouveau mois n'a pas de cours, le cours **proposé** précédent est **effacé** (jamais
     gardé sous l'étiquette `afc` ni comme proposition) ; en édition, aucune reproposition au montage, seulement au
     changement de date et pour les lignes proposées (`rateProposed`). `GET /lookup` est **en lecture seule** (35-2) et
     ne rend 404 que s'il n'y a ni cours saisi ni cours AFC (un cours saisi seul rend 200, `afcRate: null`, sans
     `POST /fetch`) : sur `404 RATE_UNAVAILABLE`, l'écran appelle **une
     fois** `POST /exchange-rates/fetch` pour ce mois — par la fonction d'API de 35-2 (seule entrée : sans bandeau
     dégradé sur 503, `timeoutMs` de 90 s, 35-2 AC-3 et AC-5), le mois tiré de `details.month` (`AAAA-MM-01`) ; les specs
     E2E pilotent la source factice et s'appuient sur le vidage des tables par `/_test/seed` (35-2 AC-7) — (rôle Comptable ou Administrateur ; mois ≥ 2000-01 et ≤ **mois suivant** le mois courant, **à toute date**, 35-2 AC-5 et R4 amendé, non « ≤ mois courant » : le mois suivant peut être publié avant le 25), puis
     relit ; `404 RATE_NOT_PUBLISHED`, `503` ou rôle Consultation → message, saisie manuelle exigée. Pour un mois
     **postérieur au mois suivant le mois courant** (Europe/Zurich ; 35-2 AC-5 : le mois suivant est admis à toute
     date, non publié → `404 RATE_NOT_PUBLISHED` ; R4 amendé, alignement de l'orchestrateur),
     **aucun** `POST /fetch` (il rendrait 400, 35-2 AC-5) : saisie manuelle directement ; un 400 du `POST /fetch` se
     traite comme `RATE_NOT_PUBLISHED`. Le cours du mois M est publié le 25 de M−1 ou le jour ouvrable suivant selon l'ESTV, mais parfois avant (relevé du 14.11.2026, à confirmer par Beat ; R4 amendé ; 35-2 *Source*), et la tâche
     automatique de 35-2 (une vérification par jour) le récupère pour les exercices multi-devises ; le cas « cours indisponible » vise surtout un
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
     `PUT` d'une écriture ancienne à lignes `afc` sur une installation restaurée, dont la table des cours, système, est
     vide (35-2 AC-7) — → l'écran propose « Récupérer le mois » (`POST /exchange-rates/fetch`, mêmes rôles et même
     règle que ci-dessus) puis le renvoi, ou le passage de la ligne en `manual` ; `FX_RATE_NOT_AFC` → le cours attendu
     (`details.expectedRate`, 35-3a AC-4) est affiché et proposé à la reprise ; `FX_AMOUNT_MISMATCH` porte
     `details.expected` / `details.provided` (35-3a AC-3) ; `FX_CURRENCY_MISMATCH`, `ACCOUNT_REQUIRES_CURRENCY`,
     `FX_AMOUNT_MISMATCH` → message traduit sur la ligne fautive (numéro en `details` : le rang **1-based** de la ligne dans `lines`,
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
     de la société ne devrait pas se produire (elle part `manual`) : s'il survient (cours AFC republié entre `lookup`
     et l'envoi), même traitement.
   - **Ligne d'écart** (R8) : écriture déséquilibrée en CHF avec au moins une ligne en devise → bouton « Ajouter la
     différence de change », qui **ajoute une ligne à l'écriture en cours** — il ne crée **aucune** écriture et
     n'envoie rien : une ligne CHF sur le compte de gains (écart au crédit) ou de pertes (au débit) désigné en 35-1,
     que l'utilisateur voit, modifie ou retire avant d'enregistrer ; au-delà de n × 0.005 CHF (n = nombre de lignes en
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
   - **Tests Vitest de la provenance** (D9) : cours saisi ≠ AFC → `manual` et ligne proposée ; cours saisi = AFC →
     `afc` ; mois sans cours AFC (`afcRate: null`) → `manual` ; cours retapé égal à l'AFC → `afc` ; date changée vers un
     mois dont l'AFC diffère, ligne retouchée au cours de l'ancien mois → `manual` (`afcMonth`) ; reproposition des
     seules lignes proposées ; édition non touchée → provenance relue conservée, sans `lookup` au montage ; édition,
     ligne `afc` dont seul le montant change → reste `afc`, sans `lookup` ; édition, cours retapé égal à l'AFC → `lookup`
     attendu puis `afc` ; ↑ (absent tant que le `lookup` du nouveau mois est en cours) ; relecture après `POST /fetch`
     malgré l'entrée négative mémoïsée ; efface le cours proposé d'un mois sans cours.
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
     pertes si le compte de conversion rend plus que CHF₂ et les frais, au crédit du compte de gains sinon (comptes
     de 35-1) ; aucune ligne si elle est nulle ;
   - chaque ligne de frais est en CHF ou dans **la** devise de son écriture (EUR à l'écriture 1, USD à l'écriture 2),
     sur un compte choisi (les comptes en devise d'une autre devise ne sont pas offerts) ; les écritures peuvent donc
     avoir plusieurs lignes.
   **Écran** : depuis *Écritures*, bouton « Conversion de devises » (société multi-devises, rôles qui saisissent des
   écritures) ; page ou dialogue en deux colonnes, cours proposés et « cours OU CHF » comme au formulaire (AC-2 :
   `lookup` du mois de **chaque** date, provenance `afc`/`manual`, bornes, `computeChf` et la fixture partagée), aperçu
   des deux écritures avec la ligne de différence calculée, puis **« Passer les écritures »**. Chaque date doit tomber
   dans un exercice **multi-devises** ouvert (R3), sinon le motif s'affiche et le bouton est désactivé. Le compte
   débité (côté EUR) et le compte crédité (côté USD) sont des comptes en devise de **deux devises différentes** (sinon
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
   exercice multi-devises) ; E2E (AC-5). L'écriture 2 reste modifiable après coup comme toute écriture (le lien entre
   les deux n'est pas gardé en base : limite écrite au manuel).
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
     `JournalEntryForm.svelte:350-355` ; ses deux `AccountAutocomplete` gardent le défaut `excludeForeignCurrency =
     true`, qui suffit — ne pas filtrer dans le formulaire, qui a besoin des comptes en devise).
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
   par `/api/v1/_test/exchange-rates/source`), écriture USD avec ligne d'écart ajoutée par le bouton (R8), conversion
   EUR → USD **par l'assistant** (deux écritures, compte de conversion à 0 après, R7), écriture à deux devises refusée
   (`FX_MULTIPLE_CURRENCIES`) **par l'API** (`authedApiContext`) — le formulaire ne la laisse pas composer (R7) ; à
   l'écran, on vérifie que le sélecteur ne propose pas le compte de l'autre devise, formulaire sans champs en devise dans un exercice mono-devise (R3), saisie du CHF au
   lieu du cours, soldes d'ouverture d'un compte USD au cours journalier de la veille de l'ouverture (genre `day`, R6) (l'E2E repart d'une société vierge par `seedTestState('with-company')`,
   précédent `opening-balances.spec.ts:21, :80` : l'ouverture exige une société sans écriture, `ALREADY_HAS_ENTRIES`),
   mono-devise inchangé ; **cours saisi par la société** (D9) : un cours posé par `PUT /exchange-rates/overrides`
   pour le mois est proposé, affiché « cours saisi », et la ligne part `rateSource: "manual"` (relue telle) ; le même
   cours égal au cours AFC part `afc` ;
   préalables posés par l'API (Administrateur) : mode multi-devises activé **avec les comptes de gain, de perte et de
      conversion désignés** (35-1 AC-3, sans quoi le bouton d'écart n'ajoute rien et l'assistant est désactivé),
      exercices passés en multi-devises par l'activation ; ⚠️ **le scénario R3 (exercice mono-devise) tourne sur son propre
      `seedTestState('with-company')`**, dont l'exercice unique est passé en mono-devise par la bascule (35-1) **avant**
      tout compte ou ligne en devise — le compte USD de ce scénario se crée **après** la bascule (la société restant
      multi-devises, la création l'admet, 35-1), sans quoi le masquage du formulaire n'aurait rien à masquer et la
      bascule s'arrêterait sur `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE` : un exercice mono-devise **postérieur** refuserait toute ligne en devise des
      exercices antérieurs (`laterFiscalYear`, 35-3a AC-4) — donc les scénarios USD, l'ouverture comprise — et la
      bascule vers le mono-devise est refusée tant qu'un compte en devise porte un solde ou une ligne
      (`FISCAL_YEAR_HAS_FOREIGN_BALANCES`, 35-1) ; aucun scénario ne mêle les deux états,
      compte USD (et EUR) créé, cours des mois et jours utiles servis par la source factice — dont la **veille** de
      l'ouverture ; un scénario au moins part
      d'une table des cours vide et exerce le repli de l'écran (`404 RATE_UNAVAILABLE` → `POST /exchange-rates/fetch`
      → relecture), les autres récupèrent leurs mois par `POST /exchange-rates/fetch` en préalable (`lookup` ne lit que
      la table) ;
      captures d'écran des états (thème clair) ; liste des échecs attendus de `docs/testing.md` recontrôlée.
6. **AC-6 — Manuel utilisateur** : saisir une écriture en devise, cours proposé (cours mensuel AFC — moyenne du 25 au
   24 publiée d'avance pour le mois, R4, jamais « cours moyen du mois » — ou cours saisi par la société, et la
   provenance enregistrée qui en découle ; « AFC : <cours> » sous le champ et le bouton ↑, qui ne retire pas le cours
   saisi de la société) et manuel (cours ou CHF), une seule devise par écriture (R7) et la ligne de différence de
   change ajoutée par le bouton (R8), conversion EUR → USD pas à pas par l'assistant et le compte de conversion (R7),
   exercice mono-devise (R3), soldes d'ouverture en devise (« solde repris au cours de clôture » : cours journalier de
   la veille de l'ouverture, ou cours mensuel du premier mois, selon le choix de l'exercice, R6) ; i18n quatre
   locales ; PDF contrôlé. ⚠️ **Garde G4-bis**
   (`crates/kesh-api/tests/textes_coherents.rs`) : tout numéro de compte cité dans le manuel, le guide de démarrage ou
   un catalogue i18n doit exister dans l'un des trois plans livrés, sous son nom — or ni 1090, ni 1021, ni 1022, ni
   6840 n'y figurent (la maquette de l'AC-2 bis est une illustration de la fiche, pas un texte à recopier) : les exemples du
   manuel et des catalogues n'emploient que des numéros livrés (p. ex. 1020 *Banque*, 6940 *Différences d'arrondi*) ou
   parlent de « votre compte de conversion » sans numéro ; contrôle : `cargo test -p kesh-api --test textes_coherents`.
   **Guide d'API** (`docs/api-external.md`) : la ligne « Écritures comptables » de son tableau de routes (l'assistant,
   `POST /journal-entries/conversions`, ouvert aux clés d'API comme `POST /journal-entries`) et le paragraphe de
   `LATER_FISCAL_YEAR_CLOSED` (toute création d'écriture, la conversion comprise) ; les champs `foreign` sont ceux de
   35-3a.

## Tâches

- [ ] Soldes d'ouverture : serveur (DTO, `ComplementLine`, statut), écran, totaux, tests (AC-1).
- [ ] `fx.ts` (calcul CHF, cours depuis CHF), formulaire, `balance.ts`, `form-helpers.ts`, détail, liste, Vitest sur la
      fixture partagée (AC-2, AC-3).
- [ ] Sélecteurs, prédicat commun, garde d'inventaire (AC-4).
- [ ] E2E, captures, manuel, i18n, CHANGELOG (AC-5, AC-6).
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
