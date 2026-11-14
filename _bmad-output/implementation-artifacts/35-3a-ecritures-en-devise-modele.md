# Story 35-3a : écritures en devise — modèle, validation, API et garde des chemins automatiques

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, C-35-3, C-35-4, C-35-5. Suit 35-1
et 35-2. **Migration non breaking** (colonnes `NULL` ajoutées). **Découpée** à la rédaction (règle de splitting
préventif : plus de cinq modules) : 35-3a = le serveur (`kesh-core`, `kesh-db`, `kesh-api`, `kesh-reconciliation`,
`kesh-seed`) et les sélecteurs serveur ; 35-3b = soldes d'ouverture, formulaire et écrans. Révisée le 2026-11-14 pour
la revue de Beat (CR-2 : R1, R3, R7), qui prime sur le registre antérieur ; puis pour sa revue CR-3 (S2 : comptes de
différence propres à un compte en devise, résolus par `fx_accounts_for` de 35-1 ; S6 : `FX_RATE_NOT_AFC` compare au
cours AFC **en vigueur**), qui prime sur CR-2 ; puis pour sa revue 2 (CR-4, **T3** : la ligne **ne stocke pas** la
devise — elle se lit sur le compte, dont la devise est verrouillée dès la première ligne, 35-1), qui primait sur tout
ce qui précède ; puis pour sa revue 3 (CR-5), qui prime sur tout : **U1** — `FX_RATE_NOT_AFC` compare au **cours
retenu par la société** (`exchange_rates` de 35-2, ligne `source = 'afc'` du mois), plus à une table système (AC-4) ;
**U2** — changer le compte d'une ligne au `PUT` : la devise de la ligne suit le compte (AC-7) ; **U5** — A3, A17, A20,
A21 approuvés (AC-4) ; **U6** — débit et crédit restent en **devise principale** (AC-3).*

## Story

En tant que comptable en multi-devises, je veux saisir une écriture dont certaines lignes sont en devise — montant en
USD, cours du mois proposé (le cours mensuel AFC, ou le cours que la société a saisi pour ce mois, D9), montant en
CHF calculé — et, pour une conversion, passer par le compte de conversion en CHF deux écritures d'une seule devise
chacune (R7), l'écart de change dans la seconde, afin que mes comptes en devise portent leur solde en devise et en CHF.

## Critères d'acceptation

1. **AC-1 — Migration** `20261114000003_journal_lines_foreign_currency.sql` : sur `journal_entry_lines`,
   `amount_currency DECIMAL(19,4) NULL`, `exchange_rate DECIMAL(20,10) NULL`,
   `rate_source VARCHAR(12) NULL` — **trois** colonnes, **aucune colonne de devise** (CR-4, T3 : « DB DRY » ; la devise
   d'une ligne est celle de son compte, `accounts.currency` de 35-1, lue par jointure, et verrouillée dès la première
   ligne du compte, 35-1 AC-4 : c'est ce verrou qui garantit qu'une ligne passée garde sa devise) ; contraintes
   **nommées** (C-35-3), chacune tolérante à la ligne CHF (trois `NULL`) :
   - `chk_jel_fx_all_or_none` : les trois `NULL`, ou les trois non `NULL` ;
   - (aucune contrainte de forme de devise : la colonne n'existe pas ; la forme vit sur `accounts`, 35-1 AC-1) ;
   - `chk_jel_fx_rate_source` : `rate_source IN ('afc','manual','revaluation')` (en `BINARY`, sur le modèle de
     `20261110000001`) ;
   - `chk_jel_fx_amount` : `rate_source = 'revaluation'` ⇒ `amount_currency = 0` ; sinon `amount_currency > 0` ;
   - `chk_jel_fx_rate` : `exchange_rate > 0` ;
   - `chk_jel_fx_chf` (filet structurel, C-35-4) : `amount_currency IS NULL OR rate_source = 'revaluation' OR debit + credit =
     ROUND(amount_currency * exchange_rate, 2)` — le `ROUND` de MariaDB sur `DECIMAL` arrondit l'équidistant loin de
     zéro, comme `MidpointAwayFromZero` ; il couvre les chemins qui n'appellent pas `validate()` (contre-passation,
     restauration `.keshbackup`, 35-5).
   **Filet, pas refus métier** : il n'existe **aucune correspondance par nom de contrainte** (une violation `4025`/`3819`
   devient `DbError::CheckConstraintViolation`, `kesh-db/src/errors.rs:1726`, puis un **400** générique
   `CHECK_CONSTRAINT_VIOLATION`, `kesh-api/src/errors.rs:3801` — jamais une 500). Les `chk_jel_fx_*` s'y appuient sans
   bras dédié ; les refus précis sont ceux de `kesh-core` (AC-3) et du dépôt (AC-4), qui les précèdent. Test, **par
   contrainte** et en SQL brut : trois colonnes partielles, source inconnue, montant 0 hors
   `revaluation`, montant < 0, ligne `revaluation` au montant ≠ 0 (seule `chk_jel_fx_amount` violée), cours 0,
   CHF incohérent → chacune refusée par **sa** contrainte (nom lu dans le message) ;
   la ligne CHF (trois `NULL`) et une ligne `revaluation` valide passent. ⚠️ **Ce que le schéma ne garde plus** :
   aucune contrainte de table ne lie une ligne en devise à un compte en devise (un `CHECK` ne lit pas `accounts`) ;
   une ligne à `amount_currency` non `NULL` sur un compte CHF n'est refusée que par le dépôt (AC-4, `FX_ACCOUNT_IN_CHF`)
   — un `INSERT` brut ou une sauvegarde retouchée pourraient la poser ; les lecteurs (35-4, 35-5) la neutralisent par
   `a.currency IS NULL` (solde en devise `NULL` pour un compte CHF). Limite assumée, écrite au Change Log. ⚠️ Hors `revaluation`, un montant ≤ 0 ou un
   cours 0 viole **aussi** `chk_jel_fx_chf` (`debit + credit > 0` par `chk_jel_debit_credit_exclusive`, squash :863) :
   `chk_jel_fx_chf` est déclarée **en dernier** dans la migration, le cas « cours 0 » se construit sur une ligne
   `revaluation` (montant 0, CHF > 0 : seule `chk_jel_fx_rate` est violée), et le nom rapporté pour les deux cas qui violent aussi
   `chk_jel_fx_chf` (« montant 0 hors `revaluation` » et « montant < 0 ») est constaté au premier essai et écrit au Change Log (ordre d'évaluation des `CHECK` de MariaDB).
   Inscriptions habituelles (sha384, audit, squash, P6 inspecté, P7 sans objet : DDL seul). Colonnes `NULL` : cette
   migration ne referme pas la fenêtre d'import, et 35-2 non plus depuis CR-2 (R1 : la liste close des absences
   admises de 35-1 AC-6 couvre `exchange_rates`, U1, CR-5). Les trois colonnes **s'inscrivent à cette liste** (déjà
   tolérées par la règle générique, la liste les garde non `is_required()`). Ce qui se teste ici : le test d'import
   d'une sauvegarde 0.17.6 de 35-1 AC-6, étendu aux trois colonnes retirées des NDJSON de `journal_entry_lines` →
   import 200, lignes aux trois colonnes `NULL` (Tâches).
