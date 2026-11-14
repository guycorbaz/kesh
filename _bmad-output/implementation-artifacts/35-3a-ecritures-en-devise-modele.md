# Story 35-3a : écritures en devise — modèle, validation, API et garde des chemins automatiques

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, C-35-3, C-35-4, C-35-5. Suit 35-1
et 35-2. **Migration non breaking** (colonnes `NULL` ajoutées). **Découpée** à la rédaction (règle de splitting
préventif : plus de cinq modules) : 35-3a = le serveur (`kesh-core`, `kesh-db`, `kesh-api`, `kesh-reconciliation`,
`kesh-seed`) et les sélecteurs serveur ; 35-3b = soldes d'ouverture, formulaire et écrans. Révisée le 2026-11-14 pour
la revue de Beat (CR-2 : R1, R3, R7), qui prime sur le registre antérieur ; puis pour sa revue CR-3 (S2 : comptes de
différence propres à un compte en devise, résolus par `fx_accounts_for` de 35-1 ; S6 : `FX_RATE_NOT_AFC` compare au
cours AFC **en vigueur**), qui prime sur CR-2.*

## Story

En tant que comptable en multi-devises, je veux saisir une écriture dont certaines lignes sont en devise — montant en
USD, cours du mois proposé (le cours mensuel AFC, ou le cours que la société a saisi pour ce mois, D9), montant en
CHF calculé — et, pour une conversion, passer par le compte de conversion en CHF deux écritures d'une seule devise
chacune (R7), l'écart de change dans la seconde, afin que mes comptes en devise portent leur solde en devise et en CHF.

## Critères d'acceptation

1. **AC-1 — Migration** `20261114000003_journal_lines_foreign_currency.sql` : sur `journal_entry_lines`,
   `currency CHAR(3) CHARACTER SET ascii COLLATE ascii_bin NULL` (**même définition** que `accounts.currency`, 35-1
   AC-1 : la table est en `utf8mb4_unicode_ci`, sans collation binaire `REGEXP` ignorerait la casse),
   `amount_currency DECIMAL(19,4) NULL`, `exchange_rate DECIMAL(20,10) NULL`,
   `rate_source VARCHAR(12) NULL` ; contraintes **nommées** (C-35-3), chacune tolérante à la ligne CHF (quatre `NULL`) :
   - `chk_jel_fx_all_or_none` : les quatre `NULL`, ou les quatre non `NULL` ;
   - `chk_jel_fx_currency` : `currency IS NULL OR (BINARY currency REGEXP '^[A-Z]{3}$' AND BINARY currency <>
     BINARY 'CHF')` (écriture de 35-1, à recopier ; symétrique de 35-1 : la devise principale s'écrit `NULL`) ;
   - `chk_jel_fx_rate_source` : `rate_source IN ('afc','manual','revaluation')` (en `BINARY`, sur le modèle de
     `20261110000001`) ;
   - `chk_jel_fx_amount` : `rate_source = 'revaluation'` ⇒ `amount_currency = 0` ; sinon `amount_currency > 0` ;
   - `chk_jel_fx_rate` : `exchange_rate > 0` ;
   - `chk_jel_fx_chf` (filet structurel, C-35-4) : `currency IS NULL OR rate_source = 'revaluation' OR debit + credit =
     ROUND(amount_currency * exchange_rate, 2)` — le `ROUND` de MariaDB sur `DECIMAL` arrondit l'équidistant loin de
     zéro, comme `MidpointAwayFromZero` ; il couvre les chemins qui n'appellent pas `validate()` (contre-passation,
     restauration `.keshbackup`, 35-5).
   **Filet, pas refus métier** : il n'existe **aucune correspondance par nom de contrainte** (une violation `4025`/`3819`
   devient `DbError::CheckConstraintViolation`, `kesh-db/src/errors.rs:1726`, puis un **400** générique
   `CHECK_CONSTRAINT_VIOLATION`, `kesh-api/src/errors.rs:3801` — jamais une 500). Les `chk_jel_fx_*` s'y appuient sans
   bras dédié ; les refus précis sont ceux de `kesh-core` (AC-3) et du dépôt (AC-4), qui les précèdent. Test, **par
   contrainte** et en SQL brut : quatre colonnes partielles, `usd`, `CHF`, source inconnue, montant 0 hors
   `revaluation`, montant < 0, ligne `revaluation` au montant ≠ 0 (seule `chk_jel_fx_amount` violée), cours 0,
   CHF incohérent → chacune refusée par **sa** contrainte (nom lu dans le message) ;
   la ligne CHF (quatre `NULL`) et une ligne `revaluation` valide passent. ⚠️ Hors `revaluation`, un montant ≤ 0 ou un
   cours 0 viole **aussi** `chk_jel_fx_chf` (`debit + credit > 0` par `chk_jel_debit_credit_exclusive`, squash :863) :
   `chk_jel_fx_chf` est déclarée **en dernier** dans la migration, le cas « cours 0 » se construit sur une ligne
   `revaluation` (montant 0, CHF > 0 : seule `chk_jel_fx_rate` est violée), et le nom rapporté pour les deux cas qui violent aussi
   `chk_jel_fx_chf` (« montant 0 hors `revaluation` » et « montant < 0 ») est constaté au premier essai et écrit au Change Log (ordre d'évaluation des `CHECK` de MariaDB).
   Inscriptions habituelles (sha384, audit, squash, P6 inspecté, P7 sans objet : DDL seul). Colonnes `NULL` : cette
   migration ne referme pas la fenêtre d'import, et 35-2 non plus depuis CR-2 (R1 : la liste close des absences
   admises de 35-1 AC-6 couvre `exchange_rate_overrides`). Les quatre colonnes **s'inscrivent à cette liste** (déjà
   tolérées par la règle générique, la liste les garde non `is_required()`). Ce qui se teste ici : le test d'import
   d'une sauvegarde 0.17.6 de 35-1 AC-6, étendu aux quatre colonnes retirées des NDJSON de `journal_entry_lines` →
   import 200, lignes aux quatre colonnes `NULL` (Tâches).