2. **AC-2 — Modèle Rust.** `ForeignAmount { declared_currency, amount, rate, source }` (`kesh-core` ;
   `declared_currency: Option<String>` est la devise **annoncée** par le client — jamais stockée, T3 : `None` pour un
   client qui ne l'envoie pas et pour les chemins internes, contre-passation et réévaluation de 35-5 ; le dépôt la
   compare à la devise du compte, AC-4 ; `kesh-core` ne la lit pas ; `source` est un enum
   nommé `RateProvenance { Afc, Manual, Revaluation }` — **pas** `RateSource`, déjà le trait de 35-2 dans `kesh-api` ;
   montant et cours en `Decimal`, non en `Money` ; le type dérive `Debug, Clone, PartialEq, Eq, Serialize,
   Deserialize` comme `JournalEntryLineDraft` qui le porte, `RateProvenance` de même) ;
   `NewJournalEntryLine.foreign: Option<ForeignAmount>` **et** `JournalEntryLineDraft.foreign: Option<ForeignAmount>`
   (`balance.rs:72`, le type que lit `validate()`), **sans** `Default` : chaque site de construction est revu
   (inventaire de l'échec de compilation, écrit au Change Log, site par site : `None` motivé ou extension) ;
   `JournalEntryLine` relu avec les **trois** colonnes **et** la devise du compte, `account_currency: Option<String>`
   (`a.currency AS account_currency`, par jointure ; nom au développement), d'où une méthode `foreign()` qui compose
   la vue `{ currency, amount, rate, source }` des réponses (devise = celle du compte) ; `foreign()` rend `None` quand
   `amount_currency` est `NULL` **et** quand `account_currency` est `NULL` alors que `amount_currency` est posé (état
   que l'AC-4 refuse, atteignable seulement par SQL brut ou sauvegarde retouchée : la ligne se lit comme une ligne CHF,
   limite de l'AC-1). `validate_lines_accounts_in_tx` (`:135`, aujourd'hui `Result<(), DbError>`, deux appelants `:428`
   et `:1595`) rend désormais la devise de chaque compte (`Result<HashMap<i64, Option<String>>, DbError>`, nom au
   développement). Sites SQL de
   `journal_entries.rs`, liste close (production ; `mod tests` commence à `:3177`) :
   - `INSERT` : `:505` (`create_in_tx_inner`) et `:1667` (`update_in_tx`) — les deux seuls `INSERT INTO
     journal_entry_lines` de production de tout `crates/` ; ils écrivent les trois colonnes, jamais de devise ;
   - lectures : `LINE_COLUMNS` (`:103`, aujourd'hui sans alias) est **qualifiée** (`jel.…`, plus `a.currency AS
     account_currency`) et s'accompagne d'une constante de source `LINE_FROM` (`journal_entry_lines jel JOIN accounts a
     ON a.id = jel.account_id`, nom au développement) : ses **sept** lectures `SELECT {LINE_COLUMNS} FROM
     journal_entry_lines WHERE entry_id = ? ORDER BY line_order` (`:549`, `:765`, `:975`, `:1630`, `:1708`, `:1975`,
     `:3081`, relevées le 2026-11-14 ; aucune n'est verrouillante — une jointure sous `FOR UPDATE` verrouillerait aussi
     les comptes) deviennent `SELECT {LINE_COLUMNS} FROM {LINE_FROM} WHERE jel.entry_id = ? ORDER BY jel.line_order`
     (DRY : la jointure s'écrit une fois). La garde lexicale `letterings_lexical.rs` (`{LINE_COLUMNS}` dans une
     écriture) n'est pas touchée : ce sont des lectures. Colonnes en dur de `list_all_lines_by_company` (`:2047`,
     source de l'export CSV, `query_as::<_, JournalEntryLine>` : elle projette aussi `account_currency` et joint donc
     `accounts` en plus de `journal_entries`) ; lecture `OriginLine` de la contre-passation (`:2896`, `FOR
     UPDATE` : elle lit les **trois** colonnes, **sans** jointure — la devise suit le compte, inchangé), puis recopie
     `:2917` ;
   - re-sommes d'équilibre (CHF, inchangées) : `:520-537` et celle d'`update_in_tx`.
   - **`is_no_op_change`** (`:1300`) : compare aujourd'hui compte, débit, crédit et projet **seulement** ; sans les
     champs en devise, un `PUT` qui ne change que le cours, la source ou le montant en devise d'une ligne (CHF
     identique) serait jugé **sans effet** — ni écriture, ni `version`, ni audit. Il compare les **trois** champs
     stockés de `foreign` (montant, cours, source ; montants comparés en valeur, non en graphie) — la devise suit le
     compte, déjà comparé, et `declared_currency` n'est pas stockée ; test : un `PUT` qui ne change que `rateSource`
     écrit et audite.
   - **`ModificationBlocker`** (`kesh-db/src/errors.rs:210` ; `modification_blocker`,
     `repositories/journal_entries.rs:1243`, motif d'écran) et
     `update_in_tx` : voir AC-7 (nouveau motif `ENTRY_IS_FX_REVALUATION`, **hors** `ModificationGuard`, que
     `delete_in_tx` partage).
   - **Locks** : voir AC-4 (concurrence).
3. **AC-3 — Validation `kesh-core`** (`accounting::validate`, `balance.rs:150`, ou fonction voisine appelée par lui ;
   cette fonction voisine, qui contrôle **une** ligne en devise seule, est **publique** : 35-3b l'appelle dans
   `parse_complement_lines` avant la transaction du complément, dont le `validate` sous verrou rendrait une erreur en
   `DbError::Invariant`, donc un 500 — alignement de l'orchestrateur sur 35-3b-P2-2),
   sur `JournalEntryLineDraft.foreign`. Pour une ligne en devise, les contrôles FX **suivent** le refus des montants
   négatifs (`EntryNegativeAmount`, `balance.rs:162-163`, inchangé : un débit ou crédit CHF négatif reste refusé sous
   ce code) et **précèdent** la règle d'exclusivité débit/crédit (`balance.rs:170`), dans cet ordre :
   1. échelle et bornes : au plus 4 décimales sur le montant, 10 sur le cours, **zéros terminaux ignorés**
      (`1.50000` est admis : on compte l'échelle après `normalize()`) ; montant ≤ 10¹³, cours ≤ 10⁶ ; **CHF arrondi
      ≤ 10¹³** (la colonne `debit`/`credit` est `DECIMAL(19,4)` : au-delà, c'est une erreur SQL, non un refus) ;
      sinon 400 `FX_VALUE_OUT_OF_RANGE`. ⚠️ Le produit **n'est pas** calculé par `Decimal::checked_mul` : montant (≤ 17
      chiffres) × cours (≤ 17 chiffres) tient sur ≤ 34 chiffres, au-delà des 28 de `Decimal`, dont la
      multiplication **réduit l'échelle en silence** (arrondi) au lieu d'échouer — le résultat pourrait alors différer
      du `ROUND` exact de MariaDB (`chk_jel_fx_chf`) sur un équidistant. Le calcul se fait en **entiers `i128`**
      (montant × 10⁴ et cours × 10¹⁰ : produit ≤ 10³⁴ < 1,7·10³⁸), arrondi loin de zéro au centime, puis converti
      (précaution : un résultat accepté tient dans les 28 chiffres de `Decimal` ; la borne CHF se juge sur le produit
      exact, non sur un produit déjà arrondi par `Decimal`) ;
   2. signe : `amount > 0` et `rate > 0` (hors `revaluation`, ci-dessous) ; sinon 400 `FX_VALUE_OUT_OF_RANGE` (même
      variante `FxValueOutOfRange { line }` que l'étape 1, aucun code de plus ; 35-5 contrôle le signe du cours de la
      requête de son côté, sous `FX_RATE_INVALID`) ; les vecteurs `error` de la fixture comptent un montant ≤ 0 et un
      cours ≤ 0 ;
   3. CHF = ce produit arrondi au centime, `MidpointAwayFromZero` (même résultat que `round_to_centimes`,
      `money.rs:66`, sur les valeurs courantes — la fixture ci-dessous le prouve) ; s'il vaut 0 →
      `FX_LINE_ROUNDS_TO_ZERO` ;
   4. le montant CHF fourni, **`debit + credit`** (la même somme que `chk_jel_fx_chf`), doit **égaler** ce calcul,
      sinon `FX_AMOUNT_MISMATCH`, `details: { line, expected, provided }` (CHF en chaînes décimales à deux
      décimales) ; une ligne qui porte les deux côtés non nuls est
      ensuite refusée par l'exclusivité (`balance.rs:170`).
   - **Erreurs `CoreError`** : nouvelles variantes `FxValueOutOfRange { line }`, `FxLineRoundsToZero { line }`,
     `FxAmountMismatch { line, expected, provided }` (+ `error_code()`, `kesh-core/src/errors.rs:114`). **`line`** est
     le rang **1-based** de la ligne dans `lines` du corps, rendu `details.line` (convention existante :
     `kesh-api/src/errors.rs:2715`, `:4476`) ; il en va de même de **tous** les refus par ligne de cette fiche
     (`ACCOUNT_REQUIRES_CURRENCY`, `FX_ACCOUNT_IN_CHF`, `FX_CURRENCY_MISMATCH`, `FX_RATE_NOT_AFC`, AC-4) : 35-3b affiche le message sur la
     ligne fautive à partir de ce seul champ (35-3b AC-2).
   - **Bornes du cours seul** : le contrôle d'échelle et de borne du cours (≤ 10 décimales après `normalize()`,
     ≤ 10⁶) est la fonction publique **distincte** de `kesh-core` **créée par 35-2** (AC-2, bornes d'un cours seul :
     elle sert d'abord au `PUT` d'un cours saisi, D9), **réutilisée** ici par la fonction par ligne ci-dessus et par
     35-5 (bornes de la requête de réévaluation, « la même fonction », 35-5 AC-4) ; 35-2 et 35-5 rendent
     `FX_RATE_INVALID`, cette fiche `FX_VALUE_OUT_OF_RANGE` (aucun code de plus) : la fonction rend donc une erreur
     **neutre** (type propre, non une variante `CoreError` déjà liée à un code), que chaque appelant traduit — ici en
     `FxValueOutOfRange { line }` (inter-fiche, 35-3a-CR1-P1-IF1). Elle contrôle aussi le signe du cours (décimal > 0,
     35-2 AC-2) : pour une ligne, un cours ≤ 0 est donc refusé dès l'étape 1, sous le même code
     `FX_VALUE_OUT_OF_RANGE` que l'étape 2 — l'ordre des refus observable ne change pas (même code, même `line`) ; le
     signe du **montant** reste à l'étape 2, et l'exemption `revaluation` (montant 0) n'y touche pas. ⛔ `map_core_error`
     (`routes/journal_entries.rs:265`) rend **tout** variant inconnu en `Validation` générique
     (`other => … "Erreur métier"` `:288`) : sans un bras par variante, le code et les `details` n'atteindraient jamais
     le client. Un bras par variante, avec son code et ses `details`, et un test par code au niveau HTTP.
   - **`rate_source = 'revaluation'`** : exemptée de l'égalité CHF = round(montant × cours) et de
     `FX_LINE_ROUNDS_TO_ZERO` ; exige `amount = 0`, `rate > 0` et un CHF > 0. Refusée par l'API (AC-6), admise pour
     35-5 et pour la contre-passation.
   - débit et crédit restent en **devise principale** (aujourd'hui CHF ; U6, CR-5) : l'équilibre reste Σ débit =
     Σ crédit en devise principale, **strict** ; déséquilibre → `EntryUnbalanced` avec l'écart.
   - **Vecteurs d'arrondi partagés** : `crates/kesh-core/tests/fixtures/fx_rounding.json` (créé ici ; tableau de
     `{ amount, rate, chf }` en chaînes décimales, ou `{ amount, rate, error: "FX_VALUE_OUT_OF_RANGE" }`), avec au moins
     les cas `x.xx5` (équidistant, arrondi loin de zéro), JPY (cours par unité < 0.01), cours à 10
     décimales, montant à 15 chiffres significatifs, un produit exact de 28 chiffres accepté (CHF ≤ 10¹³ : 14 chiffres
     entiers + 14 décimales ; un produit **accepté** ne peut pas dépasser 28 chiffres), des vecteurs `error` dont le
     produit exact dépasse 28 chiffres (aux bornes 10¹³ × 10⁶ avec décimales pleines : refusés par la borne CHF), un CHF
     > 10¹³, et hors bornes ; un test Rust le lit et l'applique à
     `kesh-core` ; **35-3b le lit dans Vitest** pour `big.js` (même fichier, aucune copie).
4. **AC-4 — Contrôle central du dépôt** (`create_in_tx_inner`, `update_in_tx` — les deux seuls chemins d'`INSERT`).
   **Ordre des refus** (une seule raison, la plus bloquante d'abord ; la devise d'un compte ne se lit qu'après le
   contrôle d'appartenance, garde KF-002) :
   1. `InactiveOrInvalidAccounts` → `AccountsNotPostable` (existants, `validate_lines_accounts_in_tx` :135) ; la
      même lecture rend la **devise de chaque compte** (`currency` ajouté à son `SELECT`, verrouillant, ci-dessous) :
      c'est la **seule** source de la devise des lignes pour tous les rangs suivants (T3, CR-4) ;
   2. ligne en devise et société **ou exercice de l'écriture** en mono-devise → **409** `MULTI_CURRENCY_DISABLED`
      (variante de 35-1 ; R3 : le mode se juge sur l'exercice de l'écriture, `fiscal_years.multi_currency` de 35-1,
      lu dans la ligne que l'étape 1 de `create_in_tx_inner` prend déjà `FOR UPDATE`, `journal_entries.rs:332-336`
      — colonne ajoutée à ce `SELECT` —, et à l'étape 1-ter d'`update_in_tx` ; `details: { fiscalYearId }` quand
      c'est l'exercice, `{}` quand c'est la société). **Garde de l'exercice antérieur (R3)** : une écriture qui porte
      une ligne en devise et dont la date précède le début d'un exercice **mono-devise** de la société → même refus,
      `details: { fiscalYearId: <l'exercice mono-devise>, reason: "laterFiscalYear" }` — sans quoi le solde d'ouverture
      de cet exercice, contrôlé nul à sa bascule (35-1 AC-3 bis), changerait après coup. Lecture
      `SELECT id FROM fiscal_years WHERE company_id = ? AND start_date > ? AND multi_currency = FALSE ORDER BY
      start_date LIMIT 1 LOCK IN SHARE MODE`, après le verrou de l'exercice de l'écriture (ordre chronologique, celui de
      la bascule de 35-1, qui verrouille les antérieurs avant l'exercice : les deux se sérialisent) ; la **même** garde
      s'applique à `delete_in_tx` pour une écriture dont les lignes **lues sous le verrou** portent une devise
      (supprimer change aussi ce solde ; rang : après l'étape 3-ter, avant le verrou de période 3-quater) et,
      par `create_in_tx_inner`, à la réévaluation de 35-5 et à la contre-passation. **Au `PUT`, la garde se juge sur
      les lignes d'avant ET sur celles du corps** : retirer ou changer une ligne en devise (la passer sur un compte CHF,
      la supprimer, en changer le montant) modifie le solde d'ouverture de l'exercice mono-devise tout autant que d'en
      ajouter une — une garde qui ne regarde que le corps laisserait faire. Les lignes d'avant (`before_lines`, lues
      aujourd'hui à l'étape 8, `:1629`) se lisent donc plus tôt, sous le verrou de l'écriture, à l'étape 6 (après la
      garde des comptes). **Critère au `PUT`** : la garde ne refuse que si les lignes en devise **changent** — l'ensemble
      des lignes d'avant qui portent une devise diffère de celle du corps (sans égard à l'ordre, même appariement que
      le point 4 ci-dessous ; compte, débit, crédit, montant en devise et cours comparés comme `is_no_op_change`,
      montants en valeur — la devise suit le compte, déjà comparé (T3) —, **sauf `rate_source`**, que la
      garde ne compare pas : la provenance d'un cours ne change aucun solde, et un `PUT` qui ne fait que la changer
      passe). Un `PUT` qui ne touche que le
      libellé, la date dans l'exercice ou des lignes CHF ne change pas le solde d'ouverture de l'exercice mono-devise
      et passe ; un `PUT` sans effet passe donc aussi, sans exception à KF-004 (toutes les gardes **précèdent** le
      court-circuit de l'étape 8, `:1636` ; celle-ci ne refuse simplement pas ce qui ne change rien). Le message dit
      quoi faire (passer l'exercice postérieur en multi-devises) ;
   2 bis. **une seule devise étrangère par écriture (R7)**, jugée sur les **devises des comptes** des lignes (T3,
      CR-4 : l'ensemble des `accounts.currency` non `NULL` lus au rang 1 — aucune lecture de plus, la ligne ne portant
      pas de devise) : deux devises étrangères différentes → 400 **`FX_MULTIPLE_CURRENCIES`** (code neuf,
      `details: { currencies: ["EUR", "USD"] }`, triées ; variante `DbError::FxMultipleCurrencies`, bras dans
      `errors.rs` et dans les correspondances `FailedProposal` de l'AC-5), rendu à ce rang de l'ordre des refus.
      **Cas admis** (T3) : **CHF–CHF** ; **devise–CHF** (et l'inverse), la ligne en devise avec son cours ;
      **devise–même devise** (p. ex. EUR–EUR, deux comptes EUR : chaque ligne a sa valeur CHF à son cours — cours AFC
      du mois ou cours saisi dans l'écriture, `afc` ou `manual` ligne par ligne). **Refusé** : EUR–USD (deux comptes de
      devises différentes), même si l'une des lignes ne porte pas `foreign` (le refus de ce rang précède
      `ACCOUNT_REQUIRES_CURRENCY`, rang 3 ; précédence testée). Une conversion entre
      deux devises passe par le compte de conversion de 35-1 en deux écritures (35-3b, assistant *Conversion*).
      Au **`PUT`** (`update_in_tx`), la règle ne refuse que si le corps porte plus d'une devise **et** une
      devise que l'écriture stockée ne portait pas (l'ensemble des devises ne s'élargit jamais ; alignement de
      l'orchestrateur sur 35-3b-CR2-P2-9) — jugé sur les lignes d'avant lues à l'étape 6 (rang 2 ci-dessus ; leur
      devise est `account_currency`, lue par la jointure de `LINE_FROM`, AC-2), seule lecture de plus : une écriture d'ouverture ou de complément déjà à plusieurs devises
      reste modifiable (alignement de l'orchestrateur sur 35-3b-CR2-P1-2). **Comportement assumé**, écrit au Change
      Log : une ouverture enregistrée avec **une** seule devise ne reçoit pas de seconde devise par `PUT` (refus
      `FX_MULTIPLE_CURRENCIES`) ; le solde d'ouverture d'un compte d'une autre devise passe par le complément (35-3b).
      Tests au dépôt : `PUT` du seul libellé d'une ouverture à deux devises → passe ; `PUT` qui ajoute une seconde
      devise à une écriture à une devise → 400 `FX_MULTIPLE_CURRENCIES` ; `PUT` qui ajoute GBP à une ouverture EUR + USD → 400
      `FX_MULTIPLE_CURRENCIES`.
      **Exemptées** : l'écriture d'ouverture et son complément (35-3b AC-1), qui reprennent des soldes de plusieurs
      comptes en devise et ne se rapprochent d'aucun relevé (le motif de R7) — mécanisme : un paramètre **privé** de
      `create_in_tx_inner` (énumération, p. ex. `PorteeDevises::UneSeule | PorteeDevises::Ouverture`) ; les deux
      chemins d'ouverture appellent aujourd'hui le `create_in_tx` public (`create_opening_entry`,
      `journal_entries.rs:727` ; complément, `opening_complement.rs:719`, autre module) : ils passent par une variante
      **`pub(crate)`** (nom au développement) qui pose `Ouverture`, et ce sont ses deux seuls appelants (garde lexicale
      sur le modèle de celle du constructeur de test, Tâches) ; jamais un champ public de `NewJournalEntry` (même raison
      que la contre-passation ci-dessous) ; la contre-passation est exemptée elle aussi (reconnue par
      `reverses_entry_id.is_some()`, comme pour `FX_RATE_NOT_AFC`) : elle recopie une écriture qui a passé ce contrôle,
      ou une ouverture. Exemption de l'ouverture : arbitrage A3, approuvé par Beat (CR-5). ⚠️ **La contre-passation n'est exemptée ni de
      `MULTI_CURRENCY_DISABLED` ni de `laterFiscalYear`** : elle est datée du jour ou du premier jour de l'exercice
      suivant (`ReversalDating`, `reverse_in_tx_inner`), exercice qui peut être mono-devise — contre-passer une
      réévaluation au 1er janvier d'un exercice mono-devise est refusé en 409, le message disant de le passer en
      multi-devises ; comportement assumé, testé (origine en devise, exercice du jour mono-devise ; origine en
      devise, exercice suivant mono-devise) ;
   3. accord de chaque ligne avec la devise de **son compte** (T3, CR-4), première ligne fautive, son rang en
      `details.line` (AC-3), avec, pour une ligne donnée, cet ordre : ligne **sans** `foreign` sur un compte en devise
      → `ACCOUNT_REQUIRES_CURRENCY` ; ligne **avec** `foreign` sur un compte CHF → 400 **`FX_ACCOUNT_IN_CHF`** (code
      neuf, variante `DbError::FxAccountInChf { line }`, bras dans `errors.rs` et dans les correspondances
      `FailedProposal` de l'AC-5, quatre locales ; quel que soit `declared_currency`, `"CHF"` compris) ; ligne avec
      `foreign` dont la devise **annoncée** (`declared_currency`, présente) diffère de celle du compte →
      `FX_CURRENCY_MISMATCH` (`details: { line, expected: <devise du compte>, provided }`) — **seul** cas de ce code
      désormais ; sans devise annoncée, la ligne prend la devise du compte. L'écran l'annonce toujours (35-3b) : c'est
      la garde contre un compte dont la devise a changé entre le chargement du formulaire et l'envoi (possible tant
      qu'il n'a pas de ligne, 35-1 AC-4) ; un client de l'API qui l'omet renonce à cette garde (écrit au manuel, 35-3b). Quand l'exercice de l'écriture est **mono-devise**, `ACCOUNT_REQUIRES_CURRENCY` porte en
      plus `details.reason: "monoCurrencyFiscalYear"` et `fiscalYearId` : l'invite « saisissez le montant en devise »
      y serait fausse (la ligne en devise est refusée au rang 2) — le message dit de passer l'exercice en
      multi-devises (clé i18n propre, quatre locales ; affichage 35-3b, inter-fiche) ;
   4. ligne `rate_source = 'afc'` (U1, CR-5 — remplace la comparaison au cours AFC brut de la table système) : le
      cours doit égaler le **cours retenu par la société** pour (devise **du compte**, lue au rang 1, mois de
      `entry_date`), c'est-à-dire `exchange_rates.rate` de la ligne de la société **dont la `source` est `afc`** (35-2
      AC-1 : copie d'une valeur publiée, ou valeur reprise par ↑), lu dans la transaction par la fonction de lecture du
      dépôt `kesh_db::repositories::exchange_rates::find_company_rate_in_tx(conn, company_id, currency,
      RateKind::Month, 1er du mois)` de 35-2 (`Option<CompanyRate>` : `rate` et `source` ; la source se lit sur la ligne
      rendue, aucune seconde lecture ; pas de second accès ; lecture **sans verrou**, comme celle de la période :
      un ↑ ou un `PUT …/rates` validé pendant la transaction n'est pas vu, la ligne garde alors le cours lu — **limite
      assumée**, sans effet sur les autres lignes, écrite au Change Log). ⛔ **Plus aucun `find_rate_in_tx` sur la table
      système pour ce contrôle** : la dernière valeur publiée (`_kesh_fetched_exchange_rates`) n'y entre jamais — une
      correction de l'AFC que la société n'a pas reprise ne change rien, et une installation restaurée contrôle ses
      lignes contre ses cours restaurés, sans attendre une récupération. **Ligne de la société `manual` ou de toute source autre
      que `afc`** (cours saisi, D9 ; une `source` inconnue, `RateOrigin::Other(String)` de 35-2, p. ex. `ecb`, se traite
      comme `manual`, A30) : une ligne d'écriture à ce cours est `manual` (le client la qualifie ainsi, 35-3b) ; envoyée
      `afc`, elle est refusée `FX_RATE_NOT_AFC`, `expectedRate` à `null` (il n'y a pas de cours `afc` à proposer : la
      ligne se garde en `manual`). Ni le cours journalier (D10). Aucune nouvelle valeur de `rate_source`. Sinon 400
      `FX_RATE_NOT_AFC`, `details: { line, expectedRate }` (`exchange_rates.rate` de la ligne `afc` de la société,
      chaîne décimale sans zéros terminaux, ou `null` si la ligne est `manual` ou d'une autre source, A30 ; clé lue par 35-3b pour proposer le
      cours) ; **aucune** ligne de la société pour ce mois → `DbError::RateUnavailable {currency, month}`,
      rendu sous le **même code et le même statut 404** que le `RATE_UNAVAILABLE` de 35-2, avec la même forme de
     `details` : `{ currency: "USD", month: "AAAA-MM-01" }` (35-2 AC-5). Contrôle à la création
      **et** au `PUT` (date changée comprise). **Le mois de référence est toujours celui de `entry_date`, sans
      exception** : un cours d'un autre mois ou d'un autre genre (p. ex. le cours d'ouverture d'un solde d'ouverture, T4 : cours journalier de la veille de l'ouverture, ou cours mensuel du dernier mois de l'exercice précédent, C-35-9 amendé)
      se saisit `manual`, jamais `afc` (35-3b y est alignée). **Au `PUT`, seules les lignes `afc` nouvelles ou
      modifiées sont contrôlées** (arbitrage A17, S6 : « les lignes déjà passées ne changent jamais ») : si la date de
      l'écriture change, toutes les lignes `afc` du corps le sont ; sinon, une ligne `afc` du corps est **inchangée**
      quand une ligne d'avant (`before_lines`, lues à l'étape 6, rang 2 ci-dessus), non encore appariée, lui est
      identique en compte, débit, crédit et trois champs stockés de `foreign` — montant, cours **et** `rate_source`
      (une ligne `manual` repassée `afc` au même cours est modifiée ; la devise suit le compte, T3) —, montants en
      valeur comme `is_no_op_change` ;
      appariement **sans égard à l'ordre** (déplacer une ligne ne la modifie pas ; le projet n'entre pas dans la
      comparaison). Une ligne inchangée n'est jamais recontrôlée et ne déclenche aucune lecture du cours : après la
      **reprise** par ↑ d'un cours AFC corrigé (35-2 AC-5, U1, U3 : la ligne de la société change de valeur) ou après
      un `PUT …/rates` qui passe le mois en `manual`, le `PUT` du seul libellé, de lignes CHF ou d'un projet passe ; une
      ligne `afc` ajoutée ou modifiée rend `404 RATE_UNAVAILABLE` (aucune ligne de la société pour ce mois — p. ex.
      exercice passé en multi-devises avant toute récupération ; 35-3b propose « Récupérer le mois », qui copie le cours
      publié, ou le passage en `manual`) ou `FX_RATE_NOT_AFC` (ancien cours, `expectedRate` = cours retenu ; 35-3b en
      tire « Reprendre le cours attendu » / « Garder mon cours », passage en `manual`). Une correction de l'AFC **non
      reprise** ne change rien. Depuis U1, une restauration ne vide plus les cours de la société (35-2 AC-7) : le
      contrôle s'y appuie dès l'import. Conséquence assumée, écrite au Change Log : `rate_source = 'afc'` dit que la
      ligne portait le cours `afc` retenu par la société **quand elle a été contrôlée**, non qu'elle porte celui
      d'aujourd'hui (aucun lecteur de 35-4 ni de 35-5 n'en tire autre chose). **Exemptée** : la contre-passation, qui recopie le cours
      d'origine à une autre date — mécanisme : `create_in_tx_inner` la reconnaît à son paramètre existant
      `reverses_entry_id.is_some()` (`:281`), aucun nouvel indicateur dans `NewJournalEntry` (un indicateur public
      permettrait à un appelant de se dispenser du contrôle).
   - **Devise des lignes relues dans la transaction (T3)** : la relecture qui suit l'`INSERT` (`:549`, `:1708`) est une
     lecture non verrouillante sous `REPEATABLE READ`, dont l'instantané peut précéder le verrou des comptes (le
     premier `SELECT` non verrouillant de la transaction, `books_locked_through` `:316-324`, l'ouvre avant) : un
     changement de devise validé entre les deux (possible tant que le compte n'a pas de ligne) y rendrait l'ancienne
     devise, dans la réponse **et** dans l'audit « après ». Les lignes relues prennent donc leur `account_currency` de
     la lecture verrouillante du rang 1 (celle qui a servi aux contrôles), jamais de la jointure ; test au dépôt :
     devise changée par une transaction concurrente entre l'instantané et le verrou → la réponse porte la devise
     contrôlée (ou argument écrit au Change Log).
   - **Concurrence** : la devise des comptes se lit **`LOCK IN SHARE MODE`** — ⛔ **pas `FOR SHARE`**, erreur de syntaxe
     sur MariaDB 10.11 (`opening_complement.rs:39`, `bank_accounts.rs:194`) ; c'est la forme de 35-1 — dans la
     requête de validation des comptes (`validate_lines_accounts_in_tx` :135, qui lit aujourd'hui sans verrou ; ajouter
     `currency` à son `SELECT`, et l'ordre des refus ci-dessus reste celui-là ; son doc-comment « Lecture **sans
     verrou** … choix C21 », :127-128, est récrit) ou une requête dédiée verrouillante.
     `companies.multi_currency_enabled` se lit **sous `LOCK IN SHARE MODE` avant** les comptes (ordre global de 35-1 :
     `companies` → réglages → `accounts`), **seulement si une ligne porte `foreign`** : une écriture sans ligne en
     devise ne prend aucun verrou sur `companies` (`create_in_tx_inner` lit `books_locked_through` **sans** verrou,
     `:316-324`, pour ne pas se bloquer derrière chaque écriture de la société) — les comptes en devise étant
     verrouillés en lecture, une société repassée en mono-devise sans compte en devise n'a rien à refuser.
     **Place du verrou de `companies`** (« avant les comptes » ne suffit pas : l'exercice est pris `FOR UPDATE` avant
     eux, `create_in_tx_inner` étape 1 `:332-336`, et les chemins Pattern 5 prennent `companies` `FOR UPDATE` **puis**
     l'exercice — `invoices.rs:698`, sentinelle puis `create_in_tx` — d'où un cycle exercice → `companies` contre
     `companies` → exercice) : dans `create_in_tx_inner`, **après l'étape 0** (projets) et avant l'exercice (étape 1) ;
     dans `update_in_tx`, **après l'étape 1-bis** (projets ; le `FOR UPDATE` de l'écriture reste le premier acte,
     `:1481-1487`) et avant l'exercice (étape 1-ter). ⛔ **Pas avant les projets** : la validation des projets
     nouveaux prend la sentinelle `companies` **`FOR UPDATE`** (`projects::validate_taggable_in_tx`,
     `projects.rs:101` → `acquire_company_sentinel_lock`, `bank_accounts.rs:1729`) ; un partagé pris avant elle
     ferait monter la transaction de S à X sur la même ligne, et deux saisies concurrentes portant chacune une ligne en
     devise et un projet s'interbloqueraient (chacune tient S et attend X). Après les projets, le partagé est soit
     redondant (sentinelle déjà tenue en X), soit le premier verrou sur `companies` ; l'ordre `companies` → exercice,
     celui de Pattern 5, est gardé. La contre-passation d'une écriture en devise prend l'exercice d'origine avant
     `create_in_tx_inner` : cycle résiduel, défendu par son rejeu (`Rejouee`) — écrit au Change Log. 35-1 prend
     `FOR UPDATE` sur le compte avant de compter ses lignes. Sans cela, un `PUT` de 35-1 validé entre la lecture et
     l'`INSERT` laisserait une ligne CHF sur un compte en devise. **Portée, à écrire au Change Log** : le verrou
     partagé des comptes est pris par **toute** écriture et tout chemin automatique (une ligne CHF sur un compte devenu
     en devise est le cas à fermer), société mono-devise comprise — contrairement à `companies`, il n'est pas
     conditionné par une ligne en devise : à déclarer. Deux cycles, défendus par le rejeu comme celui de la
     contre-passation : le complément d'ouverture prend les comptes `FOR UPDATE` **avant** l'exercice
     (`opening_complement.rs:445-448`), l'écriture ordinaire l'exercice **avant** les comptes — cycle **déjà présent**
     par le partagé de clé étrangère que l'`INSERT` des lignes prend sur `accounts` (doc-comment `:445-447`), que le
     verrou de lecture avance dans la transaction ; et, **nouveau**, un `PUT` (verrou de
     l'écriture, puis `companies` en partagé) contre une dévalidation de facture (`companies` `FOR UPDATE`, puis
     l'écriture, Pattern 5), qui n'existe que pour un corps portant une ligne en devise. Test de concurrence, ou
     argument écrit au Change Log.
   - Chaque refus est une variante de `DbError` avec son `error_code()`, et a son bras dans la correspondance
     `DbError → AppError` de `errors.rs` — sauf `DbError::RateUnavailable {currency, month}`, créée par 35-2 avec son
     bras 404 : 35-3a la réutilise et ne la redéfinit pas.
   - **Bascule d'un exercice (inter-fiche, 35-1 AC-3 bis)** : la 35-1 contrôle le solde cumulé **en CHF** des comptes
     en devise au début de l'exercice et l'existence de lignes ; cette story, qui crée `amount_currency`, **étend la
     même requête** au solde cumulé **en devise** (Σ `amount_currency` au débit − Σ au crédit, toutes les lignes du
     compte — elles sont toutes dans sa devise, T3 ; une ligne `revaluation` compte 0) : un compte à CHF nul mais devise non nulle refuse aussi la bascule
     (`FISCAL_YEAR_HAS_FOREIGN_BALANCES`, `reason: "balance"`). Test au dépôt.
5. **AC-5 — Chemins automatiques.** Factures, avoirs, règlements, factures fournisseur, lots de paiement,
   réconciliation (manuelle, éclatement, règles, propositions), — les soldes d'ouverture et leur complément exceptés
   (étendus par 35-3b ; d'ici là, ils refusent un compte en devise comme les autres) :
   - aucun ne construit de ligne en devise ; s'ils visent un compte en devise, le refus central de l'AC-4 s'applique
     et remonte en erreur claire (`ACCOUNT_REQUIRES_CURRENCY`, traduite) — ou `FX_MULTIPLE_CURRENCIES` quand la pièce
     vise deux comptes de devises étrangères différentes (précédence du rang 2 bis, T3 : la devise se lit sur les
     comptes, non sur des lignes) ; `FX_ACCOUNT_IN_CHF` et `FX_CURRENCY_MISMATCH` n'en sortent jamais (aucune ligne
     `foreign`) ;
   - **lots (`accept_batch`)** : chaque nouvelle variante a son bras dans **chaque** correspondance
     `DbError → FailedProposal` de `crates/kesh-api/src/routes/reconciliation.rs` — `project_error_to_failed_proposal`
     (`:743`, dont le repli `_ => "DATABASE_ERROR"` `:801-803` est exactement ce qu'il faut éviter : précédent
     `PeriodLocked`, commentaire `:773-780`), `supplier_payment_error_to_failed_proposal` (`:3150-3235`, bras
     `InactiveOrInvalidAccounts` `:3169`, repli `other => "DATABASE_ERROR"` `:3229-3233`), la correspondance
     **en ligne** d'`accept_one_rule` (`:4260-4303` : `if let` sur `PeriodLocked` et `LaterFiscalYearClosed`, puis
     repli `DATABASE_ERROR`) et `:675-690` (`AccountsNotPostable`) — **quatre** sites ; un constructeur commun des
     refus de devise (sur le modèle de `period_locked_failed_proposal` `:603`) évite de les écrire quatre fois ;
     inventaire des `match` au Change Log. Test : un lot dont une proposition vise un compte en devise rend **200**, un `failed[]` avec
     `ACCOUNT_REQUIRES_CURRENCY`, les autres propositions acceptées (règle *Pattern batch*) ;
   - **configurations** — refus des deux côtés, **propriété partagée sans recouvrement** : poser une devise sur un
     compte désigné, lié ou porteur d'un rôle → 409 `ACCOUNT_CURRENCY_IN_USE` (35-1) ; lier ou désigner un compte en
     devise → 400 `ACCOUNT_IN_FOREIGN_CURRENCY`, **posé par 35-1** (AC-4) pour les rôles, les réglages de facturation,
     le compte bancaire lié, l'article et la règle de réconciliation, et **posé ici** pour les colonnes de **pièce** :
     `invoice_lines.revenue_account_id`, `credit_note_lines.revenue_account_id`,
     `supplier_invoice_lines.expense_account_id`, `supplier_invoices.settlement_account_id`,
     `invoice_settlements.settlement_account_id` (validation serveur à l'enregistrement de la pièce, avant le contrôle
     central). Un **test d'inventaire** parcourt la liste close ci-dessous : chaque colonne a son test de refus
     (35-1 ou ici). Liste **close** des colonnes qui
     désignent un compte, lue au schéma (`crates/kesh-db/test-schema/0001_schema_squash.sql`) :
     `bank_accounts.journal_account_id` (sans FK) ; les dix `company_invoice_settings.default_{bad_debt, bank_fees,
     discount, payable, receivable, revenue, rounding, vat_decompte, vat_payable, vat_recoverable}_account_id` ;
     `products.default_revenue_account_id` ; `invoice_lines.revenue_account_id` ; `credit_note_lines.revenue_account_id` ;
     `supplier_invoice_lines.expense_account_id` ; `supplier_invoices.settlement_account_id` ;
     `invoice_settlements.settlement_account_id` ; `reconciliation_rules.counterparty_account_id` ; plus
     `companies.fx_gain_account_id` / `fx_loss_account_id` (35-1, en CHF par construction),
     `accounts.fx_gain_account_id` / `fx_loss_account_id` (comptes propres d'un compte en devise, 35-1 AC-4 bis, S2 ;
     en CHF par construction, gardés des deux côtés par 35-1) et les rôles de compte.
   - **Comptes de différence de change (S2, CR-3)** : cette story ne construit aucune ligne d'écart ni de
     réévaluation et ne choisit donc aucun compte de différence (la ligne d'écart du formulaire est une ligne CHF
     ordinaire, C-35-9 ; elle passe ce contrôle central comme toute ligne). Toute résolution **serveur** des comptes
     de gains et de pertes d'un compte en devise — aujourd'hui la seule réévaluation (35-5) ; la route de conversion
     (35-3b) ne revalide que le compte de conversion, sa ligne de différence étant construite par l'assistant — passe
     par `fx_accounts_for` **puis** `validate_fx_accounts_in_tx` (35-1 AC-4 bis : contrôle des comptes désignés à
     l'usage, `FX_ACCOUNT_INVALID`) ; côté écran, la ligne d'écart, l'assistant *Conversion* et l'assistant TVA (35-3b)
     passent par le miroir `fxAccountsFor`, jamais par un choix écrit à côté entre `companies.fx_*` et `accounts.fx_*` ;
     inventaire des sites qui lisent ces colonnes (grep `fx_gain_account_id`, `fx_loss_account_id`) au Change Log, chacun
     classé : résolution (`fx_accounts_for` seule), garde (35-1), création (plan comptable livré et création de compte :
     `NULL`), sérialisation (DTO, CSV, audit, sauvegarde `.keshbackup`) et réinitialisation de la société.
     Le **contrôle central de l'AC-4 reste le filet** : un état hérité ou une colonne oubliée ne produit jamais une
     ligne CHF sur un compte en devise, seulement un refus. Les sélecteurs du frontend masquent ces comptes (35-3b) ;
   - inventaire **clos** des sites, écrit au Change Log (méthode « inventorier les sites non résolus », `CLAUDE.md`).
6. **AC-6 — API des écritures.** `CreateJournalEntryLineRequest` prend `foreign?: { currency?, amount, rate,
   rateSource: "afc"|"manual" }` (`revaluation` refusé ici : réservé à 35-5) ; **`currency` facultatif** (T3, CR-4) :
   absent, la ligne prend la devise de son compte ; présent, il devient `declared_currency` et doit égaler la devise du
   compte (`FX_CURRENCY_MISMATCH`, AC-4, 3) ; il n'est jamais stocké. `debit`/`credit` restent les montants
   CHF ; **typage du DTO** : les champs de `foreign` sont des `String` (`currency` en `Option<String>` ; comme `debit`/`credit`), analysés dans
   le handler — la route reçoit le corps par le `Json` d'axum (`create_journal_entry` `:600`), dont l'échec de
   désérialisation (énumération inconnue, nombre JSON pour un décimal) rend **422**, non 400 ; un `rateSource` typé en
   énumération rendrait donc 422 et non le 400 exigé ci-dessous (test HTTP : `rateSource: "revaluation"` → 400) ;
   réponses (`JournalEntryLineResponse`, détail, liste) exposent `foreign: { currency, amount, rate, rateSource }`
   (`currency` = devise **lue sur le compte**, `account_currency` de l'AC-2, toujours présente dans `foreign` ;
   décimaux en chaînes, comme `debit`/`credit` ; `rateSource` parmi `afc|manual|revaluation`), **omis** pour une ligne
   CHF par `#[serde(skip_serializing_if = "Option::is_none")]` (registre C-35-9 : le JSON HTTP reste identique en
   mono-devise ; ⚠️ contraire à la convention locale du DTO, dont les champs de lettrage sont présents à `null`,
   test `routes/journal_entries.rs:876-881`) — test : le JSON d'une ligne CHF n'a pas de clé `foreign` ; `MAX_LINES_PER_ENTRY` inchangé ;
   erreurs traduites (quatre locales). `MULTI_CURRENCY_DISABLED` est un **409** partout. **Statuts** (une table, un
   test HTTP par code) : **400** `FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO`, `FX_AMOUNT_MISMATCH`,
   `FX_RATE_NOT_AFC`, `FX_CURRENCY_MISMATCH`, `FX_ACCOUNT_IN_CHF` (T3), `FX_MULTIPLE_CURRENCIES` (R7), `ACCOUNT_REQUIRES_CURRENCY`,
   `ACCOUNT_IN_FOREIGN_CURRENCY` ; **404**
   `RATE_UNAVAILABLE` (comme 35-2 et 35-5) ; **409** `MULTI_CURRENCY_DISABLED`, `ENTRY_IS_FX_REVALUATION`,
   `ACCOUNT_CURRENCY_IN_USE` (35-1). `foreign.currency`, quand il est fourni, se compare **tel quel** à la devise du
   compte (aucune normalisation : `usd` → `FX_CURRENCY_MISMATCH`) ; tests HTTP : sans `currency` sur un compte USD →
   201, `foreign.currency` = `USD` dans la réponse ; `currency: "EUR"` sur un compte USD → `FX_CURRENCY_MISMATCH` ;
   `foreign` sur un compte CHF (avec ou sans `currency`) → `FX_ACCOUNT_IN_CHF` ; `rateSource` hors `afc|manual` (dont `revaluation`) → 400
   `Validation` sans code dédié ; message en français et clé i18n d'erreur par code dans les quatre locales
   (`frontend/src/lib/` : codes d'erreur, comme 35-2).
7. **AC-7 — Modification et contre-passation.**
   - le `PUT` (remplacement complet des lignes : `DELETE` `:1646`, `INSERT` `:1667`) prend `foreign` comme le `POST`,
     avec les mêmes contrôles (AC-3, AC-4), `FX_RATE_NOT_AFC` et `RATE_UNAVAILABLE` sur les seules lignes `afc`
     nouvelles ou modifiées (AC-4, 4, A17) ; l'audit avant/après (`entry_snapshot_json`
     `:999`) porte `foreign` (AC-8) ;
   - **changer le compte d'une ligne (U2, CR-5)** : la devise d'une ligne **suit son compte** — elle n'est verrouillée
     qu'au compte (35-1 AC-4), jamais à la ligne, qui ne la stocke pas (T3). Au `PUT`, une ligne dont le compte change
     se juge entièrement sur le **nouveau** compte, avec les mêmes contrôles que le `POST` : compte USD → compte EUR,
     `foreign` en EUR (montant et cours neufs, 35-3b les réinitialise) → passe, la réponse porte `EUR` ; même ligne
     annonçant encore `currency: "USD"` → `FX_CURRENCY_MISMATCH` ; compte en devise → compte CHF **avec** `foreign` →
     `FX_ACCOUNT_IN_CHF`, **sans** `foreign` → passe (la ligne redevient CHF : `amount_currency`, `exchange_rate`,
     `rate_source` à `NULL`) ; compte CHF → compte en devise sans `foreign` → `ACCOUNT_REQUIRES_CURRENCY` ; une ligne au
     compte changé est **modifiée** au sens de l'AC-4 (4) : `afc`, elle est contrôlée contre le cours retenu de la
     **nouvelle** devise ; et la règle d'une seule devise (AC-4, 2 bis, au `PUT`) s'applique : passer **toutes** les
     lignes USD d'une écriture sur des comptes EUR passe (une devise dans le corps), en passer une seule laisse USD et
     EUR → `FX_MULTIPLE_CURRENCIES`. La garde `laterFiscalYear` (AC-4, 2) voit le changement (compte comparé). Le compte
     quitté perd son verrou de devise s'il ne porte plus aucune ligne (`currencyLocked` faux, 35-1 AC-4 : le verrou
     se calcule sur l'existence d'une ligne). Tests au dépôt et à l'API de chacun de ces cas ;
   - une écriture qui porte une ligne `rate_source = 'revaluation'` **n'est pas modifiable** : `PUT` → 409
     `ENTRY_IS_FX_REVALUATION`. Sites : `DbError::EntryIsFxRevaluation` (variante propre, bras dans `errors.rs`), posée
     dans `update_in_tx` en **étape 3-bis** : juste après la garde d'écriture (étape 3) et **avant** la version
     (étape 4 — « le motif réel prime sur « rechargez » »), donc avant la date, les comptes et le verrou de période ; et
     `ModificationBlocker::FxRevaluation` (code d'écran `ENTRY_IS_FX_REVALUATION`, rendu par `modification_blocker`
     `journal_entries.rs:1243` au même rang, juste après `Guard` et avant `PeriodLocked`, donc `modifiable: false` et `modificationBlockedBy` du détail, lu par 35-3b ; la liste de
     « douze valeurs » de son doc-comment passe à treize, `errors.rs:224`, **et** celle du doc-comment du champ
     `modification_blocked_by` du DTO, `routes/journal_entries.rs:173`, avec son énumération). ⛔ **Pas dans `ModificationGuard`** : `delete_in_tx` (étape
     3-ter) la partage, et la **suppression** d'une réévaluation doit rester permise (35-5 AC-3 : une réévaluation se
     corrige par suppression ; sa contre-passation reste permise mais ne corrige l'exercice que si elle est datée dans
     lui — alignement de l'orchestrateur sur 35-5-CR2-P2-IF1) ; test : `DELETE` d'une écriture de réévaluation → 204, `PUT` → 409, précédence testée ;
   - la contre-passation (repository `reverse_in_tx_inner` `:2796`, lecture `:2896`, recopie `:2917` ; handlers
     `routes/journal_entries.rs` `:448` et `:479`), y compris d'une réévaluation, recopie les trois colonnes —
     montant, cours **et** `rate_source` — en inversant les côtés, sur le **même compte** (sa devise est donc celle
     de l'origine : verrouillée depuis la première ligne, T3 ; `declared_currency` à `None`) ; elle est exemptée du
     contrôle `FX_RATE_NOT_AFC`.
8. **AC-8 — Sauvegarde, exports, MCP.**
   - CSV des lignes (`csv_tables.rs` :356) avec les **trois** colonnes de la table — aucune colonne de devise (le CSV
     suit le schéma, que compare le test des en-têtes ; la devise d'une ligne se lit dans le CSV des comptes, 35-1
     AC-6) ; `amount_currency` et `exchange_rate` s'exportent
     à leur échelle de colonne (4 et 10), **jamais** par `fmt_decimal` / `fmt_opt_decimal` (`csv_tables.rs:108`,
     `format!("{d:.2}")`, qui les ramènerait à deux décimales) ; test de sérialisation à 10 décimales ;
   - `entry_snapshot_json` (audit) les inclut **seulement pour une ligne en devise** (clé `foreign` absente sinon),
     sous la forme `{ currency, amount, rate, rateSource }`, `currency` lue sur le compte (`account_currency`, AC-2 : le
     relevé d'audit reste lisible seul ; ce n'est pas une donnée stockée de la ligne) : l'audit d'une écriture CHF
     reste identique octet pour octet (test) ;
   - MCP : `kesh_ecritures` / `kesh_ecriture` les donnent — `crates/kesh-api/src/services/ecritures.rs`
     (`list_by_company_paginated` `:201`, `find_by_id` `:243`) et `crates/kesh-api/src/mcp/outils/ecritures.rs`
     (struct `:126`, construction `:187`) ; la devise y est celle **du compte** (`account_currency`, AC-2 — les deux
     lectures du service passent par `LINE_COLUMNS`/`LINE_FROM`) ; champs **présents à `null`** sur une ligne CHF (la macro `sortie!`,
     `mcp/sortie.rs:97-114`, n'admet aucun attribut de champ et **n'est pas étendue** : choix de l'orchestrateur,
     2026-11-14, aligné sur 35-1 AC-8 ; registre C-35-9), devise du **même type** que le champ `devise` de `kesh_plan_comptable` (35-1 AC-8 :
     `TexteNonFiable` ou type scellé dédié — `Code<E>`, `mcp/sortie.rs:265`, exige une énumération), source
     `Code<RateProvenance>`, cours et montant `Montant`/texte non fiable selon `mcp::sortie` ; si `Montant` porte le
     cours, son doc-comment (`mcp/sortie.rs:205-214`, qui énumère les types décimaux des colonnes, « recompté le
     2026-10-29 ») gagne `DECIMAL(20,10)` ; la page de `kesh_ecritures` (`LIMITE_MAX_MCP = 22`,
     `LIMITE_DEFAUT = 12`, `outils/ecritures.rs:20-22`, doublée au test `mcp_outils_e2e.rs:108,184`) est **recalculée**
     pour le pire cas — lignes en devise comprises — sous `MCP_ENVOI_OCTETS_MAX`, par `tailles_mesurees`
     (`mcp_outils_e2e.rs:1345`), dont la fixture `fixture_tailles` (`:788`) gagne des lignes en devise ; `mcp_gardes_lexicales` et
     `mcp_contrat` verts ;
   - aller-retour `.keshbackup` : l'export est **générique** (`export_table`, colonnes lues de `INFORMATION_SCHEMA`,
     NDJSON, `backup.rs:215-262` — les trois colonnes sortent seules, décimaux par `rust_decimal`) et la restauration
     passe par l'`INSERT` générique (`backup.rs:833`, sous `chk_jel_fx_*`) ; test : cours à 10 décimales et montant à
     4 identiques après l'aller-retour. Le CSV de `csv_tables.rs` ci-dessus est l'**export utilisateur**, pas la
     sauvegarde.
9. **AC-9 — Limite écrite : le lettrage.** Le lettrage manuel sur un compte en devise ne raisonne qu'en CHF : un
   débiteur USD soldé en USD avec un écart de change ne se lettre pas. Limite assumée, écrite au CHANGELOG, renvoyée au
   registre (C-35-8) pour une issue à l'ouverture de la PR.
## Tâches

- [ ] Migration + inscriptions, contraintes nommées (AC-1 ; **aucune** correspondance par nom dans `errors.rs` : le
      filet rend le 400 générique `CHECK_CONSTRAINT_VIOLATION`) ; colonnes inscrites à la liste close des absences
      admises de 35-1 (R1) ; test : la sauvegarde 0.17.6 simulée de 35-1 AC-6, sans les trois colonnes, se réimporte,
      colonnes `NULL` ; test : `chk_jel_fx_chf` refuse une ligne incohérente insérée en SQL brut. **Recette de
      montée** (R1, release) : l'empreinte lit `SELECT * FROM journal_entry_lines` (`scripts/recette-montee.sh:414`,
      `TABLES`) ; les trois colonnes neuves y sont un écart attendu — `journal_entry_lines` s'ajoute au
      `--diff-attendu` que 35-1 AC-6 ouvre déjà aux « tables des autres stories » (`companies`, `accounts`,
      `fiscal_years` y sont nommées) : l'y nommer à la release, avec la note de release.
- [ ] Tests CR-2 (R3, R7), au dépôt et à l'API : écriture à lignes EUR et USD → `FX_MULTIPLE_CURRENCIES`
      (`details.currencies`), une devise et des lignes CHF → passe ; ouverture à deux devises → passe (exemption) par `create_opening_entry`
      (publique, corps construit à la main : seul chemin d'ouverture atteignable de `tests/` en 35-3a, l'AC-5 laissant
      écrans et complément à 35-3b) ; la variante `pub(crate)` (complément compris) se teste **dans `mod tests` de
      `kesh-db`**, `tests/` ne la voit pas ; le complément à deux devises par sa route : 35-3b (inter-fiche) ; même
      corps par `POST /journal-entries` → refusé ; contre-passation d'une ouverture à deux devises → passe ;
      contre-passation d'une écriture en devise vers un exercice mono-devise → 409 ; `PUT` qui retire ou change la
      ligne en devise d'une écriture antérieure à un exercice mono-devise → 409 `laterFiscalYear` (lignes d'avant),
      `PUT` du seul libellé de cette écriture → passe, `PUT` sans effet → passe (lignes en devise inchangées) ;
      saisies concurrentes portant chacune une ligne en devise et un projet nouveau → aucune montée de verrou S → X
      sur `companies` (place du partagé après les projets, AC-4) ; précédence `FX_MULTIPLE_CURRENCIES` (2 bis) avant `ACCOUNT_REQUIRES_CURRENCY` (3) ;
      ligne CHF sur un compte en devise d'un exercice mono-devise → `reason: "monoCurrencyFiscalYear"` ; ligne en devise dans un exercice mono-devise d'une société multi-devises → 409 `MULTI_CURRENCY_DISABLED`
      `details.fiscalYearId` ; ligne en devise datée avant un exercice mono-devise → 409 `reason: "laterFiscalYear"`,
      de même pour la suppression d'une telle écriture et pour une réévaluation (35-5) ; lot de réconciliation : la
      variante a son bras `FailedProposal` ; course bascule (35-1) contre écriture en devise dans l'exercice antérieur
      (sérialisées) ; extension de la requête de bascule au solde en devise.
- [ ] Tests CR-4 (T3), au dépôt et à l'API : la migration n'ajoute **aucune** colonne `currency` à
      `journal_entry_lines` (test de schéma) ; ligne en devise sans `currency` sur un compte USD → passe, relue avec
      `foreign.currency = "USD"` (jointure) ; `currency` annoncée égale → passe, différente → `FX_CURRENCY_MISMATCH`
      (`details.expected`, `provided`, `line`) ; `foreign` sur un compte CHF → `FX_ACCOUNT_IN_CHF` (dont `currency:
      "CHF"`) ; cas admis CHF–CHF, USD–CHF, CHF–USD, EUR–EUR (deux comptes EUR, l'un `afc`, l'autre `manual`) ; EUR–USD
      → `FX_MULTIPLE_CURRENCIES`, y compris quand la ligne du compte USD ne porte pas `foreign` (précédence sur
      `ACCOUNT_REQUIRES_CURRENCY`) ; `FX_RATE_NOT_AFC` lit le cours de la devise **du compte** ; contre-passation d'une
      ligne USD → ligne relue USD sur le même compte ; les sept lectures de `LINE_FROM` rendent la devise (détail,
      liste, `PUT`, suppression, réponse de création) ; audit d'une ligne en devise : `foreign.currency` présent.
- [ ] Tests CR-4 P1 (compléments) : `foreign` sur un compte CHF avec une devise annoncée **différente** (`"EUR"`) →
      `FX_ACCOUNT_IN_CHF` (précédence sur `FX_CURRENCY_MISMATCH`) ; `currency: "usd"` sur un compte USD →
      `FX_CURRENCY_MISMATCH` (aucune normalisation, AC-6) ; lot (`accept_batch`) dont une proposition vise deux comptes de
      devises différentes → 200, `failed[]` en `FX_MULTIPLE_CURRENCIES` ; `kesh_ecriture` et `kesh_ecritures` rendent
      la devise **du compte** (`null` sur une ligne CHF) ; `foreign()` d'une ligne à `amount_currency` posé sur un
      compte CHF (SQL brut) → `None` ; réponse d'une création portant la devise lue au verrou (AC-4, devise des lignes
      relues).
- [ ] Tests de `FX_RATE_NOT_AFC` (D9, U1, AC-4 4), au niveau du dépôt **et** de l'API, contre les lignes
      `exchange_rates` de la société (35-2) : (a) mois à ligne `afc` : ligne d'écriture `afc` au cours retenu → passe ;
      à un autre cours → `FX_RATE_NOT_AFC` (`expectedRate` = cours retenu) ; (b) mois à ligne `manual` (cours saisi) :
      ligne d'écriture `afc` au cours saisi → `FX_RATE_NOT_AFC` (`expectedRate` `null`), en `manual` → passe ; (b bis) mois à ligne de la société de `source` `ecb` (`RateOrigin::Other("ecb")`,
      posée par SQL, A30) : ligne d'écriture `afc` → `FX_RATE_NOT_AFC`, `expectedRate` `null` (traitée comme `manual`), en
      `manual` → passe ; (c) mois
      sans ligne de la société mais avec une valeur publiée dans `_kesh_fetched_exchange_rates` : ligne `afc` → `404
      RATE_UNAVAILABLE` (la valeur publiée n'entre jamais dans le contrôle), ligne `manual` → passe ; (d) un cours
      journalier (ligne `day`) pour la date de l'écriture ne change pas le résultat de (a) à (c) ; (e) **correction de
      l'AFC non reprise** (valeur publiée changée, ligne `afc` de la société inchangée) : ligne `afc` au cours retenu →
      passe ; à la valeur corrigée → `FX_RATE_NOT_AFC` (`expectedRate` = cours retenu) ; (f) **après reprise par ↑**
      (`POST …/rates/reset`, 35-2 AC-5) : une ligne `afc` à l'ancien cours est refusée à la création ; l'écriture passée
      **avant** la reprise garde ses lignes (aucune réécriture) et son `PUT` du seul libellé, de lignes CHF ou d'un projet
      passe, de même avec ses lignes envoyées dans un autre ordre ; son `PUT` qui modifie la ligne `afc` (montant,
      compte, côté, ou date de l'écriture, vers un mois à ligne `afc`) ou en ajoute une à l'ancien cours →
      `FX_RATE_NOT_AFC` (`expectedRate` = nouveau cours) ; la même ligne passée en `manual` à l'ancien cours passe ; une
      ligne `manual` repassée `afc` à l'ancien cours → `FX_RATE_NOT_AFC` ; (g) **deux sociétés** : un même mois retenu à
      des cours différents (A à la valeur corrigée, B à l'ancienne) — chaque ligne `afc` est jugée contre le cours de
      **sa** société ; (h) **restauration** : `.keshbackup` importé sur une installation neuve (table système vide) —
      création d'une ligne `afc` au cours restauré → passe, sans aucune récupération ; `PUT` du seul libellé d'une
      écriture `afc` → passe ; (i) **mois passé en `manual` par `PUT …/rates`** (35-2 AC-5) après une écriture `afc` : son
      `PUT` du seul libellé passe ; une ligne `afc` ajoutée ou modifiée à ce mois → `FX_RATE_NOT_AFC`, `expectedRate`
      `null`.
- [ ] Tests U2 (AC-7, changer le compte d'une ligne au `PUT`), au dépôt et à l'API : USD → EUR avec `foreign` en EUR →
      passe, réponse en `EUR` ; même ligne annonçant `currency: "USD"` → `FX_CURRENCY_MISMATCH` ; devise → CHF avec
      `foreign` → `FX_ACCOUNT_IN_CHF`, sans `foreign` → passe (trois colonnes `NULL`) ; CHF → devise sans `foreign` →
      `ACCOUNT_REQUIRES_CURRENCY` ; ligne `afc` au compte changé contrôlée contre le cours retenu de la nouvelle devise ;
      toutes les lignes USD sur des comptes EUR → passe, une seule → `FX_MULTIPLE_CURRENCIES` ; `laterFiscalYear` vu au
      changement de compte ; compte quitté sans plus de ligne → `currencyLocked` faux.
- [ ] `kesh-core` (AC-2, AC-3) : `ForeignAmount`, `JournalEntryLineDraft.foreign`, ordre des contrôles, bornes et
      produit en entiers `i128` (**jamais** `Decimal::checked_mul`, AC-3) ; fonction publique du contrôle par ligne et
      fonction publique des bornes du cours seul (AC-3) ; fixture `crates/kesh-core/tests/fixtures/fx_rounding.json` et son test Rust ; tests de propriété
      (arrondi) et de chaque refus (`FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO`, `FX_AMOUNT_MISMATCH`,
      exemption `revaluation`).
- [ ] `kesh-db` : entités, colonnes, sites SQL de l'AC-2, contrôle central dans l'ordre de l'AC-4 (`LOCK IN SHARE MODE`,
      jamais `FOR SHARE` ; place du verrou de `companies` de l'AC-4),
      `FX_RATE_NOT_AFC`, contre-passation (AC-2, AC-4, AC-7) ; tests de chacun des codes de l'AC-4, de la concurrence
      (ou argument écrit), de la recopie à la contre-passation (réévaluation comprise), du 409
      `ENTRY_IS_FX_REVALUATION`.
- [ ] Sites de **test** : `NewJournalEntryLine` sans `Default` casse aussi les binaires de `tests/` (près de 200
      littéraux au total, recomptés par `grep -rn "NewJournalEntryLine {"` : `kesh-api/tests`, `kesh-db/tests`,
      `kesh-report/tests`, `mod tests` — 52 dans `journal_entries.rs` seul) ;
      un constructeur de test commun (`kesh_db::test_fixtures::chf_line(account, debit, credit)`, fonction libre et
      non associée au type, sans `foreign` ; `project_id` par `NewJournalEntryLine { project_id, ..chf_line(…) }`) évite de
      toucher chaque littéral à la main ; il ne sert **jamais** en production. `kesh-db` n'a pas de feature de test
      (`test_fixtures` est un module public, `lib.rs:19`) : le constructeur vit dans `kesh_db::test_fixtures` (non sur
      le type), et une garde lexicale (sur le modèle de `mcp_gardes_lexicales`) échoue si un fichier de `src/` hors
      `mod tests` et hors `test_fixtures.rs` l'appelle.
- [ ] Revue site par site des constructeurs de `NewJournalEntryLine` et de `JournalEntryLineDraft`, des
      correspondances `FailedProposal` et des validations de configuration (AC-5) — inventaire au Change Log.
- [ ] API (AC-6), MCP, exports, audit (AC-8) ; test de l'export CSV à 10 décimales.
- [ ] Tests d'API : écriture USD au cours AFC et au cours manuel, `FX_RATE_NOT_AFC`, conversion EUR → USD en deux
      écritures par le compte de conversion (R7 : EUR → conversion, puis conversion → USD avec la ligne d'écart ; le
      compte de conversion revient à 0), refus d'un règlement de facture sur un compte USD, lot de réconciliation en `FailedProposal` (200),
      `MULTI_CURRENCY_DISABLED` en 409.
- [ ] CHANGELOG, avec la limite du lettrage (AC-9) (le manuel et l'E2E vont à 35-3b).

## Notes de développement

- Lignes : `crates/kesh-db/src/entities/journal_entry.rs` (`JournalEntryLine` :151, `NewJournalEntryLine` :203) ;
  `repositories/journal_entries.rs` (`LINE_COLUMNS` :103, `validate_lines_accounts_in_tx` :135, `create_in_tx_inner`
  :281, `INSERT` :505 et :1667, re-sommes :520-537 et dans `update_in_tx` (:1472, étape 10 :1682-1695), `entry_snapshot_json` :999,
  `list_all_lines_by_company` :2047, contre-passation :2558-3122 dont lecture `OriginLine` :2896 et recopie :2917).
  ⚠️ `:5290` est un `INSERT` **de test** (`mod tests` dès `:3177`), non un site de production.
- Validation : `crates/kesh-core/src/accounting/balance.rs` (`JournalEntryLineDraft` :72, `validate` :150, exclusivité
  :170 — appelée seulement par `routes/journal_entries.rs:566`, `routes/opening_balances.rs:448`,
  `opening_complement.rs:699`, d'où le `CHECK` `chk_jel_fx_chf`) ; arrondi `crates/kesh-core/src/types/money.rs`
  (`round_to_centimes` :66). Constructeurs de `JournalEntryLineDraft` : `routes/journal_entries.rs:548`,
  `routes/opening_balances.rs:369`, `opening_complement.rs:669` et :682.
- API : `crates/kesh-api/src/routes/journal_entries.rs` (DTO :68-152, construction du brouillon :515-595).
- Frontend : `frontend/src/lib/features/journal-entries/` (`JournalEntryForm.svelte`, `balance.ts`).
- Constructeurs de `NewJournalEntryLine` en production, recomptés le 2026-11-14 (littéraux hors `mod tests`) :
  **30 dans 11 fichiers** — `repositories/invoice_settlements.rs` 7, `supplier_invoices.rs` 5, `credit_notes.rs` 4,
  `invoices.rs` 4, `opening_complement.rs` 1, `journal_entries.rs` 1 (contre-passation : littéral :2920, dans l'instruction ouverte :2917) ;
  `kesh-reconciliation/src/manual.rs` 2, `split.rs` 2 ; `kesh-seed/src/pieges.rs` 2 ; `routes/journal_entries.rs` 1 ;
  `routes/opening_balances.rs` 1. `repositories/accounts.rs` n'en construit qu'en tests. La liste fait foi **après**
  l'échec de compilation, pas avant.
- Correspondances `DbError → FailedProposal` : `routes/reconciliation.rs` :675-690, :743-803, :3150-3235,
  :4260-4303 (en ligne, `accept_one_rule`).

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Opus) : 0 CRITICAL, 1 HIGH, 9 MEDIUM, 7 LOW ; remédiés.
- 2026-11-14 — validation P2 (Sonnet) : 0 CRITICAL, 2 HIGH, 10 MEDIUM, 7 LOW ; remédiés dans la fiche sauf P2-2b
  (35-3b), P2-10b (35-1), P2-17 (35-3b), P2-18 (35-5), P2-19 (35-1) (inter-fiches). Points : `FOR SHARE` (erreur de
  syntaxe MariaDB 10.11) → `LOCK IN SHARE MODE` ; mois du cours `afc` toujours celui de `entry_date` ; motif
  `ENTRY_IS_FX_REVALUATION` hors `ModificationGuard` (la suppression reste permise) ; `is_no_op_change` ; variantes
  `CoreError` et `map_core_error` ; produit en `i128` et borne CHF ; filet `CHECK` sans correspondance par nom ;
  verrou de `companies` conditionnel ; limite MCP de `kesh_ecritures` ; propriété des refus de configuration ;
  statuts HTTP.
- 2026-11-14 — validation P3 (Opus) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 8 LOW ; remédiés dans la fiche sauf
  35-3a-P3-IF1 (35-3b) et 35-3a-P3-IF2 (35-3b) (inter-fiches). Points : résidus des Tâches (`FOR SHARE`,
  `checked_mul`, correspondance des contraintes) ; quatrième correspondance `FailedProposal` (`accept_one_rule`) ;
  rang de ligne `details.line` sur tous les refus par ligne ; place du verrou de `companies` (cycle avec Pattern 5) ;
  champs `foreign` en `String` (422 d'axum) ; `modification_blocker` à `:1243` ; rang 3-bis de
  `ENTRY_IS_FX_REVALUATION` ; ordre des `CHECK` au test ; bornes du cours seul (35-5) ; constructeur de test gardé ;
  doc-comment de `Montant`.
- 2026-11-14 — validation P4 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW (plus 2 LOW inter-fiches) ; remédiés dans la
  fiche sauf 35-3a-P4-IF1 (35-1) et 35-3a-P4-IF2 (35-3b) (inter-fiches). Points : code du refus de signe (étape 2 de
  l'AC-3, `FX_VALUE_OUT_OF_RANGE`) ; dérivations de `ForeignAmount` ; second doc-comment « douze valeurs »
  (`routes/journal_entries.rs:173`) ; ligne `errors.rs:1726` ; cas de l'AC-1 qui violent aussi `chk_jel_fx_chf`.
- 2026-11-14 — validation P5 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 7 LOW (plus 1 LOW inter-fiches) ; remédiés dans la
  fiche sauf 35-3a-P5-IF1 (35-3b) (inter-fiches). Points : forme et omission (`skip_serializing_if`, C-35-9) de
  `foreign` dans les réponses HTTP (MEDIUM) ; clés des `details` de `FX_AMOUNT_MISMATCH` et `FX_RATE_NOT_AFC`
  (MEDIUM) ; type MCP de la devise (`Code<E>` exige une énumération) ; constructeur de test hors du type ; place des
  contrôles FX après `EntryNegativeAmount` ; cas de test de l'AC-1 (`montant < 0`, `revaluation` au montant ≠ 0) ;
  `PUT` d'une écriture `afc` après restauration ou republication ; doc-comment de `validate_lines_accounts_in_tx` ;
  mesure MCP (`tailles_mesurees`) ; numéros de ligne (:2917, étape 10 d'`update_in_tx`).
- 2026-11-14 — validation P6 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié. Point : la liste des
  constructeurs de production citait `:2917` (le `let reversal_lines`) pour le littéral, qui est à `:2920` (P5 avait
  rapproché le numéro de la mauvaise ligne). Les autres phrases de la remédiation P5 sont vraies au code et cohérentes
  avec le registre C-35-9 et 35-1, 35-2, 35-3b, 35-4.
- 2026-11-14 — révision CR-1 (D9, D10, terminologie ; Opus) : `FX_RATE_NOT_AFC` (AC-4, 4) compare toujours au cours AFC
  mensuel **brut**, lu par `find_rate_in_tx(conn, currency, RateKind::Month, 1er du mois)` (signature de 35-2
  étendue au genre), jamais au cours saisi par la société ni au cours journalier ; aucune nouvelle valeur de
  `rate_source` ; fonction des bornes d'un cours seul désormais créée par 35-2 (AC-3) ; la fenêtre d'import est
  refermée par 35-2, d'où le test de sauvegarde reformulé (AC-1, Tâches) ; récit de la story (cours proposé = cours
  effectif). AC touchés : Story, AC-1, AC-3, AC-4, Tâches.
- 2026-11-14 — validation CR-1 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW ; remédiés dans la fiche sauf
  35-3a-CR1-P1-IF1 (35-2, inter-fiche). Points : tests de `FX_RATE_NOT_AFC` sous D9 (ligne `afc` au cours AFC avec un
  cours saisi présent, mois sans cours AFC, jour) ; fonction des bornes d'un cours seul : erreur neutre et signe
  (35-2) au lieu de l'hypothèse « si la fonction le contrôle aussi » ; « sauvegarde de la version courante » précisée
  (version de l'epic). Sites lus : `balance.rs:72,150,163`, `journal_entries.rs:135,505,1243,1300,1667,2796`,
  `errors.rs:1726`, 35-2 AC-2, AC-5, AC-9 et Tâches ; aucun GET réseau.
- 2026-11-14 — révision CR-2 (revue de Beat R1–R10 ; Opus) : **R7** — une écriture ne porte qu'une devise étrangère :
  refus 400 `FX_MULTIPLE_CURRENCIES` au contrôle central (AC-4, rang 2 bis), bras HTTP et `FailedProposal` ; exemption
  de l'ouverture et de son complément par un paramètre privé de `create_in_tx_inner` posé par une variante
  `pub(crate)` (à confirmer par Beat), et de la contre-passation ; la conversion passe par le compte de conversion en
  deux écritures (Story, Tâches). **R3** — `MULTI_CURRENCY_DISABLED` jugé sur l'exercice de l'écriture
  (`fiscal_years.multi_currency`, lu dans le `FOR UPDATE` de l'étape 1), plus sur la société seule ; garde de
  l'exercice antérieur à un exercice mono-devise (`reason: "laterFiscalYear"`, création, modification, suppression,
  réévaluation) ; extension de la requête de bascule de 35-1 au solde en devise. **R1** — la sauvegarde 0.17.6 se
  réimporte (colonnes à la liste close de 35-1), test reformulé. Prémisses relues au code :
  `journal_entries.rs:242-256`, `:281-289`, `:332-336`, `:655`, `:727`, `:2980` ; `opening_complement.rs:719`.
  AC touchés : en-tête, Story, AC-1, AC-4, AC-6, Tâches. Fiche à revalider (cycle CR-2).
- 2026-11-14 — validation CR-2 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 4 LOW ; remédiés dans la fiche sauf
  35-3a-CR2-P1-IF1 (35-3b : test du complément d'ouverture à deux devises par sa route ; affichage de
  `ACCOUNT_REQUIRES_CURRENCY` `reason: "monoCurrencyFiscalYear"`), inter-fiche. MEDIUM : (1) garde `laterFiscalYear`
  jugée sur le seul corps au `PUT` — retirer ou changer une ligne en devise change autant le solde d'ouverture de
  l'exercice mono-devise (lignes d'avant lues plus tôt que l'étape 8, `:1629`) ; rang de la garde dans `delete_in_tx` ;
  (2) test d'exemption de l'ouverture et du complément inatteignable en 35-3a (l'AC-5 leur fait refuser les comptes
  en devise ; variante `pub(crate)` invisible de `tests/`) ; (3) portée du verrou partagé des comptes (toute écriture,
  société mono-devise comprise) et deux cycles non écrits (complément : comptes puis exercice ; `PUT` contre
  dévalidation). LOW : contre-passation vers un exercice mono-devise (non exemptée, à tester) ;
  `ACCOUNT_REQUIRES_CURRENCY` trompeur en exercice mono-devise ; vecteur « produit de plus de 28 chiffres » de la
  fixture inatteignable comme vecteur accepté et justification de l'`i128` ; tests de précédence. Sites lus :
  `journal_entries.rs:135-175, 281-420, 640-725, 1243-1330, 1470-1600, 1858-1935, 2796-3010`,
  `opening_complement.rs:445-448, 700-719`, `balance.rs:60-200`, `reconciliation.rs` (quatre correspondances
  confirmées), `csv_tables.rs:108`, `mcp/outils/ecritures.rs:20-22`, `mcp/sortie.rs:95-114`, 35-1 AC-3 bis et AC-6,
  35-2 AC-2 et AC-9, 35-5 (IF1 de sa passe CR-2 P1 : la garde `laterFiscalYear` est bien au rang 2 de l'AC-4, clos).
  Axes non exercés : GET de la source journalière (aucune prémisse de cette fiche n'en dépend), réponses HTTP et
  i18n des quatre locales (non relues au code), E2E.
- 2026-11-14 — validation CR-2 P2 (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 5 LOW ; remédiés dans la fiche sauf
  35-3a-CR2-P2-IF1 (35-1 AC-6 : `journal_entry_lines` au `--diff-attendu`) et 35-3a-CR2-P2-IF2 (35-3b AC-2 : texte
  « exemption à la seule création » périmé), inter-fiches. MEDIUM : (1) garde `laterFiscalYear` du `PUT` à l'étape 6
  contre « un `PUT` sans effet n'est pas refusé » — le court-circuit est à l'étape 8 (`:1636`), après toutes les
  gardes (KF-004) : critère précisé (refus seulement si les lignes en devise changent ; libellé seul et no-op passent) ;
  (2) partagé de `companies` « en tête de l'étape 0 » / « juste après l'étape 1 », **avant** la sentinelle
  `companies FOR UPDATE` des projets (`projects.rs:101`, `bank_accounts.rs:1729`) : montée S → X et interblocage entre
  deux saisies à ligne en devise et projet — déplacé après les projets ; (3) site R1 non résolu : l'empreinte de
  `recette-montee.sh` lit `SELECT * FROM journal_entry_lines` (`:414`), les quatre colonnes y sont un écart attendu
  absent du `--diff-attendu` de 35-1. LOW : « corps seul (aucune lecture) » contredit par la règle du `PUT` ; ouverture
  à une devise qui ne reçoit pas de seconde devise par `PUT` (assumé, complément) ; tests proposés par 35-3b-CR2-P1-2
  non reportés ; cycle complément/écriture ordinaire dit « nouveau » alors que le partagé de clé étrangère des lignes
  le formait déjà ; 35-3b périmée (IF2). Sites lus : `journal_entries.rs:103, 281-420, 596-745, 1095-1160, 1300-1320,
  1472-1700, 1858-2000, 2653, 2796-2950`, `projects.rs:90-106`, `bank_accounts.rs:1729`, `errors.rs` (kesh-db
  :150-230, :1726 ; kesh-api :2715, :3801, :4476), `routes/journal_entries.rs:173, 265, 288, 448, 479, 548, 566, 600,
  876`, `opening_complement.rs:440-450, 715-720`, `fiscal_years.rs:160-180, 870`, `recette-montee.sh:414-440` ; 35-1
  AC-3, AC-3 bis, AC-6 ; 35-3b AC-2 et Change Log ; 35-5 (`FX_MULTIPLE_CURRENCIES`, `laterFiscalYear`). Axes non
  exercés : GET de la source journalière (aucune prémisse de cette fiche n'en dépend) ; i18n des quatre locales et
  manuels (35-3b) ; E2E.
- 2026-11-14 — validation CR-2 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW (35-3a-CR2-P3-1 : la phrase
  de la recette de montée disait 35-1 AC-6 « limité » à trois tables, alors que 35-1 l'ouvre aux tables des autres
  stories — remédié). Lu au code : `journal_entries.rs:1636` (court-circuit après toutes les gardes), `:1300`
  (`is_no_op_change`, quatre champs `foreign` à ajouter, dit à l'AC), `:290-300` et `:1496-1525` (projets : la
  sentinelle `FOR UPDATE` n'est prise que si des projets nouveaux existent, `projects.rs:95-101`,
  `bank_accounts.rs:1729` — le partagé après eux est cohérent), `opening_complement.rs:438-452` (cycle
  complément / écriture ordinaire), `recette-montee.sh:414`, 35-5 AC-3 (suppression ; contre-passation datée),
  35-1 AC-6, 35-3b AC-2 (alignée). Axes non exercés : le reste de la fiche, i18n, manuels, E2E ; test de
  concurrence non exécuté (fiche seule).
- 2026-11-14 — révision CR-3 (revue de Beat S1–S6 ; Opus) : **S2** — AC-5 : `accounts.fx_gain_account_id` /
  `fx_loss_account_id` (35-1 AC-4 bis) ajoutées à la liste close des colonnes qui désignent un compte ; puce neuve :
  cette story ne choisit aucun compte de différence, toute résolution serveur (35-5) passe par `fx_accounts_for` puis
  `validate_fx_accounts_in_tx` (35-1), et l'écran (35-3b) par le miroir `fxAccountsFor`, inventaire des sites lecteurs au Change Log du
  développement. **S6** — AC-4 (4) : `FX_RATE_NOT_AFC` compare au cours AFC mensuel **en vigueur** (`rate_per_unit`),
  jamais à une valeur republiée non adoptée ; seule une adoption (35-2 AC-5) fait refuser le `PUT` d'une écriture `afc`
  à l'ancien cours (test : republication en attente → passe ; après adoption → refus). S1, S3, S4, S5 sans point de
  contact ici. Aucune migration ni route neuve. AC touchés : en-tête, AC-4, AC-5. Fiche à revalider (cycle CR-3).
- 2026-11-14 — validation CR-3 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW ; remédiés dans la fiche sauf
  35-3a-CR3-P1-4 (35-2 AC-5 / 35-3b, arbitrage). MEDIUM : (1) les Tâches ne listaient aucun test S6 de
  `FX_RATE_NOT_AFC` (republication non adoptée, adoption) alors que l'AC-4 les déclare « testés » — cas (f) et (g)
  ajoutés. LOW : (2) classes de l'inventaire des sites `fx_*_account_id` incomplètes (création, sauvegarde,
  réinitialisation) ; (3) lecture du cours AFC sans verrou face à une adoption concurrente (35-2 AC-5 les lit `FOR
  UPDATE`) — limite assumée écrite à l'AC-4 ; (4) **non appliqué** : le `PUT` d'une écriture `afc` est refusé pour ses
  lignes **inchangées** après une adoption (35-2 AC-5, 35-3a AC-4 4) alors que S6 dit que les lignes passées ne
  changent jamais ; issue de secours = passer la ligne en `manual` au même cours. Proposition, si Beat veut l'éviter :
  ne contrôler que les lignes `afc` nouvelles ou modifiées au `PUT` (comparaison à `before_lines`, déjà lues à l'étape
  6), en gardant `404 RATE_UNAVAILABLE` pour les seules lignes contrôlées ; touche 35-2 (AC-5, Tâches) et 35-3b.
  Sites lus : diff `294889cc..b6f88c24` ; 35-1 AC-4 bis (`fx_accounts_for`, `validate_fx_accounts_in_tx`, gardes
  inverses) ; 35-2 AC-3 (règle d'écriture S6), AC-5 (adoption, acquittement), Tâches (`find_rate_in_tx`) ; 35-3b
  (`fxAccountsFor`, route de conversion) ; 35-5 (résolution par `fx_accounts_for`). Axes exercés : prémisses de S2 et
  S6 contre 35-1/35-2/35-3b/35-5, tests des AC modifiés, sites SQL/lock. Non exercés : code (fiche seule, aucune ligne
  citée n'a changé), GET de la source journalière (sans prémisse ici), i18n des locales, manuels, MCP (35-1 AC-8 ne
  l'étend pas aux comptes de différence), E2E.
- 2026-11-14 — validation CR-3 P2 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW ; remédiés dans la fiche sauf
  35-3a-CR3-P2-IF1 (35-2 AC-5 et Tâches), inter-fiche. **Arbitrage A17 appliqué** : AC-4 (4) réécrit en texte propre
  (texte antérieur retiré) ; AC-7 et Tâches (g), (h) alignés. MEDIUM : (1) la liste A17 des champs qui font une ligne
  « modifiée » omettait `rate_source` (une ligne `manual` repassée `afc` au même cours aurait échappé au contrôle) et
  le côté — comparaison posée sur compte, débit, crédit et les quatre champs de `foreign` ; (2) tests (g) contraires à
  A17 (« refusée au `PUT`, libellé seul compris ») — réécrits, cas de restauration (h) ajouté. LOW : (3) appariement
  des lignes sans égard à l'ordre (un `PUT` qui réordonne ne modifie aucune ligne), aligné aussi sur le critère de
  `laterFiscalYear` (rang 2) ; (4) AC-7 disait le `PUT` soumis aux « mêmes contrôles, dont `FX_RATE_NOT_AFC` » ;
  (5) sens de `rate_source = 'afc'` après adoption (cours en vigueur au contrôle) écrit, sans lecteur en 35-4/35-5.
  Sites lus : diff `b6f88c24..d9c0018e` ; `journal_entries.rs:1481-1699` (étapes 1 à 11 d'`update_in_tx`,
  `before_lines` :1629, court-circuit :1636) ; 35-2 AC-5 (:548-555) et Tâches (:766) ; 35-3b AC-2 (:205-235, provenance
  conservée en édition :156-185, :283-284) ; 35-4 et 35-5 (aucune lecture de `rate_source = 'afc'` hors audit).
  Axes exercés : remédiation CR-3 P1 et A17, cohérence inter-fiches de S6, tests des AC modifiés. Non exercés : GET de
  la source journalière (aucune prémisse ici), i18n des quatre locales et manuels (aucune clé neuve ; 35-3b), MCP et
  E2E (non touchés par A17), code au-delà d'`update_in_tx`.
- 2026-11-14 — validation CR-3 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW ; remédiés dans la fiche.
  (1) critère du `PUT` à la garde `laterFiscalYear` : « même appariement que le point 4 » faisait entrer `rate_source`,
  si bien qu'un `PUT` qui ne change que la provenance aurait été refusé alors qu'aucun solde ne change — `rate_source`
  exclu de la garde ; (2) `RATE_UNAVAILABLE` / `FX_RATE_NOT_AFC` : « dont 35-3b tire » les deux actions, qui ne
  vont qu'au second — réécrit ; (3) test (g), date de l'écriture changée : vers un mois sans cours, le refus serait
  `RATE_UNAVAILABLE` et non `FX_RATE_NOT_AFC` — mois à cours présent précisé. Inter-fiche, non appliqué : 35-3b AC-2
  (`RATE_UNAVAILABLE`, « compte, devise, montant, cours ou date ») omet le côté et le passage `manual` → `afc` qui
  font aussi une ligne modifiée (35-3a AC-4, 4). Sites lus : diff `d9c0018e..040185ac` ;
  `journal_entries.rs:1300-1320` (`is_no_op_change`, positionnel, sans `foreign`), `:1481-1500` (`old_date` déjà lu à
  l'étape 1), `:1620-1636` ; 35-2 AC-5 (:552-561), 35-3b (:72-80, :205-236), fichier d'epic A17. Axes exercés : vérité
  au code des phrases ajoutées, cohérence AC-4/AC-7/Tâches, points de contact 35-2/35-3b/35-4/35-5. Non exercés :
  reste de la fiche, i18n, manuels, MCP, E2E.
- 2026-11-14 — révision CR-4 (revue 2 de Beat T1–T6 ; Opus) : **T3** — la colonne `journal_entry_lines.currency` est
  **retirée** : migration à trois colonnes (`amount_currency`, `exchange_rate`, `rate_source`), `chk_jel_fx_currency`
  supprimée, `chk_jel_fx_all_or_none` sur trois colonnes, `chk_jel_fx_chf` gardée par `amount_currency IS NULL`, liste
  close R1 à trois colonnes (AC-1) ; la devise d'une ligne se lit sur le compte : `LINE_COLUMNS` qualifiée et jointure
  unique `LINE_FROM` pour les sept lectures, `account_currency` sur `JournalEntryLine`, `list_all_lines_by_company`
  joint `accounts`, `OriginLine` sans devise (AC-2) ; `ForeignAmount.declared_currency` (devise annoncée, jamais
  stockée) ; `is_no_op_change`, critère `laterFiscalYear` et appariement A17 sur les trois champs stockés (AC-2,
  AC-4) ; rang 2 bis jugé sur les devises **des comptes** lues au rang 1 — cas admis CHF–CHF, devise–CHF,
  devise–même devise, refus EUR–USD (`FX_MULTIPLE_CURRENCIES`) ; rang 3 : `ACCOUNT_REQUIRES_CURRENCY`, puis code neuf
  **`FX_ACCOUNT_IN_CHF`** (ligne `foreign` sur un compte CHF), puis `FX_CURRENCY_MISMATCH` réservé à une devise
  annoncée ≠ devise du compte ; `FX_RATE_NOT_AFC` sur la devise du compte ; bascule (35-1) : toutes les lignes du
  compte (AC-4) ; API : `foreign.currency` facultatif à l'entrée, toujours rendu (lu sur le compte) (AC-6) ;
  contre-passation sur le même compte (AC-7) ; CSV sans colonne de devise, audit avec la devise du compte, MCP par la
  jointure, `.keshbackup` à trois colonnes (AC-8) ; limite écrite : aucun `CHECK` ne lie une ligne en devise à un
  compte en devise (AC-1). Prémisses relues au code : `journal_entries.rs:103` (`LINE_COLUMNS` sans alias), lectures
  `:549`, `:765`, `:975`, `:1630`, `:1708`, `:1975`, `:3081` (aucune `FOR UPDATE`), `:2047-2058`
  (`list_all_lines_by_company`, `query_as::<_, JournalEntryLine>`), `:2896-2925` (`OriginLine`, `FOR UPDATE`),
  `:999-1016` (`entry_snapshot_json`), `:135-180` (`validate_lines_accounts_in_tx`) ; `entities/journal_entry.rs:151-212`
  ; `services/ecritures.rs:201`, `:243` (via `list_by_company_paginated` et `find_by_id`) ;
  `tests/letterings_lexical.rs:85-105` (écritures seulement). Cohérence relue avec 35-3b (extourne, cours d'ouverture),
  35-4 (lecture `a.currency`) et 35-5 (ligne `revaluation`). T1, T2, T4, T5, T6 sans point de contact ici. AC
  touchés : en-tête, AC-1, AC-2, AC-3 (liste des refus par ligne), AC-4, AC-6, AC-7, AC-8, Tâches. Fiche à revalider
  (cycle CR-4).
- 2026-11-14 — validation CR-4 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 5 LOW ; remédiés dans la fiche (aucun
  inter-fiche). (1) 35-3a-CR4-P1-1 : l'exemple du mois de référence (AC-4, 4) citait « le mois précédant la date d'un
  solde d'ouverture, C-35-9 », périmé par T4 (cours d'ouverture = cours de clôture précédent, genre `day` ou `month`) ;
  l'entrée de la révision disait T4 « sans point de contact ici ». (2) -2 : AC-5 annonçait `ACCOUNT_REQUIRES_CURRENCY`
  pour tout chemin automatique visant un compte en devise, alors que, la devise se lisant sur les comptes (T3), une
  pièce visant deux devises étrangères rend `FX_MULTIPLE_CURRENCIES` (précédence du rang 2 bis). (3) -3 : `foreign()`
  non défini pour `account_currency NULL` avec `amount_currency` posé (limite de l'AC-1) ; signature de
  `validate_lines_accounts_in_tx` (`Result<(), _>` → devises des comptes) non écrite. (4) -4 : la relecture après
  `INSERT` (`:549`, `:1708`) est un `SELECT` non verrouillant sous `REPEATABLE READ`, dont l'instantané précède le
  verrou des comptes (`books_locked_through` lu sans verrou, `:316-324`) : devise périmée dans la réponse et l'audit
  « après » en cas de changement de devise concurrent d'un compte sans ligne — les lignes relues prennent la devise de la
  lecture verrouillante du rang 1. (5) -5 : tests manquants (précédence `FX_ACCOUNT_IN_CHF` sur `FX_CURRENCY_MISMATCH`,
  `usd`, lot à deux devises, MCP, `foreign()` incohérent, devise des lignes relues). Sites lus : `journal_entries.rs:103,
  135-180, 316-324, 495-540, 549, 765, 975, 1300-1320, 1630, 1667, 1708, 1975, 2047-2058, 2896-2925, 3081, 999-1016` ;
  `entities/journal_entry.rs:150-212` (`JournalEntryLine` dérive `Serialize` : aucune sérialisation directe hors DTO
  trouvée) ; `csv_tables.rs:356-397` (CSV sans colonne de devise) ; `tests/letterings_lexical.rs:60-110` (la garde ne
  lit que des écritures : `LINE_FROM` et la `LINE_COLUMNS` qualifiée n'y entrent pas) ; `backup.rs` (`is_required`) ;
  grep des `INSERT INTO journal_entry_lines` de production (deux, `:505`, `:1667`) et des `query_as` de
  `JournalEntryLine` (aucun hors `journal_entries.rs`) ; 35-1 AC-4 (verrou de devise), 35-3b, 35-4, 35-5 (aucun reste de
  `journal_entry_lines.currency`, `chk_jel_fx_currency` ni « quatre colonnes » hors historique). Axes exercés :
  résidus de T3 sur les six fiches, prémisses de code des lectures et écritures de lignes, ordre des refus, verrous et
  isolation, CSV/audit/MCP/sauvegarde, tests des AC modifiés. Non exercés : GET de la source journalière (aucune prémisse
  ici), i18n des quatre locales (aucune clé relue au code), manuels et E2E (35-3b), test de concurrence non exécuté.
- 2026-11-14 — révision CR-5 (revue 3 de Beat U1–U6 ; Opus) : **U1** — `FX_RATE_NOT_AFC` (AC-4, 4) compare au cours
  retenu par la société, la ligne `exchange_rates` `source = 'afc'` du mois, lue par `find_company_rate_in_tx` de
  35-2 ; plus aucun `find_rate_in_tx` sur la table système pour ce contrôle (la dernière valeur publiée n'y entre
  jamais) ; mois à ligne `manual` → ligne `afc` refusée, `expectedRate` `null` ; aucune ligne de la société →
  `RATE_UNAVAILABLE` ; la restauration ne vide plus les cours (contrôle opérant dès l'import) ; « adoption » remplacée
  par la reprise par ↑, par société ; tests (a)–(h) récrits, dont deux sociétés à cours différents et la restauration ;
  liste close sous le nom `exchange_rates` (AC-1). **U2** — AC-7 : changer le compte d'une ligne au `PUT` — la devise
  suit le compte (USD → EUR, devise → CHF, CHF → devise), ligne modifiée au sens d'A17, une seule devise par écriture,
  verrou du compte quitté levé s'il n'a plus de ligne ; tests. **U5** — A3 (exemption de l'ouverture) noté approuvé ;
  A17, A20, A21 déjà AC (AC-4, AC-6). **U6** — AC-3 : débit et crédit restent en devise principale. Aucune migration
  ni route neuve. AC touchés : en-tête, AC-1, AC-3, AC-4, AC-7, Tâches. Fiche à revalider (cycle CR-5).
- 2026-11-14 — validation CR-5 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW (plus 1 LOW inter-fiche) ; remédiés dans la
  fiche sauf 35-3a-CR5-P1-IF1 (35-3b). (1) -1 : les Tâches ne listaient aucun test de l'AC-7 neuf (U2) ; (2) -2 : aucun
  test du passage d'un mois en `manual` par `PUT …/rates`, que l'AC-4 dit sans effet sur le `PUT` du libellé ; (3) -3 : la
  source `afc` de la ligne de la société se lit sur `CompanyRate.source` (35-2 AC-1), non écrite à l'AC-4. IF1 : 35-3b
  AC-2 propose « Reprendre le cours attendu » sur tout `FX_RATE_NOT_AFC`, or `expectedRate` vaut `null` pour une ligne de
  la société `manual` : n'offrir alors que « Garder mon cours ». Sites lus : `journal_entries.rs:103, 135, 505, 1243, 1300,
  1667` ; `routes/journal_entries.rs:265` ; `backup.rs:46` ; 35-2 AC-1 (:529), AC-5 (:574-584), AC-7 (:955), Tâches
  (:1018-1032) ; 35-1 AC-4 (`currencyLocked` :326) ; 35-3b AC-2 (:266-300) ; epic A3, A17, A20, A21, A24 ; aucun reste
  des anciens noms (`exchange_rate_overrides`, `_kesh_exchange_rates`, `republished_*`, `find_rate_in_tx`) hors historique du
  Change Log. Axes exercés : résidus U1 sur les six fiches, prémisses de code citées, tests des AC modifiés, cohérence
  35-2/35-3b. Non exercés : GET de la source journalière (aucune prémisse ici), i18n, manuels, MCP, E2E (inchangés par CR-5),
  test de concurrence non exécuté. Arbitrages A23–A29 : sans point de contact ici (A24 : la copie couvre la devise du compte de
  la ligne, 35-1/35-2).
- 2026-11-14 — alignement A30 (orchestrateur, Sonnet) : une `source` de cours inconnue (`RateOrigin::Other(String)`, p. ex. `ecb`, 35-2 AC-1) se traite comme `manual` au contrôle `FX_RATE_NOT_AFC` (AC-4, 4) : ligne de la société `manual` ou de toute source autre que `afc`, ligne d'écriture `afc` refusée, `expectedRate` `null` ; test (b bis) ajouté (ligne `ecb` → refus, `expectedRate` `null`). Noms repris de 35-2 (`RateOrigin`, `find_company_rate_in_tx`).
- 2026-11-14 — validation CR-5 P5 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche. LOW
  35-3a-CR5-P5-1 (AC-4, 4) `expectedRate` « `null` si la ligne est `manual` » laissait de côté une autre source, que
  la phrase précédente traite comme `manual` — « ou d'une autre source, A30 ». Vérifié : test (b bis), noms
  `RateOrigin::Other`, `find_company_rate_in_tx`, `expectedRate` alignés sur 35-2 AC-1/AC-5 et 35-3b AC-2. Non
  exercés : reste de la fiche hors diff, compilation.
- 2026-11-14 — validation CR-5 P6 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW ; aucune modification. Phrases A30 du diff P5 vérifiées contre 35-2 (AC-1, AC-5, AC-6) ; aucun site restant. Non exercés : reste de la fiche hors diff, compilation, rendu.