2. **AC-2 — Modèle Rust.** `ForeignAmount { currency, amount, rate, source }` (`kesh-core` ; `source` est un enum
   nommé `RateProvenance { Afc, Manual, Revaluation }` — **pas** `RateSource`, déjà le trait de 35-2 dans `kesh-api` ;
   montant et cours en `Decimal`, non en `Money` ; le type dérive `Debug, Clone, PartialEq, Eq, Serialize,
   Deserialize` comme `JournalEntryLineDraft` qui le porte, `RateProvenance` de même) ;
   `NewJournalEntryLine.foreign: Option<ForeignAmount>` **et** `JournalEntryLineDraft.foreign: Option<ForeignAmount>`
   (`balance.rs:72`, le type que lit `validate()`), **sans** `Default` : chaque site de construction est revu
   (inventaire de l'échec de compilation, écrit au Change Log, site par site : `None` motivé ou extension) ;
   `JournalEntryLine` relu avec les quatre colonnes. Sites SQL de `journal_entries.rs`, liste close (production ; `mod
   tests` commence à `:3177`) :
   - `INSERT` : `:505` (`create_in_tx_inner`) et `:1667` (`update_in_tx`) — les deux seuls `INSERT INTO
     journal_entry_lines` de production de tout `crates/` ;
   - lectures : `LINE_COLUMNS` (`:103`) ; colonnes en dur de `list_all_lines_by_company` (`:2047`, source de l'export
     CSV) ; lecture `OriginLine` de la contre-passation (`:2896`), puis recopie `:2917` ;
   - re-sommes d'équilibre (CHF, inchangées) : `:520-537` et celle d'`update_in_tx`.
   - **`is_no_op_change`** (`:1300`) : compare aujourd'hui compte, débit, crédit et projet **seulement** ; sans les
     quatre champs en devise, un `PUT` qui ne change que le cours, la source ou la devise d'une ligne (CHF identique)
     serait jugé **sans effet** — ni écriture, ni `version`, ni audit. Il compare `foreign` (les quatre champs,
     montants comparés en valeur, non en graphie) ; test : un `PUT` qui ne change que `rateSource` écrit et audite.
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
     (`ACCOUNT_REQUIRES_CURRENCY`, `FX_CURRENCY_MISMATCH`, `FX_RATE_NOT_AFC`, AC-4) : 35-3b affiche le message sur la
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
   - l'équilibre reste Σ débit CHF = Σ crédit CHF, **strict** ; déséquilibre → `EntryUnbalanced` avec l'écart.
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
   1. `InactiveOrInvalidAccounts` → `AccountsNotPostable` (existants, `validate_lines_accounts_in_tx` :135) ;
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
      garde des comptes). **Critère au `PUT`** : la garde ne refuse que si les lignes en devise **changent** — la suite
      des lignes d'avant qui portent une devise diffère de celle du corps (dans l'ordre, compte, débit, crédit et les
      quatre champs de `foreign` comparés comme `is_no_op_change`, montants en valeur). Un `PUT` qui ne touche que le
      libellé, la date dans l'exercice ou des lignes CHF ne change pas le solde d'ouverture de l'exercice mono-devise
      et passe ; un `PUT` sans effet passe donc aussi, sans exception à KF-004 (toutes les gardes **précèdent** le
      court-circuit de l'étape 8, `:1636` ; celle-ci ne refuse simplement pas ce qui ne change rien). Le message dit
      quoi faire (passer l'exercice postérieur en multi-devises) ;
   2 bis. **une seule devise étrangère par écriture (R7)** : des lignes en devise de deux devises différentes →
      400 **`FX_MULTIPLE_CURRENCIES`** (code neuf, `details: { currencies: ["EUR", "USD"] }`, triées ; variante
      `DbError::FxMultipleCurrencies`, bras dans `errors.rs` et dans les correspondances `FailedProposal` de l'AC-5),
      calculé sur le corps seul à la création (aucune lecture), rendu à ce rang de l'ordre des refus ; les lignes CHF
      restent admises à côté. Une conversion entre
      deux devises passe par le compte de conversion de 35-1 en deux écritures (35-3b, assistant *Conversion*).
      Au **`PUT`** (`update_in_tx`), la règle ne refuse que si le corps porte plus d'une devise **et** une
      devise que l'écriture stockée ne portait pas (l'ensemble des devises ne s'élargit jamais ; alignement de
      l'orchestrateur sur 35-3b-CR2-P2-9) — jugé sur les lignes d'avant lues à l'étape 6 (rang 2 ci-dessus),
      seule lecture de plus : une écriture d'ouverture ou de complément déjà à plusieurs devises
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
      ou une ouverture. Exemption de l'ouverture à confirmer par Beat (Change Log). ⚠️ **La contre-passation n'est exemptée ni de
      `MULTI_CURRENCY_DISABLED` ni de `laterFiscalYear`** : elle est datée du jour ou du premier jour de l'exercice
      suivant (`ReversalDating`, `reverse_in_tx_inner`), exercice qui peut être mono-devise — contre-passer une
      réévaluation au 1er janvier d'un exercice mono-devise est refusé en 409, le message disant de le passer en
      multi-devises ; comportement assumé, testé (origine en devise, exercice du jour mono-devise ; origine en
      devise, exercice suivant mono-devise) ;
   3. ligne **sans** devise sur un compte en devise → `ACCOUNT_REQUIRES_CURRENCY` ; ligne en devise sur un compte d'une
      autre devise ou CHF → `FX_CURRENCY_MISMATCH` (pour **l'un et l'autre** : première ligne fautive, son rang en
      `details.line`, AC-3). Quand l'exercice de l'écriture est **mono-devise**, `ACCOUNT_REQUIRES_CURRENCY` porte en
      plus `details.reason: "monoCurrencyFiscalYear"` et `fiscalYearId` : l'invite « saisissez le montant en devise »
      y serait fausse (la ligne en devise est refusée au rang 2) — le message dit de passer l'exercice en
      multi-devises (clé i18n propre, quatre locales ; affichage 35-3b, inter-fiche) ;
   4. ligne `rate_source = 'afc'` : le cours doit égaler `rate_per_unit` du cours AFC **mensuel brut en vigueur** de
      `_kesh_exchange_rates` (35-2) pour (devise, mois de `entry_date`) — **jamais** une valeur republiée non adoptée
      (`republished_rate`, S6, CR-3 : une correction de l'AFC reste à part jusqu'à son adoption à l'écran *Cours de
      change*, 35-2 AC-5 ; `find_rate_in_tx` rend le cours en vigueur, la lecture ne change pas) —, lu dans la transaction par la fonction de
      lecture du dépôt `kesh_db::repositories::exchange_rates::find_rate_in_tx(conn, currency, RateKind::Month, 1er du
      mois)` de 35-2 (pas de second accès). ⛔ **Jamais le cours effectif** : un cours saisi par la société (D9,
      `exchange_rate_overrides`) n'entre pas dans ce contrôle — une ligne à ce cours, s'il diffère du cours AFC, est
      `manual` (le client la qualifie ainsi, 35-3b) ; envoyée `afc`, elle est refusée. Ni le cours journalier (D10).
      Aucune nouvelle valeur de `rate_source`. Sinon 400
      `FX_RATE_NOT_AFC`, `details: { line, expectedRate }` (`rate_per_unit` de la table, chaîne décimale sans zéros
      terminaux ; clé lue par 35-3b pour proposer le cours) ; cours absent → `DbError::RateUnavailable {currency, month}`,
      rendu sous le **même code et le même statut 404** que le `RATE_UNAVAILABLE` de 35-2, avec la même forme de
     `details` : `{ currency: "USD", month: "AAAA-MM-01" }` (35-2 AC-5). Contrôle à la création
      **et** au `PUT` (date changée comprise). **Le mois de référence est toujours celui de `entry_date`, sans
      exception** : un cours d'un autre mois (p. ex. celui du mois précédant la date d'un solde d'ouverture, C-35-9)
      se saisit `manual`, jamais `afc` (35-3b y est alignée). Le contrôle porte sur **toutes** les lignes `afc` du
      corps, inchangées comprises : après une restauration (tables des cours vides, 35-2 AC-7), le `PUT` d'une
      écriture `afc` — même pour son seul libellé — rend `404 RATE_UNAVAILABLE` jusqu'à la récupération du cours, et,
      après l'**adoption** d'un cours AFC republié (35-2 AC-5, S6), le `PUT` d'une écriture `afc` à l'ancien cours rend
      `FX_RATE_NOT_AFC` (une republication **non adoptée** ne change rien : le cours en vigueur reste celui de la
      ligne) ; comportement assumé, testé (republication en attente → `PUT` passe ; après adoption → refus), écrit au
      Change Log. **Exemptée** : la contre-passation, qui recopie le cours
      d'origine à une autre date — mécanisme : `create_in_tx_inner` la reconnaît à son paramètre existant
      `reverses_entry_id.is_some()` (`:281`), aucun nouvel indicateur dans `NewJournalEntry` (un indicateur public
      permettrait à un appelant de se dispenser du contrôle).
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
     même requête** au solde cumulé **en devise** (Σ `amount_currency` au débit − Σ au crédit, lignes de la devise du
     compte ; une ligne `revaluation` compte 0) : un compte à CHF nul mais devise non nulle refuse aussi la bascule
     (`FISCAL_YEAR_HAS_FOREIGN_BALANCES`, `reason: "balance"`). Test au dépôt.
5. **AC-5 — Chemins automatiques.** Factures, avoirs, règlements, factures fournisseur, lots de paiement,
   réconciliation (manuelle, éclatement, règles, propositions), — les soldes d'ouverture et leur complément exceptés
   (étendus par 35-3b ; d'ici là, ils refusent un compte en devise comme les autres) :
   - aucun ne construit de ligne en devise ; s'ils visent un compte en devise, le refus central de l'AC-4 s'applique
     et remonte en erreur claire (`ACCOUNT_REQUIRES_CURRENCY`, traduite) ;
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
     classé : résolution (`fx_accounts_for` seule), garde (35-1), sérialisation (DTO, CSV, audit).
     Le **contrôle central de l'AC-4 reste le filet** : un état hérité ou une colonne oubliée ne produit jamais une
     ligne CHF sur un compte en devise, seulement un refus. Les sélecteurs du frontend masquent ces comptes (35-3b) ;
   - inventaire **clos** des sites, écrit au Change Log (méthode « inventorier les sites non résolus », `CLAUDE.md`).
6. **AC-6 — API des écritures.** `CreateJournalEntryLineRequest` prend `foreign?: { currency, amount, rate,
   rateSource: "afc"|"manual" }` (`revaluation` refusé ici : réservé à 35-5) ; `debit`/`credit` restent les montants
   CHF ; **typage du DTO** : les quatre champs de `foreign` sont des `String` (comme `debit`/`credit`), analysés dans
   le handler — la route reçoit le corps par le `Json` d'axum (`create_journal_entry` `:600`), dont l'échec de
   désérialisation (énumération inconnue, nombre JSON pour un décimal) rend **422**, non 400 ; un `rateSource` typé en
   énumération rendrait donc 422 et non le 400 exigé ci-dessous (test HTTP : `rateSource: "revaluation"` → 400) ;
   réponses (`JournalEntryLineResponse`, détail, liste) exposent `foreign: { currency, amount, rate, rateSource }`
   (décimaux en chaînes, comme `debit`/`credit` ; `rateSource` parmi `afc|manual|revaluation`), **omis** pour une ligne
   CHF par `#[serde(skip_serializing_if = "Option::is_none")]` (registre C-35-9 : le JSON HTTP reste identique en
   mono-devise ; ⚠️ contraire à la convention locale du DTO, dont les champs de lettrage sont présents à `null`,
   test `routes/journal_entries.rs:876-881`) — test : le JSON d'une ligne CHF n'a pas de clé `foreign` ; `MAX_LINES_PER_ENTRY` inchangé ;
   erreurs traduites (quatre locales). `MULTI_CURRENCY_DISABLED` est un **409** partout. **Statuts** (une table, un
   test HTTP par code) : **400** `FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO`, `FX_AMOUNT_MISMATCH`,
   `FX_RATE_NOT_AFC`, `FX_CURRENCY_MISMATCH`, `FX_MULTIPLE_CURRENCIES` (R7), `ACCOUNT_REQUIRES_CURRENCY`,
   `ACCOUNT_IN_FOREIGN_CURRENCY` ; **404**
   `RATE_UNAVAILABLE` (comme 35-2 et 35-5) ; **409** `MULTI_CURRENCY_DISABLED`, `ENTRY_IS_FX_REVALUATION`,
   `ACCOUNT_CURRENCY_IN_USE` (35-1). `foreign.currency` se compare **tel quel** à la devise du compte (aucune
   normalisation : `usd` → `FX_CURRENCY_MISMATCH`) ; `rateSource` hors `afc|manual` (dont `revaluation`) → 400
   `Validation` sans code dédié ; message en français et clé i18n d'erreur par code dans les quatre locales
   (`frontend/src/lib/` : codes d'erreur, comme 35-2).
7. **AC-7 — Modification et contre-passation.**
   - le `PUT` (remplacement complet des lignes : `DELETE` `:1646`, `INSERT` `:1667`) prend `foreign` comme le `POST`,
     avec les mêmes contrôles (AC-3, AC-4, dont `FX_RATE_NOT_AFC`) ; l'audit avant/après (`entry_snapshot_json`
     `:999`) porte les quatre champs ;
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
     `routes/journal_entries.rs` `:448` et `:479`), y compris d'une réévaluation, recopie `foreign` **et**
     `rate_source` en inversant les côtés ; elle est exemptée du contrôle `FX_RATE_NOT_AFC`.
8. **AC-8 — Sauvegarde, exports, MCP.**
   - CSV des lignes (`csv_tables.rs` :356) avec les quatre colonnes ; `amount_currency` et `exchange_rate` s'exportent
     à leur échelle de colonne (4 et 10), **jamais** par `fmt_decimal` / `fmt_opt_decimal` (`csv_tables.rs:108`,
     `format!("{d:.2}")`, qui les ramènerait à deux décimales) ; test de sérialisation à 10 décimales ;
   - `entry_snapshot_json` (audit) les inclut **seulement pour une ligne en devise** (clé `foreign` absente sinon) :
     l'audit d'une écriture CHF reste identique octet pour octet (test) ;
   - MCP : `kesh_ecritures` / `kesh_ecriture` les donnent — `crates/kesh-api/src/services/ecritures.rs`
     (`list_by_company_paginated` `:201`, `find_by_id` `:243`) et `crates/kesh-api/src/mcp/outils/ecritures.rs`
     (struct `:126`, construction `:187`) ; champs **présents à `null`** sur une ligne CHF (la macro `sortie!`,
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
     NDJSON, `backup.rs:215-262` — les quatre colonnes sortent seules, décimaux par `rust_decimal`) et la restauration
     passe par l'`INSERT` générique (`backup.rs:833`, sous `chk_jel_fx_*`) ; test : cours à 10 décimales et montant à
     4 identiques après l'aller-retour. Le CSV de `csv_tables.rs` ci-dessus est l'**export utilisateur**, pas la
     sauvegarde.
9. **AC-9 — Limite écrite : le lettrage.** Le lettrage manuel sur un compte en devise ne raisonne qu'en CHF : un
   débiteur USD soldé en USD avec un écart de change ne se lettre pas. Limite assumée, écrite au CHANGELOG, renvoyée au
   registre (C-35-8) pour une issue à l'ouverture de la PR.
## Tâches

- [ ] Migration + inscriptions, contraintes nommées (AC-1 ; **aucune** correspondance par nom dans `errors.rs` : le
      filet rend le 400 générique `CHECK_CONSTRAINT_VIOLATION`) ; colonnes inscrites à la liste close des absences
      admises de 35-1 (R1) ; test : la sauvegarde 0.17.6 simulée de 35-1 AC-6, sans les quatre colonnes, se réimporte,
      colonnes `NULL` ; test : `chk_jel_fx_chf` refuse une ligne incohérente insérée en SQL brut. **Recette de
      montée** (R1, release) : l'empreinte lit `SELECT * FROM journal_entry_lines` (`scripts/recette-montee.sh:414`,
      `TABLES`) ; les quatre colonnes neuves y sont un écart attendu — `journal_entry_lines` s'ajoute au
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
- [ ] Tests de `FX_RATE_NOT_AFC` (D9, AC-4 4), au niveau du dépôt **et** de l'API, avec un cours AFC mensuel posé et un
      cours saisi (`exchange_rate_overrides`) **différent** pour le même mois : (a) ligne `afc` au cours saisi →
      `FX_RATE_NOT_AFC` (`expectedRate` = cours AFC) ; (b) ligne `afc` au cours AFC → **passe** (le cours saisi
      n'entre pas dans le contrôle, dans les deux sens) ; (c) la même ligne en `manual` au cours saisi → passe ;
      (d) mois sans cours AFC mais avec cours saisi : ligne `afc` → `404 RATE_UNAVAILABLE` (le cours saisi ne tient pas
      lieu de cours AFC), ligne `manual` → passe ; (e) un cours journalier AFC ou saisi pour la date de l'écriture ne
      change pas le résultat de (a) à (d).
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
