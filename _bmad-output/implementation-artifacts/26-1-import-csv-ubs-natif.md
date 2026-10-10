# Story 26.1 : Importer le CSV d'UBS tel quel — sens des montants et lignes à ignorer

## Status

**in-progress** — fiche créée le 2026-10-10 en autonomie (skill `bmad-create-story`) ; validation **close** le
2026-10-11 après quatre passes (P1 Sonnet, P2 Opus, P3 Sonnet, P4 ciblée Haiku — bilan au Change Log) ; développement
(`bmad-dev-story`) en cours. Choix tranchés : **C-26-1-1 à C-26-1-27** (et suivants), registre
`_bmad-output/implementation-artifacts/epic-26-choix-autonomes.md`.

**Issues** : [#620](https://github.com/guycorbaz/kesh/issues/620) (P1, erreur comptable) et
[#621](https://github.com/guycorbaz/kesh/issues/621) (CR). **PR : `closes #620`, `refs #621`** — #621 reste ouverte pour
la lecture de l'IBAN et des soldes du préambule (décision 3 du Project Lead).

**Première story de l'Epic 26** (plan : `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md` § 26-1). Guy l'attend
pour poursuivre sa recette de la v0.13.0 : **il n'importera pas de fichier retouché**.

## Story

**As a** particulier ou indépendant client d'UBS,
**I want** importer dans Kesh le fichier CSV que me donne l'e-banking d'UBS, sans le retoucher,
**so that** mes dépenses entrent comme des sorties et mes encaissements comme des entrées, au centime, et que je
puisse tenir ma comptabilité personnelle avec le seul format qu'UBS me propose (hors MT940, #618).

## Le défaut, en une phrase

En mode « Colonnes débit/crédit séparées », `parse_row` (`crates/kesh-import/src/csv/parser.rs`, branche
`(false, true)`) rend **`-parsed`** : UBS écrivant ses débits **déjà négatifs**, `-(-500.00) = +500.00` — **chaque
dépense devient un encaissement**, sans erreur ni avertissement, et le CSV ne portant pas de solde de clôture,
aucun contrôle ne le révèle. Et le profil nécessaire n'est même pas enregistrable : le préambule UBS demande 9 lignes
ignorées, le plafond est 5 (`profile.rs`, `CHECK (header_row_count <= 5)`, `max="5"` du formulaire).

## Décisions imposées par le Project Lead (2026-10-10) — non négociables

1. **Le sens des montants est un réglage explicite du profil**, jamais une déduction de Kesh.
2. **Le nombre de lignes à ignorer reste un réglage**, plafond relevé ; **pas** de détection de la ligne de titres.
3. **La lecture de l'IBAN et des soldes du préambule est HORS périmètre** (#621 reste ouverte pour elle).

## Le fichier d'UBS — ce qu'on en sait, en formes seulement

⛔ **DONNÉES PERSONNELLES, DÉPÔT PUBLIC.** Le fichier réel de Guy (`/home/gcorbaz/kesh-test/transactions.csv`) ne se
copie **jamais** dans le dépôt, ni en entier, ni en extrait ; aucune ligne de transaction, aucun nom, aucune adresse,
aucun IBAN, aucun solde ne s'écrit dans un fichier versionné, un rapport de passe, un message de commit ou une PR.
⛔ **Pas même d'agrégat** (somme, soldes, nombre de mouvements, de sorties ou d'entrées : arbitrage (0) de
l'orchestrateur, validation P2) : au Dev Agent Record, le contrôle du vrai fichier s'écrit « conforme » ou « non
conforme », rien d'autre.

Structure relevée le 2026-10-10 sur le fichier réel (formes seulement, sans valeurs ni comptes) :

| Élément | Fait |
|---|---|
| Encodage | UTF-8 **avec BOM** (`EF BB BF`), fins de ligne **LF**, **pas de LF après la dernière ligne** |
| Lignes 1-8 | préambule clé/valeur, 3 champs chacune (`Clé:;valeur;`) : `Numéro de compte:`, `IBAN:`, `Du:`, `Au:`, `Solde initial:`, `Solde final:`, `Évaluation en:`, `Nombre de transactions dans cette période:` |
| Ligne 9 | **vide** (aucun caractère) |
| Ligne 10 | titres : `Date de transaction;Heure de transaction;Date de comptabilisation;Date de valeur;Devise;Débit;Crédit;Sous-montant;Solde;No de transaction;Description1;Description2;Description3;Notes de bas de page;` (14 titres + `;` final = **15 champs**) |
| Lignes 11 à la fin | les transactions (autant que la clé `Nombre de transactions…` en annonce), toutes à 15 champs, **dates décroissantes** (la plus récente en tête) |
| Dates | colonnes 0, 2, 3 au format `AAAA-MM-JJ` ; colonne 1 (heure) **toujours vide** |
| Débit (col. 5) | **toutes négatives**, sans séparateur de milliers, point décimal |
| Crédit (col. 6) | toutes positives |
| Débit **et** crédit | jamais les deux sur une ligne, jamais aucun des deux |
| Sous-montant (col. 7), Notes (col. 13) | toujours vides |
| Descriptions (col. 10-12) | souvent **entre guillemets et contenant des `;`** ; col. 11 parfois vide |
| Contrôle attendu | somme des montants signés = solde final − solde initial **du préambule** ; nombre = nombre annoncé par le préambule. ⛔ Aucun chiffre réel (somme, soldes, nombre de mouvements, de sorties ou d'entrées) ne s'écrit dans le dépôt : le test AC17 les lit dans le fichier et ne rend que « conforme / non conforme ». |

## Critères d'acceptation

### Le réglage du sens (#620)

**AC1 — `DebitSign` dans le profil (`kesh-import`).** `crates/kesh-import/src/csv/profile.rs` gagne
`pub enum DebitSign { Positive, Negative }` (`serde(rename_all = "lowercase")`, `Default = Positive`, `as_str()` →
`"positive"` / `"negative"`, `FromStr` / `TryFrom<&str>` refusant toute autre chaîne) et `CsvProfile` le champ
`pub debit_sign: DebitSign`, documentés (`///`) : *« Comment la banque écrit les montants de la colonne Débit, en mode
débit/crédit séparés : `Positive` — sans signe, Kesh les rend négatifs (comportement d'avant la 0.13.1) ; `Negative` —
déjà précédés d'un signe moins (UBS), Kesh les garde. »* (C-26-1-2). Réexporté par `csv/mod.rs` et `lib.rs` comme
`ColumnMapping` / `CsvProfile`. **`ColumnMapping` n'est pas touché** (C-26-1-3) : ni champ, ni retrait de
`deny_unknown_fields`.

`CsvProfile::validate` refuse `debit_sign = Negative` quand le mapping est en **colonne unique** (`amount` présent) :
`ProfileMisconfigured("debit_sign = negative ne s'applique qu'en mode débit/crédit séparés")`. Le doc-module de
`profile.rs` (liste « Validation ») gagne cette règle.

**AC2 — Le parseur applique le réglage, sans rien déduire.** Dans `parse_row`, branche `debit_credit_split` :

| Réglage | Colonne Débit `d` | Colonne Crédit `c` |
|---|---|---|
| `Positive` (défaut) | montant = `-d` (inchangé) | montant = `c` (inchangé) |
| `Negative` | montant = `d` (gardé tel quel) | montant = `c` (inchangé) |

Le mode colonne unique (`amount`) est **inchangé** : signe préservé. Les branches `(true, true)` (`EMPTY_MANDATORY_FIELD`)
et `(false, false)` (`AMBIGUOUS_DEBIT_CREDIT`) sont inchangées. Le doc-comment de `parse_row` et celui du module
(pipeline) décrivent le réglage.

**AC3 — Un montant de signe contraire à sa colonne refuse l'import entier (C-26-1-5, C-26-1-15).** Nouveau variant
`CsvLineErrorCode::AmountSignMismatch` (sérialisé `AMOUNT_SIGN_MISMATCH` — cf. AC3-bis, `i18n_key()` →
`bank-import-csv-errors-amount-sign-mismatch`), constaté en mode séparé, `value = Some(<cellule brute>)` :

- colonne Débit, réglage `Positive`, valeur **strictement négative** ;
- colonne Débit, réglage `Negative`, valeur **strictement positive** ;
- colonne Crédit, valeur **strictement négative**, quel que soit le réglage.

Zéro (`0.00`, `-0.00`, `0,00`) est accepté partout (le test `explicit_zero_debit_or_credit_accepted` reste vert, et un
cas `-0.00` sous `Negative` s'y ajoute ; comparer `< Decimal::ZERO` / `> Decimal::ZERO`, jamais le bit de signe).

⛔ **Bloquante et non confirmable** (arbitrage de l'orchestrateur sur F4, C-26-1-15) : si **une seule** ligne porte ce
code, l'import **entier** est refusé — la case « importer les lignes valides quand même » (`confirmPartialImport`) **ne
s'applique pas** à cette erreur. Motif : sous le mauvais réglage, les seuls montants « valides » sont ceux de l'autre
colonne ; les importer seuls reproduirait #620 (des dépenses absentes, un solde faux, en silence).

⛔ **Bloquante aussi pour une extourne isolée** (F2-5, arbitrage de l'orchestrateur, C-26-1-23) : un crédit négatif
(encaissement contre-passé) ou, sous `Negative`, un débit positif (paiement remboursé) a pourtant un sens non ambigu.
Il bloque quand même le relevé entier — sécurité d'abord. **Conséquence assumée et écrite** (fiche, manuel,
CHANGELOG) : un relevé qui porte une seule telle ligne **ne s'importe pas tel quel** dans cette version. Le relevé réel
de Guy n'en contient aucune au 2026-10-10 (structure relevée, sans chiffre). Si le cas se présente, une issue suivra.

- `kesh-import` — **compteur et vecteur propres au signe** (R2-1 = F2-1, arbitrage (1)) : `parse_csv_collect` tient,
  **indépendamment** du vecteur général plafonné et de son compteur, `sign_total: usize` (toutes les erreurs de signe
  du fichier) et `sign_errors: Vec<CsvLineError>` plafonné à `MAX_CSV_LINE_ERRORS`, alimentés à chaque erreur de
  signe **avant** et **quel que soit** l'état du plafond général. La décision porte sur **`sign_total > 0`**, jamais
  sur le vecteur général écrêté (qui peut avoir été rempli par 100 erreurs d'une autre nature avant la première de
  signe). Si `sign_total > 0`, `parse_csv_collect` rend
  `ParseCsvOutcome::HardFailure(CsvError::AmountSignMismatch { errors: sign_errors, total_errors: sign_total,
  truncated: sign_total > MAX_CSV_LINE_ERRORS, whole_debit_column })` — nouvelle variante documentée : les autres
  erreurs de ligne du même fichier ne sont pas rapportées (régler le profil d'abord) ; `whole_debit_column` —
  **définition fermée** (R3-2 = F3-2, arbitrage (2) de la P3, C-26-1-25) : parmi les lignes dont le montant de la colonne
  **Débit a été lu** (date lisible, une seule des deux cellules remplie, la cellule Débit, montant lisible), soit `d` le
  nombre de débits **non nuls** et `e` le nombre de ceux qui sont **en défaut de signe** ; `whole_debit_column ⇔ d ≥ 2
  et e = d`. Les lignes en échec **avant** le montant (date illisible, ligne trop courte, `AMBIGUOUS_DEBIT_CREDIT`,
  `EMPTY_MANDATORY_FIELD`) et un débit **illisible** (`INVALID_AMOUNT`) sont **hors** du dénominateur comme du
  numérateur ; une erreur de signe de la colonne **Crédit** compte dans `sign_total`, jamais dans `d` ni `e`. Un
  fichier à **un seul** débit en défaut (`d = 1`) reçoit le message « lignes listées », **jamais** le conseil de changer
  le réglage (F3-4 : le suivre importerait un remboursement comme une dépense, en silence).
  **D'où sort l'information** : `parse_row` change de type de retour (fonction privée) pour
  `RowParse { result: Result<ImportedTransaction, CsvLineError>, debit: DebitObservation }`, avec l'énumération interne
  `DebitObservation { NotRead, Zero, Consistent, SignMismatch }` (`NotRead` : colonne unique, ligne de crédit, échec
  avant le montant, débit illisible) — **rien** n'est ajouté à `CsvLineError`, type public sérialisé dans
  `details.lines`. `parse_csv_collect` compte `d` (`Consistent` + `SignMismatch`) et `e` (`SignMismatch`). Le test
  existant qui appelle `parse_row` directement, `parses_corrupt_profile_with_both_amount_and_split_prefers_split`
  (`parser.rs` l. ≈ 1064-1067), suit : `parse_row(…).result`. `parse_csv` (wrapper) rend la même `Err`. La décision
  vit dans `kesh-import`, pas dans la route : tout appelant en hérite.
- **Préséances** (R2-9, F2-11, R3-10) : (0) les erreurs d'avant la boucle des lignes l'emportent toutes, dans
  l'ordre du code : profil invalide (`validate`), encodage non supporté, `EncodingMismatch` (traité en (1)), en-têtes
  manquants (`MissingHeader`), **garde de surcomptage** (`HeaderSkipsData`, AC5), fichier vide (`EmptyFile`), indices
  hors bornes du premier enregistrement (et, au décodage, `DecodingFailed`) ; (1) `EncodingMismatch` — il sort avant toute lecture de ligne
  (`parser.rs` l. ≈ 137-146) ; en import sans `confirmEncodingMismatch` il l'emporte, et le refus de signe n'apparaît
  qu'au passage suivant ; en aperçu, la nouvelle tentative forcée (`bank_imports.rs` l. ≈ 1644-1661) rend le refus de
  signe et l'avertissement d'encodage n'est plus montré — comportement accepté, écrit ici ; (2) puis le signe ;
  (3) puis les autres erreurs de ligne. Sur **une même** ligne, `parse_row` lit la date avant le montant : une date
  illisible rend `INVALID_DATE` et le signe de cette ligne n'est pas examiné.
- `kesh-api` : bras **explicite** dans `map_csv_error` → `AppError::BankCsvAmountSignMismatch { lines, total_errors,
  truncated, whole_debit_column }` → **`422 BANK_CSV_AMOUNT_SIGN_MISMATCH`**, `details: { lines: [ … ], totalErrors,
  truncated, wholeDebitColumn }` (même forme de ligne que `details.lines` de `BANK_CSV_PARTIAL_FAILURE` — variant
  `AppError::BankCsvParsePartialFailure` —, via `csv_line_error_to_payload`). **Deux messages** (C-26-1-23) :
  - `wholeDebitColumn = true` → `bank-import-csv-errors-amount-sign-refused-profile` (sans argument) : « Tous les
    débits lus dans le fichier ont un signe contraire au réglage du profil. Réglez le sens des débits du profil (Administration
    → Profils bancaires), puis redéposez le fichier. » (arbitrage (1) de la P3 : « redéposez », la procédure d'import
    appartenant à la 26-3) ;
  - sinon → `bank-import-csv-errors-amount-sign-refused-lines`, `{ $count }` : « { $count } ligne(s) portent un
    montant de signe contraire à leur colonne (une extourne, par exemple) : ce relevé ne peut pas être importé tel
    quel dans cette version. Les lignes en cause sont listées ci-dessous. »
  Rendu **en preview comme en import**, que `confirmPartialImport` vaille `true` ou `false` — les deux chemins testés
  (AC15).
- Écran (`BankImportUpload.svelte`, R2-2 = F2-4, arbitrage (2)) : aujourd'hui le panneau d'erreur
  (`data-testid="bank-import-error"`, l. ≈ 273-282) n'affiche que `errorMessage` et ne lit pas `details` (porté par
  `api-client.ts` l. ≈ 243). Pour `errorCode === 'BANK_CSV_AMOUNT_SIGN_MISMATCH'`, il garde `details` dans un état
  `errorDetails` et **liste les lignes en cause** — numéro et message traduit par ligne (AC3-bis), valeur — sous le
  message global, avec la mention de troncature si `truncated`, par la clé **existante**
  `bank-import-warnings-invalid-lines-truncated` (`BankImportUpload.svelte` l. ≈ 424 ; pas de clé neuve)
  (`data-testid="bank-import-error-line"` par ligne). ⚠️ **Deux sites de capture, trois de remise à zéro** (R3-13) :
  `errorDetails` est posé dans les **deux** `catch` qui posent `errorCode` (`handleFileSelect` l. ≈ 118-127 et
  `handleConfirm` l. ≈ 148-157 — le refus peut arriver à la confirmation) et remis à `null` partout où `errorCode`
  l'est (`reset()`, `handleFileSelect`, `handleConfirm`). **Aucune
  case de confirmation** n'est offerte pour ce code (la case actuelle dépend de `preview.warnings.invalidLines`,
  l. ≈ 190, 409-456, jamais d'un 422 — le garder ainsi).

⚠️ **Changement de comportement assumé** : sous le défaut `Positive`, un débit négatif — aujourd'hui inversé en
silence — refuse désormais l'import. C'est #620 rendu visible à qui n'a pas réglé son profil. Le doc-comment de
`CsvLineErrorCode::AmountSignMismatch` et celui de `CsvError::AmountSignMismatch` disent pourquoi.

**AC3-bis — Le code d'erreur de ligne sort sous sa graphie canonique, et l'écran en affiche le message (F1/R-2,
C-26-1-14).** Aujourd'hui `csv_line_error_to_payload` (`routes/bank_imports.rs` ≈ l. 613) émet
`format!("{:?}", e.code).to_uppercase()` — `INVALIDDATE`, `AMBIGUOUSDEBITCREDIT`… — et le panneau
(`BankImportUpload.svelte` ≈ l. 443) n'affiche que ce code brut ; `messageI18nKey` est typé
(`bank-import.types.ts:80`) et lu nulle part.

- `CsvLineErrorCode::as_str(&self) -> &'static str` (`match` exhaustif, sans bras par défaut) rend la graphie
  `SCREAMING_SNAKE_CASE` (`INVALID_DATE`, `INVALID_AMOUNT`, `AMBIGUOUS_DEBIT_CREDIT`, `EMPTY_MANDATORY_FIELD`,
  `ROW_TOO_SHORT`, `AMOUNT_SIGN_MISMATCH`) ; `csv_line_error_to_payload` l'emploie. Un test unitaire vérifie, **pour
  chaque variant**, que `as_str()` égale `serde_json::to_value(code)` (les deux graphies ne peuvent plus diverger) ;
  un test d'intégration (AC15) asserte la valeur exacte `details.lines[0].code` sur le fil.
- Le panneau des lignes invalides (aperçu, `warnings.invalidLines`) **et** la liste du refus de signe (AC3)
  affichent, pour chaque ligne, le **message traduit** à côté du code — **par une table TS littérale**, pas par
  `i18nMsg(line.messageI18nKey, …)` (F2-2, arbitrage (5)) : un `i18nMsg(` dont la clé n'est pas littérale est un
  **site non résolu** pour `frontend/src/lib/shared/i18n-keys.test.ts` (l. ≈ 797, 942-950), qui interdit de
  simplement le recompter. Nouveau module `frontend/src/lib/features/bank-import/csv-line-error-label.ts`, patron de
  `imported-supplier-invoices/error-label.ts` : `export const CSV_LINE_ERROR_LABELS: Record<string, [string,
  string]>` (code → `[suffixe, repli français]`, six entrées : `invalid-date`, `invalid-amount`,
  `ambiguous-debit-credit`, `empty-mandatory-field`, `row-too-short`, `amount-sign-mismatch`) et
  `csvLineErrorLabel(code)` → `i18nMsg(\`bank-import-csv-errors-${suffixe}\`, repli)`, repli sur le code brut pour un
  code inconnu. Dans `i18n-keys.test.ts` (R3-4 = F3-3) : la famille `bank-import-csv-errors-` déclarée dans
  `MOTIFS_DYNAMIQUES` (l. ≈ 580) avec ses six suffixes, `CARDINALITES` (l. ≈ 639) **lue depuis la production** —
  `Object.keys(CSV_LINE_ERROR_LABELS).length`, patron de `imported-supplier-invoices-error-` (l. ≈ 646), jamais un 6
  recopié ; une entrée `'lib/features/bank-import/csv-line-error-label.ts  bank-import-csv-errors-'` (deux espaces
  avant le préfixe) dans **`SITES_GABARIT_ATTENDUS`** (l. ≈ 714) — c'est un **site de gabarit**
  (`kind === 'template'`), pas un relais ni un site non résolu. Attendu : `ATTENDU.sitesGabarit` **10 → 11**,
  `sitesTotal` augmenté des `i18nMsg(` neufs, `relais` (6) et `sitesNonResolus` (31) **inchangés** — à **mesurer**,
  pas à croire. Les nombres en dur des titres suivent : docblock « Les 8 préfixes dynamiques » (l. ≈ 562) → 9,
  « Les 10 sites de gabarit » (l. ≈ 708) → 11, `it('les 8 préfixes dynamiques sont déclarés, et leurs 10 sites
  confrontés'` (l. ≈ 925) → 9 et 11 (greper `\b(8|10)\b` dans le fichier et trier). **Chaque** nombre changé reçoit
  sa ligne d'historique « Story 26-1 : … » (patron l. ≈ 393-540) — jamais un nombre ajusté pour faire passer. `messageI18nKey` reste sur le
  fil (champ de l'API), il n'est simplement pas lu par l'écran. Un Vitest `csv-line-error-label.test.ts` (patron
  `reminder-error-label.test.ts`) : chaque code rend sa clé ; code inconnu → code brut. Les en-têtes en dur du
  tableau (« Ligne », « Code », « Valeur ») restent hors périmètre.
- **Changement d'API visible** : les cinq codes existants changent de graphie dans `warnings.invalidLines.lines[].code`
  et `details.lines[].code`. Vérifié le 2026-10-10 : aucun consommateur dans le dépôt
  (`grep -rnE "INVALIDDATE|INVALIDAMOUNT|AMBIGUOUSDEBIT|EMPTYMANDATORY|ROWTOOSHORT" crates frontend docs` → 0 hors
  fiche) ; `docs/api-external.md` ne documente ni ces codes ni les routes d'import CSV (seul `GET
  /api/v1/bank-imports/{id}`, `matchedEntryId`, l. ≈ 561) — **rien à y changer**, le re-vérifier. À consigner au
  CHANGELOG, section `### Modifié` (AC18).

### Les lignes à ignorer (#621)

**AC4 — Plafond 20, lignes vides non comptées (C-26-1-6, C-26-1-7).**

- `CsvProfile::validate` : `header_row_count > 20` → `ProfileMisconfigured("header_row_count {n} > 20")` ; le
  doc-module (`header_row_count ≤ 5`) et le doc du champ (`(0..=5)`) passent à 20. Le test
  `rejects_header_row_count_over_5` devient `rejects_header_row_count_over_20` (21 refusé) et un test accepte 20 et 9.
- **Sémantique documentée, non changée** : le réglage compte les **lignes non vides** avant les données, **ligne de
  titres comprise** ; une ligne **vide** (aucun caractère) ne compte pas, parce que le lecteur `csv` l'ignore
  (csv-core 0.1.13, `src/reader.rs` l. 94). ⚠️ Une ligne qui ne contient que des espaces ou des `;` **n'est pas** vide
  et compte. Le doc-comment du champ `header_row_count` et celui de l'étape 5 du pipeline le disent. **Fichier UBS :
  9.**
- **Numéros de ligne — sémantique réelle, non changée (F3/R-1, C-26-1-16)** : le numéro rapporté est celui de la ligne
  **où commence la lecture** de l'enregistrement. `csv` 1.4.0 pose la position **avant** la lecture
  (`src/reader.rs` l. 1626) et csv-core n'avale la ligne vide qu'**en** lisant l'enregistrement suivant
  (`reader.rs` l. 94, 665) : un enregistrement **précédé d'une ou plusieurs lignes vides est rapporté à la première
  d'entre elles**. Tout autre enregistrement porte son numéro physique (celui d'un éditeur). Sur le fichier UBS : la
  ligne de titres (physique 10) est rapportée **9** ; les transactions (11 et suivantes, sans ligne vide devant)
  portent leur numéro exact. On ne corrige **pas** ce décalage (il faudrait recalculer les positions hors du
  lecteur) : on le **fixe par un test** (AC14-4) et on le **documente** (doc-comment de `CsvLineError::line`, manuel :
  « une ligne qui suit une ligne vide peut être annoncée au numéro de cette ligne vide »). ⛔ Écrire le test qui fixe
  le numéro **avant** de le promettre au manuel.

**AC5 — Garde contre le surcomptage (C-26-1-8).** Quand `header_row_count ≥ 1`, la **dernière ligne ignorée** —
`reader.headers()` si `header_row_count == 1`, sinon le dernier enregistrement consommé par la boucle de saut — est
examinée : si elle a au moins `column_mapping.date + 1` champs et que sa cellule `date` (trimée) se lit avec
`NaiveDate::parse_from_str(…, &profile.date_format)`, le parseur échoue avec la nouvelle variante
`CsvError::HeaderSkipsData { line: usize }` (`#[error]` en français, documentée). `line` = la position de cet
enregistrement, **même sémantique qu'AC4** : pour `header_row_count == 1`, lire la position portée par l'enregistrement
d'en-têtes lui-même (`reader.headers()?.position()`, `csv` l. 1323-1326) — **pas** `reader.position()` après lecture,
qui vaut la ligne suivante (F12, C-26-1-21). **Position dans le code** (R3-10, F3-9) : la garde s'exécute **juste après
la boucle de saut, avant** le `iter.next()` qui lit le premier enregistrement de données — sinon, sur un fichier dont
la seule transaction est avalée, `EmptyFile` répondrait avant elle. Ce n'est pas une détection de la ligne de titres : le nombre reste celui
du profil (décision 2).

⚠️ **Angle mort assumé (R-12, C-26-1-17)** : la garde ne voit qu'une ligne avalée **dont la cellule date se lit**.
Un surcomptage qui avale une transaction à date **vide**, **illisible** ou dont la ligne a **moins de
`date + 1` champs** reste muet — plus aucune erreur `INVALID_DATE` n'est produite pour elle, puisqu'elle est dans le
saut. Sur un relevé bancaire (date toujours présente), c'est rare ; c'est écrit au doc-comment de `HeaderSkipsData` et
au manuel (« la garde ne protège que contre une ligne datée »), pour ne pas laisser croire que plus rien ne disparaît.
Un test fixe l'angle mort (ligne avalée à date vide → pas d'erreur, une transaction de moins) — il documente, il ne
promet pas.

- `kesh-api` : `AppError::BankCsvHeaderSkipsData { line }` → **`422 BANK_CSV_HEADER_SKIPS_DATA`**, message traduit
  `bank-import-csv-errors-header-skips-data` (argument `$line`), `details: { "line": <n> }` — modèle :
  `AppError::BankCsvEmptyFile` (`crates/kesh-api/src/errors.rs`). Le `match` de conversion `CsvError → AppError`
  (`routes/bank_imports.rs` ≈ l. 630-654) gagne le bras **explicite** (pas le `other =>` générique, qui rendrait un 500
  au message masqué).
- Les deux chemins `preview` et `create` sont couverts (ils partagent la conversion — le vérifier, ne pas le supposer).

### La base, la version, la persistance

**AC6 — Migration `crates/kesh-db/migrations/20261011000001_bank_profiles_debit_sign.sql` (C-26-1-3, C-26-1-4,
C-26-1-6, C-26-1-18).** Horodatage **attribué** par l'orchestrateur : `20261011000001` à la 26-1, `20261011000002` à
la 26-2 ; la 26-1 fusionne **en premier**, la 26-2 recompte après rebase.

```sql
-- BREAKING (P1) : relève kesh_version_min_required à 0.13.1 (dernière instruction).
-- (en-tête : pourquoi — une 0.13.0 ignorerait debit_sign = 'negative' et inverserait
--  chaque débit d'un profil UBS en silence : le P1 #620 rouvert par un retour en arrière)
ALTER TABLE bank_profiles
    ADD COLUMN debit_sign VARCHAR(10) NOT NULL DEFAULT 'positive'
        COMMENT 'Colonne Débit (mode séparé). positive = sans signe, Kesh inverse. negative = déjà négatif, Kesh garde'
        AFTER header_row_count,
    ADD CONSTRAINT chk_bank_profiles_debit_sign
        CHECK (debit_sign IN ('positive', 'negative'));
ALTER TABLE bank_profiles DROP CONSTRAINT chk_bank_profiles_header_row_count;
ALTER TABLE bank_profiles ADD CONSTRAINT chk_bank_profiles_header_row_count
        CHECK (header_row_count <= 20);
UPDATE _kesh_version SET kesh_version_min_required = '0.13.1' WHERE id = 1;
```

(Forme exacte libre — notamment si MariaDB accepte `DROP CONSTRAINT` + `ADD CONSTRAINT` du même nom dans un seul
`ALTER` ; le vérifier contre la base de dev, ne pas le supposer.) **Aucune écriture de données applicatives** : les
profils existants prennent `positive` par le `DEFAULT`, c'est-à-dire **exactement** leur comportement d'avant, et leur
`header_row_count` (≤ 5) reste valide sous la nouvelle borne.

⛔ **Aucun `;` ni `--` dans un littéral de la migration** (`COMMENT` compris — R-7, C-26-1-19) : le découpeur de
`post_restore.rs` (`split_statements`, l. ≈ 736-753) coupe au premier `;` sans tenir compte des apostrophes, et le
détecteur P7 verrait des fragments faux.

⚠️ **Horodatage** : `20261011000001` est attribué à la 26-1, `20261011000002` à la 26-2 (C-26-1-18) ; ne jamais
renommer une migration déjà appliquée sur une base de dev (P8).

Garde-fous de la § *Migration breaking policy*, tous à exécuter :

- **P2 / P2-bis** : dernière instruction = le bump ; **dans le même commit**, les **dix** `crates/*/Cargo.toml` passent
  à `version = "0.13.1"` et `Cargo.lock` suit (patron `0724904c`, Story 15-1a-i). Recompter :
  `grep -h '^version = ' crates/*/Cargo.toml | sort | uniq -c` → `10 version = "0.13.1"`. ⛔ **Pas de remplacement
  global** de `0.13.0` (F3-6) : deux dépendances tierces portent cette version — `yaml-rust2 = "0.13.0"`
  (`crates/kesh-api/Cargo.toml` l. ≈ 103) et `ravif` (dans `Cargo.lock`) ; seules les lignes **ancrées** `^version = `
  des dix crates bougent, puis `Cargo.lock` se met à jour par un build (ou `cargo update -w`).
- **P3** : sans objet mécaniquement (aucune opération listée) ; le bump est fait **quand même**, pour la raison de
  l'en-tête (C-26-1-4).
- **P5** : une ligne au tableau de `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx` ou `no` à
  justifier — le `ADD COLUMN` n'est pas rejouable) ; l'en-tête `## Table d'audit (N migrations)`, la ligne `Total`
  (avec « + 1 Story 26-1 ») et le compteur de partition concerné, **recomptés** :
  `ls crates/kesh-db/migrations/*.sql | wc -l` et `grep -c '^| \`20' docs/migrations-idempotence-audit.md` —
  **78 avant** (base `fe162504`, recompté le 2026-10-10), **79 après** la 26-1 ; 80 après l'union avec la 26-2, que
  la 26-2 recomptera à son rebase (R-11, F13).
- **Registre des checksums `crates/kesh-db/migrations.sha384`** (R-3) : ajouter la ligne
  `20261011000001 <sha384sum du fichier>` (format des lignes existantes ; calcul : `sha384sum <fichier> | cut -d' '
  -f1`), **après la dernière modification du fichier** — l'y inscrire, c'est le déclarer publié (P8) ; ⚠️ **jusqu'au
  merge, la ligne suit le fichier** (F2-12, C-26-1-24) : si la boucle de revue de code retouche la migration, la
  retouche est **délibérée** (bases du worktree reconstruites, P8), la ligne `sha384` est **recalculée dans le même
  commit**, et le Change Log le dit — l'en-tête du registre (« ne se MODIFIE jamais ») vaut pour une migration publiée,
  pas pour une branche non fusionnée ; le test
  `published_migrations_keep_their_checksums` (`test_schema_guard.rs` ≈ l. 562, 648-654) rougit sinon. Au rebase de la
  26-2 : deux lignes, union.
- **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et inspection de chaque site
  (`crates/kesh-db/tests/migrations_upgrade_path.rs`, `lettering_documents_backfill.rs`,
  `audit_log_company_id_backfill.rs`, `common/mod.rs`) ; un site positionnel sans garde fail-loud est un MEDIUM.
  **Inventaire complet** (R2-5, F2-7) : `grep -nE "\b(44|78)\b" crates/kesh-db/tests/migrations_upgrade_path.rs`
  et **trier chaque occurrence** (au 2026-10-10 : l. ≈ 38, 40, 57-58, 107, 110-111, 115, 123, 130, 133, 185, 192, 195,
  264 — commentaires, doc-comment, message d'`assert!` et code) ; `common/mod.rs` n'en contient **aucune**. Greper
  **aussi** `\b(43|35)\b` (F3-9) : la phrase l. ≈ 132-133 (« Le laisser à 43 avec `total = 78` porterait la frontière à
  35 ») se périme avec le reste (après 26-1 : « le laisser à 44 avec `total = 79` porterait la frontière à 35 »).
  **Deux littéraux nommés, à bouger du même pas** (F6/R-4) dans `crates/kesh-db/tests/migrations_upgrade_path.rs` :
  `assert_eq!(total, 78, …)` (l. ≈ 110, et son message « 78 migrations attendues … ») → **79**, et
  `let n_before_upgrade_window = total - 44;` (l. ≈ 194, et `.expect("apply_migrations_up_to(total - 44) failed")`)
  → **`total - 45`**, frontière **inchangée** (79 − 45 = 34) ; une ligne d'historique au commentaire (« Story 26-1 :
  44 → 45, frontière inchangée (79 - 45 = 34) ») et le texte de l'`assert!` de la fonction **locale**
  `apply_migrations_up_to` de `migrations_upgrade_path.rs` (l. ≈ 34-46 : `total - 44`, `total == 78`) suivent. Bumper `total` seul ferait glisser la frontière et rougir sur `COUNT(accounts)`, échec
  sans rapport apparent avec sa cause (le fichier le consigne). Après union avec la 26-2 : 80 et 46, à la 26-2.
- **P7** : le seul statement d'écriture est le bump `_kesh_version` → entrée `EXEMPT_MIGRATIONS`
  (`crates/kesh-db/src/post_restore.rs`), `ExemptionBasis::Durable`, justification sur le patron de `20261009000001`
  (« UPDATE sur _kesh_version (bump kesh_version_min_required), table système jamais restaurée. Le reste de la
  migration est du DDL pur … une sauvegarde antérieure restaure la colonne à son DEFAULT 'positive', qui est la
  sémantique des profils d'avant »). Le compteur `assert_eq!(EXEMPT_MIGRATIONS.len(), 17, …)` passe à **18**, avec sa
  ligne d'historique (« 17 → 18 (Story 26-1, #620) »). Le test `every_data_backfill_migration_is_triaged` doit être
  vert.
- **P8** : la migration, une fois appliquée à une base de dev, **ne se modifie plus** — pas même un commentaire.
- **Squash de test** : `scripts/regen-test-schema.sh` (base jetable **propre au worktree**, consigne 6) régénère
  `crates/kesh-db/test-schema/0001_schema_squash.sql` (colonne, deux CHECK, `_kesh_version` à `'0.13.1'`) ;
  `test_schema_guard` vert.

**AC7 — Les assertions de version suivent.** `crates/kesh-db/tests/migrations_fresh_install.rs`
(`migrations_kesh_version_initial_row` : `"0.13.1"`, doc-comment complété de la généalogie « 0.13.1 depuis la Story
26-1 ») et `crates/kesh-db/tests/migrations_upgrade_path.rs` (≈ l. 516-536 : binaire `0.13.1` aligné ; binaire
`0.13.0` désormais **refusé** — ajouter ce cas ; binaire `0.14.0` en avance avec `db_min = 0.13.1`). **Greper la
valeur** : `grep -rnE "0\.13\.0" crates/kesh-db/tests crates/kesh-api/tests crates/kesh-db/src` et trier chaque
occurrence (les généalogies historiques restent).

**AC8 — Entité et dépôt.** `BankProfile` et `NewBankProfile` (`crates/kesh-db/src/entities/bank_profile.rs`) gagnent
`pub debit_sign: String` (la colonne `VARCHAR` se lit en `String`, comme `decimal_separator`) **et** une méthode
`debit_sign_enum(&self) -> Result<DebitSign, DbError>` qui rend `DbError::Invariant` sur une valeur hors domaine (pas
de repli silencieux sur `positive` : un repli rouvrirait #620 sur une base corrompue). `crates/kesh-db/src/repositories/
bank_profiles.rs` : `COLUMNS`, `INSERT` (`create`) et `UPDATE` (`update`) portent la colonne ; le calcul
`fields_changed` de l'audit de `update` gagne `"debit_sign"` ; le test
`update_full_replacement_writes_audit_log_with_fields_changed` vérifie que `debit_sign` modifié y figure ; un test
crée un profil `header_row_count = 9`, `debit_sign = "negative"` et le relit à l'identique ; un test vérifie que la
base **refuse** `header_row_count = 21` et `debit_sign = 'autre'` par ses CHECK (`DbError::CheckConstraintViolation`) ;
un test unitaire sur une entité **construite à la main** (`debit_sign = "autre"`, la base l'interdisant) vérifie que
`debit_sign_enum()` rend `DbError::Invariant` (R3-6 — c'est le test de M10). Le helper `make_new_profile`
(`crates/kesh-db/src/repositories/bank_profiles.rs` l. ≈ 358) gagne le champ, comme les autres littéraux
`NewBankProfile { … }` (`grep -rn "NewBankProfile\s*{" crates` : routes, `bank_imports_e2e.rs` l. ≈ 691, ce helper).
Les `INSERT` SQL bruts des tests (`crates/kesh-db/tests/companies_repository.rs` ≈ l. 938,
`crates/kesh-api/tests/exports_global_e2e.rs` ≈ l. 1566) restent valides par le `DEFAULT` — ne les toucher que si un
test l'exige.

**AC9 — Le profil d'avant reste ce qu'il était (preuve par la migration).** Un test **dans
`crates/kesh-db/tests/migrations_upgrade_path.rs`** — fichier déjà inscrit à `ALLOWED_REAL_MIGRATOR_FILES`
(`test_schema_guard.rs` l. ≈ 38-62), ce qui évite d'élargir cette liste (F7, C-26-1-20). ⚠️ Ce fichier **ne déclare
pas** `mod common;` et définit sa **propre** `apply_migrations_up_to` (R2-5, F2-7), qui porte la garde de frontière :
on la garde. Pour résoudre l'index **par version** sans dupliquer `common::migrations_before` (DRY, R3-5,
C-26-1-26) : `#[allow(dead_code)] mod common;` (l'attribut, posé sur la seule déclaration, couvre
`common::apply_migrations_up_to`, inutilisée ici), et appel **qualifié** `common::migrations_before(20261011000001,
"bank_profiles_debit_sign")` — jamais `use common::*` (collision avec la fonction locale). Le test applique les
migrations **jusqu'à la précédente** (`apply_migrations_up_to(&pool, common::migrations_before(…))`, résolution **par
version**, P6), insère un profil `header_row_count = 5` en mode
séparé, applique le reste, et vérifie : `debit_sign = 'positive'`, `header_row_count = 5`, `column_mapping` intact, et
`kesh_version_min_required = '0.13.1'`. Une assertion de montage vérifie que la colonne **n'existe pas** avant
(`information_schema.COLUMNS`) — sans quoi le test passerait à vide (précédent 16-1a).

### L'API, les exports

**AC10 — DTO et routes (`crates/kesh-api/src/routes/bank_profiles.rs`).**

- `CreateOrUpdateBankProfileRequest` gagne `pub debit_sign: DebitSign` (`debitSign` en JSON), **sans
  `#[serde(default)]`** (C-26-1-9). `to_csv_profile` et `to_new_bank_profile` le portent.
- **Statuts réels** (F2, C-26-1-21 — la P1 a relevé que la fiche annonçait des 400 et un code inexistant) :
  - `negative` en colonne unique, `headerRowCount: 21` → passent par `validate()` → `AppError::BankCsvProfileValidation`
    → **`422 BANK_CSV_PROFILE_INVALID`** (`errors.rs` l. ≈ 2280-2281 ; tests voisins `bank_profiles_e2e.rs`
    l. ≈ 230, 253-338) ;
  - requête **sans** `debitSign`, ou `debitSign: "autre"` → rejet de l'extracteur `Json` nu d'axum : **422**, corps
    **texte brut** hors enveloppe `{ error: … }` — c'est le comportement actuel de **toutes** les erreurs de
    désérialisation de cette route (`headerRowCount: -1`, champ inconnu…), qu'on **décrit** sans le changer : le
    formulaire envoie toujours le champ, seul un client d'API peut l'omettre, et l'extracteur à rejet géré
    (`impl_validation_extractor!`) est local à `bank_accounts.rs` — le partager élargirait la story. Le test asserte
    `422` et l'absence d'écriture en base, **pas** le corps.
- `BankProfileResponse` gagne `debit_sign: DebitSign` (`debitSign`), lu par `debit_sign_enum()` (une valeur hors
  domaine en base → 500 `Invariant`, jamais un repli).
- `db_profile_to_csv_profile` (`routes/bank_imports.rs` ≈ l. 1574) porte le réglage — **c'est le site qui fait passer le
  réglage du formulaire à l'import** ; un oubli ici et tout le reste est vert pour rien (l'E2E AC16 et le test
  d'intégration AC15 le gardent).
- Doc-module de `bank_profiles.rs` : la validation liste les deux règles nouvelles.

**AC11 — Exports.** `serialize_bank_profiles_csv` (`crates/kesh-api/src/exports/csv_tables.rs` ≈ l. 1036) écrit la
colonne `debit_sign` **juste après** `header_row_count` (ordre du schéma, `AFTER header_row_count`) ; le test qui
compare les en-têtes CSV aux colonnes réelles (≈ l. 2229) doit passer sans exception ajoutée. Le `.keshbackup` est
générique (`crates/kesh-db/src/backup.rs`, colonnes lues dans `information_schema`) : **aucun code**, mais un test sur
le patron de `column_constraints_flags_required_and_optional` établit que `bank_profiles.debit_sign` **n'est pas
requise** (`is_required() == false`, `DEFAULT` présent) — c'est ce qui rend importable une sauvegarde 0.13.0, restaurée
à `positive`. Une sauvegarde 0.13.1 dans une 0.13.0 est refusée par le manifeste (`min_required = 0.13.1`) — rien à
coder, à écrire au CHANGELOG (AC18).

### L'écran, les textes

**AC12 — Formulaire (`frontend/src/lib/features/bank-import/BankProfileForm.svelte`) et types.**

- `bank-profile.types.ts` : `export type DebitSign = 'positive' | 'negative'` ; `BankProfile.debitSign: DebitSign`
  (donc dans `CreateOrUpdateBankProfileRequest`).
- Le champ « lignes à ignorer » : `max="20"`, libellé et texte d'aide traduits (AC13), le texte d'aide disant
  *ligne de titres comprise, lignes vides non comptées*. `data-testid="header-row-count-input"` inchangé.
- **Quand « Colonnes débit/crédit séparées » est cochée**, et seulement alors : un groupe de deux boutons radio
  (`<fieldset>` + `<legend>` traduite — accessibilité, la spec axe doit rester à zéro violation) :
  « Débits écrits sans signe (500.00) » (`value="positive"`, `data-testid="debit-sign-positive"`) / « Débits écrits avec
  un signe moins (−500.00), comme UBS » (`value="negative"`, `data-testid="debit-sign-negative"`). Initialisé depuis
  `existing?.debitSign ?? 'positive'`.
- La requête envoie `debitSign` : la valeur choisie en mode séparé, **`'positive'` en mode colonne unique** (sinon le
  serveur refuse, AC1). Décocher puis recocher le mode séparé garde le choix fait (état local).
- **Vitest** (nouveau `BankProfileForm.test.ts` ; aucun test de composant n'existe dans `features/bank-import/` —
  patrons ailleurs : `features/reconciliation/TransactionSplitModal.test.ts`, `features/reports/AgedReceivablesView.test.ts`,
  `@testing-library/svelte` configuré dans `vite.config.ts`, R3-13) : (a) mode séparé + « signe moins » → la requête porte `debitSign: 'negative'` et
  `debit_credit_split` ; (b) mode colonne unique → `debitSign: 'positive'` même après avoir choisi « signe moins » puis
  décoché ; (c) un profil existant `negative` s'ouvre avec le bon bouton coché ; (d) le champ des lignes porte
  `max="20"` ; (e) « signe moins », décocher, **recocher** → `debit-sign-negative` coché et requête `'negative'`
  (R2-14).

**AC13 — Textes, quatre locales** (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`), clés présentes
dans les **quatre** fichiers (le test `frontend/src/lib/shared/i18n-keys.test.ts` et `npm run lint-i18n-ownership`
verts) :

| Clé | Rôle |
|---|---|
| `bank-import-profile-labels-header-row-count` | **modifiée** : « Lignes à ignorer avant les données (0-20) » (plus de « header », plus de « 0-5 ») |
| `bank-import-profile-labels-header-row-count-help` | neuve : ligne de titres comprise, lignes vides non comptées ; exemple UBS : 9 |
| `bank-import-profile-labels-debit-sign` | neuve : légende du groupe |
| `bank-import-profile-labels-debit-sign-positive` | neuve |
| `bank-import-profile-labels-debit-sign-negative` | neuve |
| `bank-import-profile-errors-debit-sign-single-column` | neuve, si l'écran l'affiche (sinon ne pas la créer) |
| `bank-import-csv-errors-amount-sign-mismatch` | neuve (AC3, message **par ligne**, affiché par AC3-bis) : « Montant de signe contraire à sa colonne — vérifier le réglage du sens des débits du profil. » |
| `bank-import-csv-errors-amount-sign-refused-profile` | neuve (AC3, refus global quand **tous** les débits lus — au moins deux — sont en défaut, `whole_debit_column` : « réglez le sens des débits du profil »), sans argument |
| `bank-import-csv-errors-amount-sign-refused-lines` | neuve (AC3, refus global d'une ou de quelques lignes — extourne —, `{ $count }`, renvoie à la liste des lignes) |
| `bank-import-csv-errors-header-skips-data` | neuve (AC5), avec `{ $line }` : la ligne N, ignorée, porte déjà une date ; le nombre de lignes à ignorer est trop grand |

Les **valeurs de repli** françaises passées à `i18nMsg(…, '<repli>')` dans le Svelte sont identiques aux textes
`fr-CH` (le repli `'Nb lignes header (0-5)'` disparaît). **Greper la valeur** après coup :
`grep -rnE "0-5|0–5|\(0-5\)|max=\"5\"" frontend/src crates/kesh-i18n docs/manual/fr/*.tex` → zéro occurrence liée aux
profils.

### Les preuves

**AC14 — Fixture synthétique UBS (C-26-1-1, C-26-1-12).** Nouveau fichier
`crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv`, **inventé de toutes pièces**, de structure **identique**
au fichier UBS : octets `EF BB BF` en tête, fins de ligne **LF**, **aucun LF après la dernière ligne** (comme le
fichier réel — un dernier enregistrement sans terminateur est un cas distinct du lecteur, R-8/F9), et exactement ce
contenu (aucun espace en fin de ligne ; la ligne 9 est **vide**) :

```
Numéro de compte:;0000 00000000.0;
IBAN:;CH93 0076 2011 6238 5295 7;
Du:;2026-01-06;
Au:;2026-03-02;
Solde initial:;5000.00;
Solde final:;4763.75;
Évaluation en:;CHF;
Nombre de transactions dans cette période:;11;

Date de transaction;Heure de transaction;Date de comptabilisation;Date de valeur;Devise;Débit;Crédit;Sous-montant;Solde;No de transaction;Description1;Description2;Description3;Notes de bas de page;
2026-03-02;;2026-03-02;2026-03-02;CHF;-199.95;;;4763.75;9900111ZZ0000011;Électricité Fictive SA;Décompte annuel;eBill;;
2026-02-22;;2026-02-24;2026-02-24;CHF;-22.80;;;4963.70;9900110ZZ0000010;Pharmacie Exemple;;Paiement carte;;
2026-02-20;;2026-02-20;2026-02-20;CHF;-980.00;;;4986.50;9900109ZZ0000009;Garage Imaginaire;Facture 2026-014;"Paiement QR; référence fictive";;
2026-02-16;;2026-02-16;2026-02-16;CHF;;150.25;;5966.50;9900108ZZ0000008;Assurance Fictive;Remboursement;Virement entrant;;
2026-02-07;;2026-02-09;2026-02-09;CHF;-15.00;;;5816.25;9900107ZZ0000007;Transports Publics Exemple;;Paiement carte;;
2026-01-31;;2026-02-02;2026-02-02;CHF;-64.35;;;5831.25;9900106ZZ0000006;"Café du Marché; Sàrl";;Paiement carte;;
2026-01-26;;2026-01-26;2026-01-26;CHF;-300.00;;;5895.60;9900105ZZ0000005;Caisse maladie Exemple;Prime janvier;eBill;;
2026-01-17;;2026-01-19;2026-01-19;CHF;-8.90;;;6195.60;9900104ZZ0000004;Kiosque de la Gare;;Paiement carte;;
2026-01-12;;2026-01-12;2026-01-12;CHF;-1250.00;;;6204.50;9900103ZZ0000003;Régie Fictive SA;Loyer janvier;Ordre permanent;;
2026-01-08;;2026-01-08;2026-01-08;CHF;;2500.00;;7454.50;9900102ZZ0000002;Exemple SA;Salaire janvier;Virement entrant;;
2026-01-04;;2026-01-06;2026-01-06;CHF;-45.50;;;4954.50;9900101ZZ0000001;Boulangerie du Lac;;Paiement carte;;
```

Invariants (recalculés lors de la fixation, à **re-vérifier** par le test, pas à croire) : 11 transactions, **9
sorties** et **2 entrées**, somme **−236.25** = 4763.75 − 5000.00 ; la colonne Solde est le solde courant après chaque
ligne. L'IBAN est l'exemple public des documentations bancaires suisses (`CH9300762011623852957`), aucun compte réel.
Les paiements par carte ont une **date de transaction antérieure** à la date de comptabilisation : c'est ce qui
discrimine le mapping `date = 2` (C-26-1-13). `crates/kesh-import/tests/fixtures/README.md` gagne une section « CSV »
(provenance synthétique, structure UBS reproduite, invariants).

**Tests du parseur** (`parser.rs`, module `tests`, fixture par `include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
"/tests/fixtures/csv/ubs_synthetique.csv"))`), profil UBS de C-26-1-13 :

1. `Negative` → `AllValid`, 11 transactions, 9 montants `< 0`, 2 `> 0`, somme `dec!(-236.25)` ; **et** la somme se
   compare à `solde final − solde initial` **lus dans le préambule de la fixture par le test** (deux constantes en dur
   ne prouveraient rien) ;
2. `period_from = 2026-01-06`, `period_to = 2026-03-02` (colonne 2, pas la colonne 0 qui donnerait `2026-01-04`) ;
3. même fichier sous `Positive` → `HardFailure(CsvError::AmountSignMismatch { errors, total_errors: 9, truncated:
   false, whole_debit_column: true })`, `errors` ne portant **que** des `AmountSignMismatch` (9) — et **aucune** transaction rendue : #620 ne
   peut plus passer, ni en silence ni par l'import partiel (C-26-1-15) ; un cas en ligne mêlant une erreur de signe et
   une `INVALID_DATE` rend aussi `AmountSignMismatch` (le signe domine) ;
4. `header_row_count = 8` → la ligne de titres arrive en données : erreur `INVALID_DATE` rapportée à la **ligne 9**
   (la ligne vide qui la précède — sémantique d'AC4, C-26-1-16 ; la ligne physique est 10) ; et un cas en ligne **sans**
   ligne vide où l'erreur porte son numéro physique exact ;
5. `header_row_count = 10` → `HardFailure(HeaderSkipsData { line: 11 })` (la première transaction aurait disparu ;
   11 car aucune ligne vide ne la précède) ;
6. `header_row_count = 9`, `0`, `1` sur de petits CSV en ligne (pas la fixture) : garde AC5 muette quand la dernière
   ligne ignorée est un titre ; `header_row_count = 0` n'examine rien ; **`header_row_count = 1` sur un CSV sans
   titres** (première ligne datée) → `HeaderSkipsData { line: 1 }` — fixe la lecture par `headers()?.position()`
   (F12) ; garde à la limite : dernière ligne ignorée à exactement `date` champs (trop courte) → garde muette, à
   `date + 1` champs datée → refus (mutation M13) ;
6-bis. **angle mort d'AC5** fixé : `header_row_count` trop grand d'une unité qui avale une transaction à **date vide**
   → pas d'erreur, une transaction de moins (le test documente, il ne promet pas — R-12) ;
7. AC3 en ligne : débit négatif sous `Positive`, débit positif sous `Negative`, crédit négatif sous les deux, zéros
   acceptés (`0.00`, `-0.00`) ; un crédit écrit **`+500.00`** est lu `500.00` et accepté (phrase de #620, R2-13) ;
7-bis. **Compteur propre au signe** (R2-1 = F2-1) : (a) 101 lignes `INVALID_AMOUNT` **puis** une ligne de signe
   contraire → `HardFailure(AmountSignMismatch { total_errors: 1, truncated: false, .. })`, `errors.len() == 1` ;
   (b) 150 débits négatifs sous `Positive` → `total_errors: 150`, `truncated: true`, `errors.len() == 100` ;
7-ter. **`whole_debit_column`** (C-26-1-23) : fixture sous `Positive` → `true` ; un CSV en ligne aux débits justes
   et à **un** crédit négatif (extourne) → `HardFailure(AmountSignMismatch { whole_debit_column: false, total_errors:
   1, .. })` — bloquant quand même ; un CSV où une partie seulement des débits est de mauvais signe → `false` ;
   **cas limites fixés** (R3-2 = F3-2, R3-3) : aucun débit (crédits seulement, un crédit négatif) → `false` ; débits
   tous à `0.00` + un crédit négatif → `false` ; **un seul** débit, en défaut → `false` (`d = 1`, message « lignes
   listées », F3-4) ; deux débits tous deux en défaut → `true` ; profil `Negative` dont tous les débits (≥ 2) sont
   positifs → `true` ; un débit de mauvais signe sur une ligne à **date illisible** + deux débits justes → pas de refus
   de signe (`INVALID_DATE`, la ligne n'entre pas au dénominateur) ; un débit **illisible** (`INVALID_AMOUNT`) + deux
   débits en défaut → `true` (le débit illisible est hors dénominateur) ;
7-quater. **Préséance sur une même ligne** (F2-11) : date illisible **et** débit de mauvais signe sur la même ligne →
   `INVALID_DATE` pour cette ligne, pas de refus de signe (si c'est la seule) ; sur **deux** lignes distinctes → refus
   de signe ;
8. AC1 : `validate` refuse `Negative` en colonne unique, accepte `Positive` ; `DebitSign` serde : `"positive"` /
   `"negative"` acceptés, `"Negative"` et `"autre"` refusés ;
9. AC3-bis (`error.rs`) : pour **chaque** variant de `CsvLineErrorCode`, `as_str()` ==
   `serde_json::to_value(code)` ; la liste des variants est produite par la **source** — une fonction
   `CsvLineErrorCode::all() -> &'static [Self]` dont la complétude est gardée par un `match` exhaustif sans bras par
   défaut (un variant neuf oublié de `all()` ne compile pas), et non énumérée à la main dans le test (R3-12) ;
10. **Forme de la fixture** (R3-11) : `ubs_fixture_has_the_documented_shape` — trois premiers octets `EF BB BF`,
   dernier octet ≠ `\n`, aucun `\r`, 21 éléments au sens de `split('\n')`, le 9ᵉ vide.

**AC15 — Test d'intégration de route.** Dans `crates/kesh-api/tests/bank_imports_e2e.rs` (ou `bank_profiles_e2e.rs`,
au plus près de l'existant) : création du profil UBS **par `POST /api/v1/bank-profiles`** (pas par le dépôt — c'est le
chemin DTO → base → `db_profile_to_csv_profile` qu'on veut traverser), réponse `debitSign == "negative"`,
`headerRowCount == 9` ; import de la fixture par le multipart existant (`build_csv_multipart`) ; en base,
`SELECT COUNT(*), SUM(amount), SUM(amount < 0) FROM bank_transactions WHERE …` → `11`, `-236.25`, `9`. Plus (statuts
réels, AC10) : une requête sans `debitSign` → **422** (rejet `Json` d'axum), rien en base ; `negative` en colonne
unique → **422 `BANK_CSV_PROFILE_INVALID`** ; `headerRowCount: 21` → **422 `BANK_CSV_PROFILE_INVALID`** ; le profil UBS
avec `headerRowCount: 10` → **422 `BANK_CSV_HEADER_SKIPS_DATA`**, `details.line == 11`, en preview **et** en import ;
le profil UBS réglé `positive` → **422 `BANK_CSV_AMOUNT_SIGN_MISMATCH`**, `details.totalErrors == 9`,
`details.wholeDebitColumn == true`, message `…-refused-profile`, `details.lines[0].code == "AMOUNT_SIGN_MISMATCH"`
(graphie exacte sur le fil, AC3-bis), en preview **et** en import avec `confirmPartialImport=true` — **aucune**
transaction écrite (`COUNT(*) = 0`) ; un CSV à extourne isolée → même code, `wholeDebitColumn == false`, message
`…-refused-lines`, rien d'écrit ; et un CSV à `INVALID_DATE`
existant rend `warnings.invalidLines.lines[0].code == "INVALID_DATE"` (la graphie corrigée des codes existants). `valid_profile_payload` (`bank_profiles_e2e.rs` ≈ l. 158) et le
`NewBankProfile` de `create_test_bank_profile_csv` (`bank_imports_e2e.rs` ≈ l. 691) gagnent le champ ; le test de
`PUT` (≈ l. 527) vérifie que `debitSign` se modifie et se relit.

**AC16 — E2E Playwright** (`frontend/tests/e2e/bank-csv-import.spec.ts`, scénario neuf, patron de
`uploads CSV with matching profile end-to-end`) : création du profil **par l'écran** — nom « UBS synthétique —
moins », motif `^ubs_synthetique\.csv$`, date `%Y-%m-%d`, `;`, `.`, lignes à ignorer **9**, case « débit/crédit séparées » cochée,
colonnes date 2 / débit 5 / crédit 6 / référence 9 / détails 10, bouton `debit-sign-negative` — puis import de la
fixture (chemin relatif vers `crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv`, **sans copie**), aperçu
`preview-tx-count` = `11`, confirmation ; enfin, par l'API (`authedApiContext`), les transactions de cet import :
11, dont 9 négatives, somme −236.25 (trouver la route de lecture existante, ne pas en créer).

**Second scénario, indépendant du premier** (R-5/F8, C-26-1-22) : profil « UBS synthétique — sans signe », réglage
« sans signe », **motif distinct** `^ubs_sans_signe\.csv$`, et la **même fixture** téléversée sous ce nom
(`setInputFiles({ name: 'ubs_sans_signe.csv', mimeType: 'text/csv', buffer })`, contenu lu depuis le chemin de la
fixture — toujours sans copie versionnée). Ainsi aucun des deux fichiers n'est reconnu par les deux profils, l'ordre et
le rejeu d'un scénario n'influent pas sur l'autre. Attendu : le panneau d'erreur (`bank-import-error`) porte
`data-error-code="BANK_CSV_AMOUNT_SIGN_MISMATCH"` et le **message traduit** — assertion **insensible à la casse** sur
« sens des débits » (`toContainText(/sens des débits/i)`, F2-9 : le message commence par « Réglez ») ; **au moins une
ligne listée** (`bank-import-error-line`) avec son numéro et son message par ligne traduit (R2-2 = F2-4, arbitrage
(2)) ; **aucune** case « importer les lignes valides quand même » n'est offerte ; **aucun import de plus** : le nombre
de lignes `bank-import-list-row` (ou le compte rendu par l'API) est relevé **avant** le téléversement et comparé
**après** — la spec partage sa base (`seedTestState` en `beforeAll`, une fois par fichier ; `workers: 1`) avec des
scénarios qui importent, la liste n'est donc pas vide (F2-9).
Un troisième scénario, **neuf** (le seul existant qui exerce `warning-invalid-lines`, `bank-import-confirms.spec.ts`
l. ≈ 175-181, est désactivé par `test.skip`, F3-8) : profil créé par l'écran, CSV en ligne à `INVALID_DATE`, le tableau
des lignes invalides affiche le message traduit à côté du code (AC3-bis). Le scénario désactivé reste tel quel.

⚠️ **« Aucun import de plus »** se mesure par l'API (`GET /api/v1/bank-imports?bankAccountId=…`, champ `total` de la
réponse paginée), **pas** par `bank-import-list-row` : la liste de l'écran est plafonnée à `limit = 20`
(`listBankImports`, `bank-import.api.ts` l. ≈ 75-79), un import ajouté au-delà ne changerait pas le compte (F3-8).

**Accessibilité** (R-6, R2-11, F2-10) : aucune spec existante ne scanne le **formulaire** de profil. Deux scans existent
dans le domaine : `/bank-import` (`bank-import.spec.ts` l. ≈ 225-231) et la **liste** `/bank-import/profiles`
(`bank-csv-import.spec.ts` l. ≈ 145-152, test « accessibility — profile pages axe scan zero violations ») ; le scan
neuf se place **à côté de ce dernier**, dans le même fichier. Un test axe **neuf** sur `/bank-import/profiles/new`,
**mode séparé coché** (groupe radio visible), zéro violation. **Conduite si le scan révèle une violation
préexistante** du formulaire (libellés de colonnes en dur, `<label>` contenant un `<span>`, F3-8) : la corriger si elle
tient en quelques lignes du formulaire ; sinon ouvrir une issue (demandée à l'orchestrateur) et l'**exclure
nommément** du scan (`disableRules([...])` avec un commentaire qui cite l'issue) — jamais une exclusion silencieuse ni un
scan affaibli sans trace. Les specs existantes qui remplissent le formulaire (mode
colonne unique) passent sans retouche.

**AC17 — Contrôle local sur le vrai fichier, hors dépôt (C-26-1-1).** Un test versionné
`crates/kesh-import/tests/ubs_fichier_reel.rs`, **`#[ignore]`**, lit le chemin dans `KESH_UBS_REAL_CSV` (absent → le
test **échoue** avec un message clair, il ne passe pas à vide quand on le lance explicitement) ; il parse le fichier
avec le profil UBS (C-26-1-13), puis **lit lui-même le préambule** (`Solde initial:`, `Solde final:`, `Nombre de
transactions…:`) et vérifie : `AllValid` ; nombre = « Nombre de transactions » ; somme = solde final − solde initial,
au centime ; aucune erreur de ligne ; **répartition** : chaque transaction issue de la colonne Débit est `< 0` et
chaque transaction issue de la colonne Crédit `> 0` (assertion structurelle, aucun nombre en dur — R2-13). Il
n'imprime **que** « conforme » ou le nom de l'assertion qui a échoué — jamais un nombre, une somme, une ligne, un
libellé, l'IBAN, ni un solde. **Aucun chiffre du fichier réel n'est écrit dans le test.** ⛔ **Même en
échec** (R-9) : pas d'`assert_eq!` sur une valeur lue (il imprimerait gauche et droite, donc un solde) —
`assert!(somme == attendu, "somme ≠ solde final − solde initial")`, `assert!(n == annoncé, "nombre ≠ nombre annoncé
par le préambule")`, sans interpoler de montant ni de solde ; les erreurs du parseur s'affichent par leur **code** et
leur **numéro de ligne** seulement, jamais par `value` ni `Debug` complet. Ce test est **la** preuve du critère
posé par l'epic et #621 (nombre annoncé, somme = solde final − solde initial) : la fixture synthétique est **confirmée par
l'orchestrateur** (dépôt public, R-10, C-26-1-1) et ce test, hors CI, est **exécuté au gate** et rapporté « conforme / non conforme ». Commande ; au
Dev Agent Record, **seulement « conforme » ou « non conforme »** (arbitrage (0) de l'orchestrateur, P2 — aucun nombre,
aucune somme, même agrégés) :

```sh
KESH_UBS_REAL_CSV=/home/gcorbaz/kesh-test/transactions.csv \
  cargo test -p kesh-import --test ubs_fichier_reel -- --ignored --nocapture
```

### La documentation

**AC18 — Manuel utilisateur FR et CHANGELOG.**

- `docs/manual/fr/user-manual.tex`, § *Import CSV multi-encodage* → *Profils banque* (≈ l. 1669-1681) : la liste
  actuelle décrit un écran qui n'existe pas (« Uploader un fichier CSV exemple », « libellé, devise »,
  « Windows-1252 ») ; la **réécrire** d'après le formulaire réel : nom, motif de nom de fichier, format de date,
  séparateurs, encodage (UTF-8, ISO-8859-1 ou détection), **lignes à ignorer** (ligne de titres comprise, lignes vides
  non comptées, 0 à 20), mapping des colonnes (numérotées depuis 0), mode débit/crédit séparés et **sens des débits**.
  Ajouter une sous-section **« Le relevé CSV d'UBS »** : les valeurs exactes du profil (tableau de C-26-1-13), pourquoi
  9 et non 10, ce qu'il advient d'un débit de signe contraire (refus de tout le fichier), et ce qu'il advient d'un nombre trop
  grand (refus nommant la ligne) ; dire que **le préambule (IBAN, soldes) n'est pas lu** dans cette version et
  qu'un import CSV ne porte donc toujours pas de solde de clôture (cohérent avec la phrase de la l. ≈ 1630). Ne pas
  toucher les lignes ≈ 1638 et 1652 (CAMT.053 « toutes les banques », case « Compte de banque ») : elles sont à la 26-3
  (#616, #617) — au rebase, **union**.
- Régénérer le PDF (`make fr` dans `docs/manual/`) et le commiter ; contrôler le **PDF** aplati :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "Le relevé CSV d.UBS.\{0,200\}"`.
  Les macros `\keshVersion` & co **ne bougent pas** (gate de release, pas de story).
- Dans la même réécriture (F11, R-1, R-12, F10) : **en colonne unique, le signe du fichier est gardé tel quel** — une
  dépense doit y être négative, et Kesh ne le contrôle pas (une banque qui écrirait ses dépenses en positif dans une
  colonne unique serait importée à l'envers) ; **un fichier dont une seule ligne a un signe contraire à sa colonne
  est refusé en entier** (régler le sens des débits du profil, puis redéposer le fichier) ; le numéro de ligne annoncé pour une ligne qui
  suit une ligne vide peut être celui de la ligne vide ; la garde contre un nombre trop grand ne voit qu'une ligne
  **datée** ; **la colonne Devise n'est pas lue** : tout import CSV est en CHF ; **une ligne dont le signe est contraire
  à ce que le profil attend pour sa colonne bloque le relevé entier** — sous ses **trois** formes (F3-5) : un crédit
  écrit en négatif ; un débit écrit en négatif quand le profil attend des débits sans signe ; un débit écrit en positif
  quand le profil attend des débits négatifs. Une extourne écrite ainsi ne peut pas être importée telle quelle dans
  cette version (C-26-1-23). Dans la sous-section UBS : la colonne « No de transaction » (référence) **nourrit la
  détection des doublons** — sans elle, deux paiements par carte du même jour et du même montant seraient pris l'un
  pour l'autre (R3-8).
- **§ « Rejet partiel »** (l. ≈ 1709-1711, R2-3 = F2-3, arbitrage (4)) : la phrase « Kesh importe les valides » est
  fausse deux fois — l'import des lignes valides se fait **sur confirmation** (case « importer les lignes valides quand
  même », `confirm-partial-import`), et **pas du tout** pour un montant de signe contraire à sa colonne, qui refuse le
  fichier entier (renvoi à la sous-section UBS). La réécrire en ce sens ; garder le renvoi à la détection de doublons
  s'il reste vrai (le vérifier au code). La brochure (« gestion du rejet partiel », `marketing-brochure.tex` l. ≈ 165)
  reste vraie en général : n'y pas toucher.
- ⛔ **§ « Import avec un profil existant » : NE PAS la toucher** (R3-1 = F3-1, arbitrage (1) de la P3, C-26-1-27).
  Elle appartient à la **26-3** (AC 2.1 de sa fiche, déjà en développement), qui l'a rédigée d'après l'écran réel
  (choisir le profil **avant** de déposer le fichier, ou *Annuler*, choisir, redéposer — l'aperçu n'est pas recalculé
  quand on change de profil après le dépôt ; issue #626). La 26-1 y **renvoie** depuis la sous-section UBS et le refus de
  signe (« redéposez le fichier »), sans la réécrire.
- **§ « Détection des doublons »** (l. ≈ 1693-1707 sur `fe162504`, R3-8, arbitrage (1)) : fausse au code et touchée par
  **aucune** autre story (vérifié en lecture dans la fiche et la branche de la 26-3 le 2026-10-11 : elle ne traite que
  le rapport CAMT, `:1657-:1663`, et laisse cette section intacte). La réécrire : l'empreinte est faite de la **date**, du
  **montant**, de la **référence** (à défaut l'identifiant de bout en bout ou de transaction, absents d'un CSV) et du
  **compte bancaire** — **pas** du libellé (`dedup_key_scalar`, `kesh-core/src/bank_imports.rs` l. ≈ 337-356) ; les
  doublons ne sont pas « ignorés silencieusement » : l'aperçu les signale (`warning-duplicate-lines`) et l'utilisateur
  choisit de les **ignorer** (par défaut) ou de les **importer quand même** (`confirm-duplicate-lines-skip` /
  `-import`) ; un fichier déjà importé est signalé à part (`confirm-duplicate-file`). Pas de « rapport d'import ».
- Contrôle du **PDF** aplati sur les passages réécrits : « Le relevé CSV d'UBS », la phrase neuve de « Rejet partiel »
  et celle de « Détection des doublons » (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep
  -o "<début de phrase>.\{0,200\}"`). ⚠️ `pdftotext` rend l'apostrophe **typographique** (`’`) là où le `.tex` écrit
  `'` : écrire **`.`** à la place de toute apostrophe dans le motif (R3-14), comme `Le relevé CSV d.UBS`.
- `CHANGELOG.md` : section **`## [0.13.1] — Non publié`** (forme exacte attendue par `scripts/prepare-release.sh`
  l. 237 ; à créer si absente, à **compléter par union** si la 26-2 ou la 26-3 l'a créée entre-temps) : *Corrigé* —
  #620 (le sens des débits devient un réglage ; un fichier dont un montant a un signe contraire à sa colonne est
  refusé, sans import partiel possible) ; *Modifié* — #621 (jusqu'à 20 lignes à ignorer ; refus d'un nombre qui
  avalerait une transaction datée) — et, en *Précisé*, que les lignes vides **ne comptent pas**, **comme avant** : c'est
  le comportement existant, désormais écrit, non un changement (R3-9) ; *Modifié* — **graphie des codes d'erreur de ligne** de
  l'import CSV (`warnings.invalidLines.lines[].code`, `details.lines[].code`) : `INVALIDDATE` → `INVALID_DATE`,
  `INVALIDAMOUNT` → `INVALID_AMOUNT`, `AMBIGUOUSDEBITCREDIT` → `AMBIGUOUS_DEBIT_CREDIT`, `EMPTYMANDATORYFIELD` →
  `EMPTY_MANDATORY_FIELD`, `ROWTOOSHORT` → `ROW_TOO_SHORT` (changement visible d'un client d'API) ; *Modifié* —
  **API des profils bancaires** : `POST` et `PUT /api/v1/bank-profiles` **exigent** `debitSign` (`"positive"` ou
  `"negative"` ; sans lui, `422`), et l'import CSV gagne deux refus `422` : `BANK_CSV_AMOUNT_SIGN_MISMATCH` (non
  confirmable) et `BANK_CSV_HEADER_SKIPS_DATA` (R2-10) ; `GET /api/v1/bank-profiles` et `GET …/{id}` renvoient
  `debitSign` ; l'**export CSV des tables** (`bank_profiles.csv`) gagne la colonne `debit_sign` juste après
  `header_row_count` — un lecteur par position voit se décaler `created_at` et `updated_at` (R3-9) ; la conséquence de l'extourne (un relevé qui en porte une ne
  s'importe pas tel quel) écrite en toutes lettres ; un encadré **« À faire en passant à la 0.13.1 »** : sauvegarder avant ; la base passe à
  `kesh_version_min_required = 0.13.1` (une 0.13.0 refuse ensuite de démarrer et d'importer une sauvegarde 0.13.1) ;
  un profil en débit/crédit séparés dont la banque écrit déjà ses débits en négatif doit être **réglé** (sinon : import
  refusé, plus d'inversion muette).
- `docs/api-external.md` : ne documente ni les routes d'import CSV ni les codes de ligne (vérifié par grep le
  2026-10-10) — rien à y écrire ; le re-vérifier au moment du patch.
- Manuels DE/IT/EN : ils n'existent pas (`docs/manual/{de,it,en}/README.md` seuls) — rien à faire. Manuel
  d'administration, brochure, site, README : aucun ne décrit les profils CSV (vérifié par grep le 2026-10-10 ; le
  re-vérifier).

### Les mutations et les gates

**AC19 — Mutations** (chacune appliquée seule, le test désigné **rougit**, puis restaurée — et le fichier **touché**
après restauration, cargo gardant sinon le binaire muté) :

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | `parse_row` : la branche `Negative` rend `-parsed` (réglage ignoré) | AC14-1, AC15, AC16 |
| M2 | contrôle de signe de la colonne Crédit retiré | AC14-7 |
| M3 | contrôle de signe Débit sous `Positive` retiré | AC14-3 |
| M4 | borne de `validate` remise à 5 | AC4, AC15 (`headerRowCount: 9`) |
| M5 | garde AC5 retirée | AC14-5, AC15 (422) |
| M6 | `db_profile_to_csv_profile` force `DebitSign::Positive` | AC15 (somme), AC16 |
| M7 | `update` du dépôt n'écrit plus `debit_sign` | AC8 (PUT) / AC15 (PUT) |
| M8 | le formulaire envoie toujours `'positive'` | Vitest AC12-(a), AC16 |
| M9 | `validate` accepte `Negative` en colonne unique | AC14-8, AC15 (422 `BANK_CSV_PROFILE_INVALID`) |
| M10 | `debit_sign_enum` replie sur `Positive` au lieu d'`Invariant` | test AC8 sur valeur hors domaine (à écrire : `UPDATE` brut hors CHECK impossible → tester la fonction sur une entité construite à la main) |
| M11 | `serialize_bank_profiles_csv` n'écrit pas la colonne `debit_sign` (en-tête et valeur) | test de cohérence en-têtes/schéma (`csv_tables.rs` ≈ l. 2229) + un test de valeur (`negative` exporté) |
| M12 | `"debit_sign"` retiré de `fields_changed` | test d'audit d'AC8 |
| M13 | garde AC5 : `>= date + 1` champs remplacé par `> date + 1` (ou retiré) | AC14-6 (limite de longueur) |
| M14 | `parse_csv_collect` rend `PartialFailure` au lieu de `HardFailure` sur `AmountSignMismatch` (erreur redevenue confirmable) | AC14-3, AC15 (`confirmPartialImport=true` → 0 écrit), AC16 (2e scénario) |
| M15 | `csv_line_error_to_payload` revient à `format!("{:?}").to_uppercase()` | AC15 (`"INVALID_DATE"`, `"AMOUNT_SIGN_MISMATCH"`) |
| M16 | `csvLineErrorLabel` rend le code brut au lieu de la clé (ou le panneau ne l'appelle plus) | Vitest `csv-line-error-label.test.ts`, AC16 (message par ligne) |
| M17 | décision du refus de signe prise sur le vecteur **général** écrêté au lieu de `sign_total` | AC14-7-bis (a) |
| M18 | contrôle « débit **positif** sous `Negative` » retiré | AC14-7, AC14-7-ter |
| M19 | comparaison au zéro rendue non stricte (`<= ZERO`) ou par `is_sign_negative()` | AC14-7 (`-0.00`, `0.00`) |
| M20 | ligne d'en-têtes lue par `reader.position()` au lieu de `headers()?.position()` | AC14-6 (`header_row_count = 1` → ligne 1) |
| M21 | `whole_debit_column` toujours `true` (ou toujours `false`) | AC14-7-ter, AC15 (deux messages) |
| M22 | le panneau d'erreur ne liste plus les lignes du refus de signe | AC16 (`bank-import-error-line`) |
| M23 | `whole_debit_column` sans la condition `d ≥ 2` (ou sans « au moins un débit non nul ») | AC14-7-ter (un seul débit ; aucun débit) |
| M24 | contrôle de signe placé **avant** la lecture de la date dans `parse_row` | AC14-7-quater |
| M25 | bras explicites `AmountSignMismatch` / `HeaderSkipsData` de `map_csv_error` retirés (repli `other =>` → 500) | AC15 (deux 422) |
| M26 | migration sans le relâchement de `chk_bank_profiles_header_row_count` (borne restée à 5) | AC8 (profil à 9 relu) |
| M27 | `#[serde(default)]` ajouté à `debit_sign` dans la requête | AC15 (sans `debitSign` → 422) |
| M28 | fixture reformatée (LF final ajouté, ou BOM retiré) | AC14-10 |

Une mutation qui ne fait rien rougir est un **trou de test** : écrire le test manquant, pas reclasser.

**AC20 — Gates** (CLAUDE.md, § *Test Locally First*, sous `scripts/mem-guard.sh`, cible cargo **du worktree**, bases
`kesh_<clé>` / `kesh_e2e_<clé>` **propres au worktree**, consignes 6 et 8) :

- **`kesh-db` touché → gate COMPLET même en cours de boucle** (exception `kesh-db` de la § *Pendant une boucle de
  revue*) : `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest workspace) ;
- frontend : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build` ;
- **E2E complet au DERNIER commit de code** (décision D7), comparé fichier par fichier aux échecs attendus
  (`docs/testing.md` § *Les échecs attendus*) ; le backend E2E démarre contre une base **persistante** migrée par le
  worktree (c'est le seul gate qui verrait une violation de P8) ;
- `cargo publish --dry-run -p kesh-import` (job CI ; le crate gagne un variant public et un champ public, cf. Dev
  Notes) ;
- le contrôle AC17 sur le vrai fichier (résultat au Dev Agent Record : « conforme » ou « non conforme », rien d'autre) ;
- **ne déclarer que ce qui a tourné**, avec ses chiffres et son périmètre ; recompter tout décompte écrit.

## Tâches

- [x] **T1 — `kesh-import`** (AC1-5, AC3-bis, AC14 tests 1-9 dont 6-bis, 7-bis, 7-ter, 7-quater) : compteur et vecteur
  propres au signe, `whole_debit_column`, `DebitSign`, champ, `validate`, `parse_row`, garde
  AC5, variants `CsvLineErrorCode::AmountSignMismatch`, `CsvError::AmountSignMismatch` / `HeaderSkipsData`,
  `CsvLineErrorCode::as_str()`, mise à jour des **tous** les littéraux `CsvProfile { … }` des tests
  (`profile.rs` ≈ l. 244, `parser.rs` ≈ l. 575 et 1040), fixture + README des fixtures.
- [x] **T2 — Migration et base** (AC6, AC7, AC9) : migration, squash régénéré, ligne de `migrations.sha384`,
  P5/P6 (`total` 78 → 79 **et** `total - 44` → `total - 45`)/P7, assertions de version, test de migration dans
  `migrations_upgrade_path.rs` ; bump des dix `Cargo.toml` + `Cargo.lock` dans le **même commit** que la migration (P2-bis).
- [x] **T3 — Entité, dépôt** (AC8) : dont le helper `make_new_profile` et le test `debit_sign_enum` → `Invariant`.
- [x] **T4 — API** (AC3, AC3-bis, AC5 côté API, AC10, AC11, AC15) : DTO, réponse, conversion,
  `AppError::BankCsvHeaderSkipsData` et `AppError::BankCsvAmountSignMismatch` (bras explicites de `map_csv_error`),
  `csv_line_error_to_payload` par `as_str()`, export CSV, test `is_required`, tests d'intégration.
- [x] **T5 — Frontend et i18n** (AC3, AC3-bis, AC12, AC13) : types, formulaire, module `csv-line-error-label.ts` et sa
  déclaration dans `i18n-keys.test.ts` (`MOTIFS_DYNAMIQUES`, `CARDINALITES`, relais, recompte d'`ATTENDU`), panneau
  d'erreur qui liste les lignes du refus de signe (sans case de confirmation), Vitest, quatre locales.
- [x] **T6 — E2E** (AC16).
- [x] **T7 — Documentation** (AC18) : manuel (Profils banque, sous-section UBS, Rejet partiel, Détection des doublons ;
  **pas** « Import avec un profil existant », à la 26-3) + PDF contrôlé aplati, CHANGELOG (graphie des codes, `debitSign` obligatoire, deux 422, extourne) ; grep de la valeur « 5 »
  / « 0-5 », et des graphies `INVALIDDATE` & co.
- [ ] **T8 — Contrôle sur le vrai fichier** (AC17), mutations (AC19), gates (AC20), Dev Agent Record.

Ordre conseillé : T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8. Un commit par étape BMAD (CLAUDE.md), messages en
`refs #620 #621`.

## Dev Notes

### État actuel des fichiers modifiés (lus le 2026-10-10 sur `fe162504`)

- **`crates/kesh-import/src/csv/profile.rs`** (421 l.) — `ColumnMapping` (`deny_unknown_fields`, XOR
  `amount`/`debit_credit_split`), `CsvProfile` (8 champs, `header_row_count: u8`), `validate()` (borne 5 l. ≈ 202).
  À préserver : toutes les règles existantes, l'ordre des contrôles, les messages (des tests les lisent).
- **`crates/kesh-import/src/csv/parser.rs`** (1275 l., 28 tests) — pipeline : `validate` → encodage/BOM → `ReaderBuilder`
  (`has_headers(header_row_count > 0)`, `flexible(true)`) → saut de `header_row_count − 1` enregistrements → premier
  enregistrement (contrôle d'indices) → `parse_row` par ligne → `ImportedStatement`. `parse_row` l. ≈ 369-500 ; la
  branche à changer l. ≈ 425-436. À préserver : `MAX_CSV_LINE_ERRORS`, la sentinelle `NaiveDate::MIN`, les numéros de
  ligne absolus (`line_numbers_are_file_absolute_with_multi_header`), `parse_amount` (inchangé : il lit déjà
  `-500.00` ; **ne pas** y toucher pour le signe).
- **`crates/kesh-import/src/error.rs`** — `CsvError` (dont `ProfileMisconfigured`, rendu en **500 au message masqué**
  par l'API — d'où une variante dédiée pour AC5) ; `CsvLineErrorCode` (5 variants, `SCREAMING_SNAKE_CASE`, `i18n_key()`).
- **`crates/kesh-db/migrations/20260505000001_bank_profiles.sql`** — **ne pas modifier** (P8) ; la contrainte
  `chk_bank_profiles_header_row_count` s'y nomme ainsi.
- **`crates/kesh-db/src/entities/bank_profile.rs`**, **`repositories/bank_profiles.rs`** — `COLUMNS` explicite (une
  0.13.0 lit donc une table à colonne en plus sans erreur — fondement de C-26-1-3) ; audit `fields_changed` dans
  `update` (l. ≈ 260-290).
- **`crates/kesh-api/src/routes/bank_profiles.rs`** (288 l.) — DTO `deny_unknown_fields`, `validate_request`,
  `try_from_db`. **`routes/bank_imports.rs`** — `db_profile_to_csv_profile` (l. ≈ 1574), conversion `CsvError →
  AppError` (l. ≈ 630-654).
- **`crates/kesh-api/src/exports/csv_tables.rs`** — `serialize_bank_profiles_csv` (l. ≈ 1036) ; test de cohérence
  en-têtes/schéma (l. ≈ 2229, avec le renommage `column_mapping → column_mapping_json`).
- **`frontend/src/lib/features/bank-import/BankProfileForm.svelte`** — `max="5"`, repli `'Nb lignes header (0-5)'`,
  case `use-debit-credit-split`. Les libellés de colonnes (« date », « débit »…) sont **en dur** : hors périmètre, ne
  pas les traduire ici (ne pas élargir).

### Ce qui reste hors périmètre

**L'import d'un relevé qui porte une extourne** (C-26-1-23) : bloqué en entier dans cette version, par choix de
sécurité ; une issue suivra si le cas se présente.

**La devise forcée à CHF** (F10) : `parse_row` écrit `currency: "CHF"` pour toute ligne CSV, la colonne `Devise`
d'UBS (col. 4) et la clé `Évaluation en:` ne sont pas lues — un compte en devise étrangère s'importerait en CHF sans un
mot. Défaut **préexistant**, de la même famille que #620 mais hors de #620/#621 : l'orchestrateur ouvre une issue ; la
story se borne à l'écrire au manuel (AC18). Lecture du préambule (IBAN, soldes, période — #621 reste ouverte) ; profil UBS prêt à l'emploi (C-26-1-10) ; MT940
(#618) ; détection de la ligne de titres (décision 2) ; réglage du sens en colonne unique (C-26-1-2) ; colonne
`counterparty` dans le formulaire ; traduction des libellés de colonnes en dur.

### Points de vigilance

- **Clés i18n des codes de ligne** (AC3-bis, F2-2) : l'écran passe par la table littérale
  `CSV_LINE_ERROR_LABELS`, jamais par `messageI18nKey` — un `i18nMsg(` à clé non littérale est un site **non résolu**
  que `i18n-keys.test.ts` relève (l. ≈ 797) et interdit de recompter (l. ≈ 942-950). Les messages arrivent au frontend
  en bloc (`GET /api/v1/i18n/messages`, `all_messages`) : aucune clé à « livrer » en plus. `lint-i18n-ownership` ne
  relève que les littéraux.

- **`kesh-import` n'est pas publié** sur crates.io (constaté le 2026-10-10 par la P2 : l'API de crates.io répond
  « crate `kesh-import` does not exist ») : la rupture semver ci-dessous est **sans objet** aujourd'hui (R2-12) ; le
  `cargo publish --dry-run` de la CI reste à passer. Pour mémoire : ajouter un champ à la struct
  publique `CsvProfile` et des variants aux enums publics est une **rupture d'API** au sens semver, sous un bump
  0.13.0 → 0.13.1 — sans conséquence tant que le crate n'est pas publié. Ne **pas** ajouter `#[non_exhaustive]` de son propre chef
  (il casserait les littéraux de `kesh-api`).
- **Un montant `-0.00`** : `Decimal` le lit comme zéro négatif ; `is_sign_negative()` peut être vrai sur un zéro —
  tester `< Decimal::ZERO` (strict), pas le bit de signe.
- **`+500.00`** en colonne Crédit : vérifier que `Decimal::from_str_exact` l'accepte (sinon `INVALID_AMOUNT`, comme
  aujourd'hui — ne pas élargir).
- **La garde AC5 et `has_headers(true)`** : pour `header_row_count == 1`, la ligne ignorée n'est pas renvoyée par
  `records()` — elle est dans `reader.headers()`, à lire **avant** d'itérer (l'emprunt de `records()` l'interdirait
  après). Le numéro de la ligne d'en-têtes : l'enregistrement rendu par `reader.headers()` porte sa propre position
  (`csv` 1.4.0 l. 1323-1326) — `reader.headers()?.position().map(|p| p.line())` ; **pas** `reader.position()` après
  lecture, qui vaut déjà la ligne suivante (F12). Testé par AC14-6 (`header_row_count = 1` sur un CSV sans titres).
- **Ne pas confondre les deux comptages** dans les textes : « lignes à ignorer » (non vides) ≠ « numéro de ligne »
  (ligne où commence la lecture — physique, sauf après une ligne vide, AC4). Le manuel donne l'exemple UBS : 9 lignes
  ignorées, première transaction à la **ligne 11** de l'éditeur (aucune ligne vide ne la précède : numéro exact).
- **Base de dev partagée** : la migration bumpe `min_required` à 0.13.1. Une base migrée par ce worktree **refuse**
  ensuite un binaire 0.13.0 (les autres worktrees de l'Epic 26, tant qu'ils ne sont pas rebasés). D'où l'obligation
  des bases **propres au worktree** (consigne 6) — ne jamais migrer `kesh` ni `kesh_e2e` partagés depuis cette branche.
- **Conflits attendus au rebase** (R3-15, F3-7) — avec la 26-2 : migration, `docs/migrations-idempotence-audit.md`,
  `migrations.sha384`, compteurs de `migrations_upgrade_path.rs`, squash, `EXEMPT_MIGRATIONS` si elle en ajoute,
  CHANGELOG, `BankImportUpload.svelte` (libellé des options de compte), `i18n-keys.test.ts`, les quatre `messages.ftl` ;
  avec la 26-3 : manuel FR (même section *Import bancaire* : la 26-3 tient « Import dans Kesh » et « Import avec un
  profil existant », la 26-1 « Profils banque », la sous-section UBS, « Rejet partiel » et « Détection des doublons »),
  CHANGELOG, `BankImportUpload.svelte` (`aria-label` de la zone de dépôt), `i18n-keys.test.ts`, les quatre
  `messages.ftl`, `textes_coherents.rs`. **Union** partout ; `ATTENDU.sitesTotal` (et tout compteur d'`i18n-keys.test.ts`)
  se **recompte** au rebase par la mesure du test, chacun avec sa ligne d'historique — **ne jamais additionner** les
  nombres des branches ; compteurs de migrations recomptés ; PDF régénéré après union.

### Architecture et conventions à suivre

- Rust : `///` sur tout public neuf, erreurs typées (`thiserror`), aucun `unwrap()` hors tests, aucun `unreachable!()`
  sur un `match` d'enum (patron CLAUDE.md : `tracing::error!` + `AppError::Internal`). Multi-tenant : aucune requête
  neuve sans `company_id`.
- Enum stocké en base : `VARCHAR` + `CHECK IN (…)` (patron `lettering_origin`, migration `20261009000001`), pas
  d'`ENUM` MariaDB.
- Frontend : Svelte 5 runes (`$state`, `$props`), `i18nMsg(clé, repli)`, `data-testid` sur tout contrôle testé.
- i18n : les quatre locales dans le même commit ; propriété des préfixes vérifiée par `lint-i18n-ownership`.

### Leçons des stories précédentes à appliquer ici

- **Un test qui passe à vide ne prouve rien** (16-1a, mémoire « tests qui prouvent moins ») : d'où l'assertion de
  montage d'AC9, la somme comparée au préambule lu par le test (AC14-1), et l'échec explicite d'AC17 sans variable.
- **Greper la valeur, pas la formulation** : « 5 », « 0-5 », « 0.13.0 », `header_row_count` — après chaque patch.
- **Les revues lisent le manuel** (CLAUDE.md, A6 de l'Epic 24) : le prompt de chaque passe nomme
  `docs/manual/fr/user-manual.tex` **et son PDF**.
- **Mutation restaurée, binaire périmé** : toucher le fichier après restauration.
- **Ne jamais écrire de donnée réelle** : un rapport de passe qui citerait une ligne du fichier de Guy est une fuite —
  les prompts de revue le rappellent.

### Project Structure Notes

Aucun répertoire neuf hors `crates/kesh-import/tests/fixtures/csv/`. Le test `ubs_fichier_reel.rs` vit dans
`crates/kesh-import/tests/` (test d'intégration du crate, binaire séparé, ignoré par défaut). Aucun conflit de
structure relevé.

### References

- Plan de l'epic : `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md` § 26-1 et « Décisions du Project Lead ».
- Issues : #620 (description + décision du PL), #621 (description + deux décisions du PL).
- Registre : `_bmad-output/implementation-artifacts/epic-26-choix-autonomes.md`, C-26-1-1 à C-26-1-27.
- CLAUDE.md : § *Migration breaking policy* (P1-P8), § *Test Locally First*, § *Review Iteration Rule*, § *Règle de
  commit et push* (CHANGELOG, manuels).
- Précédent de bump avec version Cargo dans la story : `0724904c` (Story 15-1a-i), migration
  `20261009000001_journal_entry_lines_lettering.sql`, entrée `EXEMPT_MIGRATIONS` correspondante.
- csv-core 0.1.13 : `src/reader.rs` l. 94 (lignes vides ignorées), l. 665/724 (comptage des lignes) ; csv 1.4.0 :
  `src/reader.rs` l. 1626 (position posée avant la lecture), l. 1323-1326 (`headers()` et sa position).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`bmad-dev-story`, autonomie), 2026-10-11.

### Debug Log References

- Gate complet backend `scripts/test-fast.sh` sur `kesh_261` remise à zéro : premier passage rouge sur `cargo fmt`
  (trois fichiers neufs), corrigé ; second passage **3383/3383**, 6 ignorés, fmt + clippy `-D warnings` verts.
- Vitest complet : premier passage rouge sur `i18n-libelle-en-dur.test.ts` (déclaration candidate neuve
  `csvLineErrorLabel`, 61 → 62, conforme 47 → 48 — recompte avec sa ligne d'historique) ; second passage
  **1295/1295**, 125 fichiers.
- E2E ciblé `bank-csv-import.spec.ts` (backend :3031, base `kesh_e2e_261` migrée par le worktree) : **12/12**, dont les
  trois scénarios neufs et le scan axe du formulaire (zéro violation, aucune exclusion).

### Completion Notes List

- Fiche créée le 2026-10-10 (Opus 5.5, `bmad-create-story`, autonomie) : analyse des sources, structure du fichier UBS
  relevée **en formes** (aucune donnée copiée), choix C-26-1-1 à 13 consignés au registre.
- Validation P1 remédiée le 2026-10-10 (Opus 5.5) : AC3-bis ajouté (21 AC), choix C-26-1-14 à 22 ; bilan au Change Log.
- Validation P2 remédiée le 2026-10-10 (Opus 5.5) : chiffres réels retirés ; compteur de signe propre, deux messages,
  liste des lignes à l'écran, table littérale des libellés, manuel (Rejet partiel, Import avec un profil existant —
  ce dernier retiré en P3, C-26-1-27) ;
  C-26-1-23, 24 ; bilan au Change Log.
- Développement le 2026-10-11 (Opus 5.5) : T1 à T7 livrés ; choix C-26-1-28 à 35 au registre. Contrôle AC17 du
  relevé réel : **conforme**. Contrôle du diff contre le relevé réel, sans affichage : aucun nombre non rond ; les
  seules coïncidences sont onze dates calendaires, toutes déjà présentes dans le dépôt avant la story (fixture de la
  fiche validée).
- Validation P3 remédiée le 2026-10-11 (Opus 5.5) : `whole_debit_column` défini (d ≥ 2 et e = d, `RowParse`), manuel
  recentré (Détection des doublons ajoutée, Import avec un profil existant laissé à la 26-3), DRY `common`, mutations
  M23-M28 ; C-26-1-25 à 27 ; bilan au Change Log.

### File List

- `crates/kesh-import/src/csv/profile.rs`, `csv/parser.rs`, `csv/mod.rs`, `error.rs`, `lib.rs`
- `crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv` (neuf), `tests/fixtures/README.md`,
  `tests/ubs_fichier_reel.rs` (neuf, `#[ignore]`)
- `crates/kesh-db/migrations/20261011000001_bank_profiles_debit_sign.sql` (neuf), `migrations.sha384`,
  `test-schema/0001_schema_squash.sql`, `src/entities/bank_profile.rs`, `src/repositories/bank_profiles.rs`,
  `src/post_restore.rs`, `src/backup.rs`, `tests/migrations_upgrade_path.rs`, `tests/migrations_fresh_install.rs`
- `crates/kesh-api/src/errors.rs`, `src/routes/bank_profiles.rs`, `src/routes/bank_imports.rs`,
  `src/exports/csv_tables.rs`, `tests/bank_profiles_e2e.rs`, `tests/bank_imports_e2e.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `crates/*/Cargo.toml` (dix crates → 0.13.1), `Cargo.lock`
- `frontend/src/lib/features/bank-import/BankProfileForm.svelte`, `BankProfileForm.test.ts` (neuf),
  `BankImportUpload.svelte`, `bank-profile.types.ts`, `csv-line-error-label.ts` (neuf),
  `csv-line-error-label.test.ts` (neuf)
- `frontend/src/lib/shared/i18n-keys.test.ts`, `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts`
- `frontend/tests/e2e/bank-csv-import.spec.ts`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`, `docs/migrations-idempotence-audit.md`,
  `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/26-1-import-csv-ubs-natif.md`, `epic-26-choix-autonomes.md`,
  `sprint-status.yaml`

## Change Log

| Date | Auteur | Changement |
|---|---|---|
| 2026-10-10 | Opus 5.5 (create-story) | Création de la fiche ; 20 AC ; statut ready-for-dev ; validation à suivre. |
| 2026-10-10 | Opus 5.5 (remédiation validation P1) | Validation P1 (Sonnet, lentilles R et F, contexte frais) : **R 0C/0H/4M/8L** (R-1 à R-12 ; le bilan du rapport disait 3M/7L, ses titres comptent 4 MEDIUM et 8 LOW — corrigé en P2, R2-7), **F 0C/1H/5M/8L** (F1 à F14). Après dédoublonnage : **1 HIGH, 7 MEDIUM, 12 LOW distincts** (MEDIUM : F2, F3=R-1, F4, F5, F6=R-4, R-3, R-5=F8). Bilan finding par finding ci-dessous. Nouvel AC3-bis (21 AC) ; choix C-26-1-14 à C-26-1-22. |
| 2026-10-10 | Opus 5.5 (remédiation validation P2) | Validation P2 (Opus R/F) : 5 MEDIUM, 14 LOW distincts, plus l'arbitrage (0) sur les données réelles (commit à part). Bilan finding par finding ci-dessous. Choix C-26-1-23, 24. |
| 2026-10-11 | Opus 5.5 (remédiation validation P3) | Validation P3 (Sonnet R/F) : 2 MEDIUM, 18 LOW distincts. Bilan finding par finding ci-dessous. Choix C-26-1-25 à 27. |
| 2026-10-11 | Opus 5.5 (clôture de la validation) | Validation P4 **ciblée** (Haiku, une lentille, sur le commit de remédiation P3) : 1 MEDIUM, 2 LOW, appliqués. **Validation close.** Statut → in-progress. Bilan ci-dessous. |
| 2026-10-11 | Opus 5.5 (dev-story) | Implémentation T1-T7 (migration `20261011000001`, bump 0.13.1, `DebitSign`, refus de signe non confirmable, garde de surcomptage, graphie des codes, formulaire, panneau, manuel, CHANGELOG). Gate complet 3383/3383, Vitest 1295/1295, E2E ciblé 12/12 ; mutations et E2E complet à suivre (T8). |

### Validation P1 — bilan finding par finding

| Finding | Sév. | Traitement |
|---|---|---|
| F1 = R-2 | HIGH / M | **Corrigé** — AC3-bis : `CsvLineErrorCode::as_str()` (graphie `SCREAMING_SNAKE_CASE`, test d'égalité avec serde pour chaque variant), panneaux qui affichent `messageI18nKey` ; changement de graphie des cinq codes existants consigné au CHANGELOG (`### Modifié`) ; `docs/api-external.md` vérifié sans objet. C-26-1-14 (arbitrage orchestrateur). |
| F2 | M | **Corrigé** — AC10/AC15 réécrits sur les statuts réels : `422 BANK_CSV_PROFILE_INVALID` (validate), `422` axum texte brut (désérialisation) ; pas d'extracteur à rejet géré (local à `bank_accounts.rs`, élargirait la story). C-26-1-21. |
| F3 = R-1 | M / M | **Corrigé** — AC4 pose la sémantique réelle (ligne où commence la lecture ; un enregistrement précédé de lignes vides est rapporté à la première), AC14-4 attend **9** (+ cas sans ligne vide), manuel et doc-comment le disent ; C-26-1-7 amendé par C-26-1-16. |
| F4 | M | **Corrigé** — `AMOUNT_SIGN_MISMATCH` bloquante et non confirmable (arbitrage orchestrateur) : `CsvError::AmountSignMismatch` en `HardFailure`, `422 BANK_CSV_AMOUNT_SIGN_MISMATCH`, preview et import, `confirmPartialImport` sans effet ; AC3, AC14-3, AC15, AC16, CHANGELOG, manuel ; mutation M14. C-26-1-15 (amende C-26-1-5). |
| F5 | M | **Corrigé** — horodatages attribués : 26-1 `20261011000001`, 26-2 `20261011000002` ; la 26-1 fusionne en premier. C-26-1-18. |
| F6 = R-4 | M / L | **Corrigé** — AC6 (P6) et T2 nomment `total, 78` → 79 **et** `total - 44` → `total - 45`, frontière 34 inchangée, commentaire d'historique et texte de `common/mod.rs`. |
| R-3 | M | **Corrigé** — AC6 : ligne `crates/kesh-db/migrations.sha384` après la dernière modification du fichier ; T2. |
| R-5 = F8 | M / L | **Corrigé** — AC16 : noms et **motifs distincts**, même fixture téléversée sous un autre nom (buffer), scénarios indépendants. C-26-1-22. |
| R-6 | L | **Corrigé** — AC16 : test axe neuf sur `/bank-import/profiles/new`, mode séparé coché. |
| R-7 | L | **Corrigé** — `COMMENT` sans `;` ; interdiction de `;`/`--` dans les littéraux de la migration. C-26-1-19. |
| R-8 = F9 | L | **Corrigé** — fixture sans LF final (comme le fichier réel) ; tableau de structure complété. |
| R-9 | L | **Corrigé** — AC17 : `assert!` sans valeur interpolée, erreurs du parseur affichées par code et numéro seulement. |
| R-10 | L | **Tranché par l'orchestrateur** — fixture synthétique confirmée (dépôt public) ; le nombre annoncé et somme = final − initial prouvés par le test `#[ignore]` exécuté au gate, en agrégats (AC17, C-26-1-1). |
| R-11 = F13 | L | **Corrigé** — 78 avant, 79 après, 80 après union avec la 26-2 (à la 26-2). |
| R-12 | L | **Corrigé** — angle mort de la garde AC5 écrit (doc-comment, manuel) et fixé par un test (AC14-6-bis). C-26-1-17. |
| F7 | L | **Corrigé** — test AC9 placé dans `migrations_upgrade_path.rs` (déjà autorisé). C-26-1-20. |
| F10 | L | **Hors périmètre, écrit** — devise forcée à CHF, préexistante : § « Ce qui reste hors périmètre » + phrase au manuel ; issue à ouvrir par l'orchestrateur. |
| F11 | L | **Corrigé** — manuel : en colonne unique, signe gardé tel quel, non contrôlé. |
| F12 | L | **Corrigé** — AC5 et Dev Notes : `headers()?.position()`, pas `reader.position()` ; test AC14-6. |
| F14 | L | **Corrigé** — mutations M11 (export CSV), M12 (`fields_changed`), M13 (longueur de la garde) ; plus M14-M16 nées de la remédiation. |

Reclassement : aucun finding MEDIUM+ reclassé en dette. Symptômes grepés après patch dans la fiche et le registre
(`ligne 10`, `erreur de ligne`, `400`, `BANK_CSV_PROFILE_VALIDATION`, `79 attendus`, `INVALIDDATE`) : résidus
corrigés.


### Validation P2 — bilan finding par finding

Validation P2 (Opus, lentilles R et F, contexte frais) : **R 0C/0H/3M/11L** (R2-1 à R2-14), **F 0C/0H/5M/8L** (F2-1 à
F2-13). Après dédoublonnage : **5 MEDIUM, 14 LOW distincts** (19). Douze des treize findings F et les trois MEDIUM de R
sont **nés de la remédiation P1** (refus bloquant, AC3-bis) ; un seul est d'origine (F2-13). Défauts **distincts** de
ceux de la P1 : signal D5 déclaré par l'orchestrateur, **pas de découpage**. S'y ajoute l'arbitrage **(0)** de
l'orchestrateur (données personnelles), traité en premier, dans un commit à part.

**Trend** : P1 **1 HIGH / 7 MEDIUM** → P2 **0 HIGH / 5 MEDIUM** (Sonnet → Opus). La remédiation P2 change encore une
règle métier (compteur de signe, deux messages) : P3 complète (Sonnet R/F), pas une passe ciblée.

| Finding | Sév. | Traitement |
|---|---|---|
| (0) orchestrateur | — | **Corrigé** (commit `deec289e`) — somme, nombres de mouvements, de sorties, d'entrées et de lignes du fichier réel retirés de la fiche, du registre et du prompt P2 ; le contrôle du vrai fichier ne rend que « conforme / non conforme » ; date de début de la fixture décalée (coïncidait avec une valeur réelle). Contrôle sans affichage par un script lisant le vrai fichier : 0 valeur réelle restante. ⚠️ L'**historique** de la branche (commits antérieurs à `deec289e`) porte encore ces chiffres : signalé à l'orchestrateur. |
| R2-1 = F2-1 | M | **Corrigé** — AC3 : `sign_total` et `sign_errors` propres au signe, indépendants du plafond général ; décision sur `sign_total > 0` ; AC14-7-bis (a) et (b) ; M17. Arbitrage (1). |
| R2-2 = F2-4 | M | **Corrigé** — AC3 : le panneau d'erreur garde `details` et liste les lignes du refus (numéro, message traduit, valeur) ; AC16 l'asserte ; M22. Arbitrage (2). |
| R2-3 = F2-3 | M | **Corrigé** — AC18 réécrit § « Rejet partiel » (sur confirmation ; exception du signe), contrôle PDF. Arbitrage (4). |
| F2-2 | M | **Corrigé** — AC3-bis : table littérale `CSV_LINE_ERROR_LABELS` (`csv-line-error-label.ts`), famille `bank-import-csv-errors-` déclarée dans `MOTIFS_DYNAMIQUES`/`CARDINALITES` (6), recompte honnête d'`ATTENDU` avec lignes d'historique ; Dev Note fausse corrigée. Arbitrage (5). |
| F2-5 | M | **Corrigé** — bloquant même pour une extourne isolée (arbitrage (3)) ; conséquence écrite (AC3, manuel, CHANGELOG, hors périmètre) ; deux messages selon `whole_debit_column` ; AC14-7-ter, AC15 ; M21. C-26-1-23. |
| R2-4 = F2-6 | L | **Corrigé** — `BANK_CSV_PARTIAL_FAILURE` (code) / `AppError::BankCsvParsePartialFailure` (variant). |
| R2-5 = F2-7 | L | **Corrigé** — attribution à la fonction locale de `migrations_upgrade_path.rs` ; inventaire `(44|78)` complet ; AC9 résout l'index par une fonction locale `index_of_migration`, sans `mod common`. |
| R2-6 = F2-8 | L | **Corrigé** — bandeau « Amendé par C-26-1-16 » sur C-26-1-8 ; corps de C-26-1-7 reformulé. |
| R2-7 | L | **Corrigé** — « R 0C/0H/4M/8L » au bilan P1. |
| R2-8 | L | **Corrigé** — M18 (débit positif sous `Negative`), M19 (zéro strict), M20 (`headers()?.position()`). |
| R2-9 = F2-11 | L | **Corrigé** — préséances écrites en AC3 (encodage, puis signe, puis autres ; même ligne : la date d'abord) ; AC14-7-quater. |
| R2-10 | L | **Corrigé** — CHANGELOG : `debitSign` obligatoire, deux 422 neufs. |
| R2-11 = F2-10 | L | **Corrigé** — AC16 cite le scan axe existant de la liste et place le neuf à côté. |
| R2-12 | L | **Corrigé** — `kesh-import` non publié : semver sans objet (Dev Notes). |
| R2-13 | L | **Corrigé** — AC17 asserte la répartition (débits < 0, crédits > 0) sans nombre ; AC14-7 fixe `+500.00`. |
| R2-14 | L | **Corrigé** — Vitest (e) décocher / recocher. |
| F2-9 | L | **Corrigé** — AC16 : « aucun import de plus » (compte avant/après), assertion insensible à la casse. |
| F2-12 | L | **Corrigé** — AC6 : jusqu'au merge, la ligne `sha384` suit le fichier, recalculée dans le même commit. C-26-1-24. |
| F2-13 | L | **Corrigé** (arbitrage (4)) — AC18 réécrit « Import avec un profil existant » d'après l'écran réel, vérifié au code. *(Retiré en P3 : la section appartient à la 26-3, C-26-1-27.)* |

Reclassement P2 : aucun. Symptômes grepés après patch (valeurs) : `BANK_CSV_PARSE_PARTIAL_FAILURE`, `common/mod.rs`,
`physique`, `messageI18nKey`, `amount-sign-refused\b`, `aucun import n'apparaît`, `cargo search`, et les valeurs du relevé réel
(par script, sans affichage) — résidus corrigés ; le script de contrôle des données réelles rend 0.


### Validation P3 — bilan finding par finding

Validation P3 (Sonnet, lentilles R et F, contexte frais) : **R 0C/0H/2M/13L** (R3-1 à R3-15), **F 0C/0H/2M/7L** (F3-1 à
F3-9). Après dédoublonnage : **2 MEDIUM, 18 LOW distincts** (20). Les deux MEDIUM sont **nés de la remédiation P2**,
distincts de ceux des passes précédentes ; aucun défaut de conception d'origine au-dessus de LOW. Signal D5 déclaré,
pas de découpage.

**Trend** : P1 **1 HIGH / 7 MEDIUM** (Sonnet) → P2 **0 HIGH / 5 MEDIUM** (Opus) → P3 **0 HIGH / 2 MEDIUM** (Sonnet) →
P4 **passe ciblée** (Haiku, une lentille) sur le seul commit de remédiation P3 : convergence ; la remédiation P3 ne
touche que la fiche, le registre et les prompts (aucun code), mais elle redéfinit une règle (`whole_debit_column`) —
c'est elle que la P4 vise.

| Finding | Sév. | Traitement |
|---|---|---|
| R3-1 = F3-1 | M | **Corrigé** (arbitrage (1)) — la 26-1 ne réécrit plus « Import avec un profil existant » (à la 26-3) ; elle corrige « Détection des doublons » (que la 26-3 ne touche pas, vérifié en lecture) ; le refus dit « redéposez le fichier ». C-26-1-27. |
| R3-2 = F3-2 | M | **Corrigé** (arbitrage (2)) — `whole_debit_column ⇔ d ≥ 2 et e = d` sur les débits lus ; `RowParse` / `DebitObservation` internes, rien dans `CsvLineError` ; test `parser.rs` l. ≈ 1064 adapté ; cas limites fixés (AC14-7-ter). C-26-1-25. |
| R3-3 | L | **Corrigé** — cas limites de 7-ter ; mutations M23 à M27. |
| R3-4 = F3-3 | L | **Corrigé** — `SITES_GABARIT_ATTENDUS`, `sitesGabarit` 10 → 11, `relais`/`sitesNonResolus` inchangés à mesurer, titres « 8 préfixes » / « 10 sites » ; `CARDINALITES` lue depuis `CSV_LINE_ERROR_LABELS`. |
| R3-5 | L | **Corrigé** — `common::migrations_before` réutilisé (`#[allow(dead_code)] mod common;`). C-26-1-26. |
| R3-6 | L | **Corrigé** — AC8 nomme le test `debit_sign_enum` → `Invariant` et le helper `make_new_profile`. |
| R3-7 | L | **Corrigé** — corps de C-26-1-5, 14 et 15 réécrits (plus seulement des bandeaux) ; note de correction ajoutée au prompt P3 (« M1-M16 » → M1-M22). |
| R3-8 | L | **Corrigé** — « Détection des doublons » réécrite d'après `dedup_key_scalar` et l'écran ; la sous-section UBS dit que la référence nourrit la détection. |
| R3-9 | L | **Corrigé** — CHANGELOG : lignes vides en *Précisé* (comportement inchangé) ; colonne `debit_sign` de l'export ; `debitSign` dans les `GET`. |
| R3-10 | L | **Corrigé** — préséances complètes (erreurs d'avant la boucle) ; garde AC5 placée avant le premier `iter.next()`. |
| R3-11 | L | **Corrigé** — test de forme de la fixture (AC14-10) ; M28. |
| R3-12 | L | **Corrigé** — `CsvLineErrorCode::all()` gardée par un `match` exhaustif ; le test n'énumère plus à la main. |
| R3-13 | L | **Corrigé** — `errorDetails` posé aux deux `catch`, remis à zéro aux trois sites ; clé de troncature existante réutilisée. Patrons Vitest nommés en AC12 (aucun test de composant dans `features/bank-import/`). |
| R3-14 | L | **Corrigé** — `.` pour toute apostrophe dans les motifs `grep` du PDF. |
| R3-15 = F3-7 | L | **Corrigé** — liste complète des conflits attendus ; `sitesTotal` recompté au rebase, jamais additionné. |
| F3-4 | L | **Corrigé** — `d ≥ 2` : un seul débit en défaut ne reçoit jamais le conseil de changer le réglage. |
| F3-5 | L | **Corrigé** — manuel : les trois formes de signe contraire. |
| F3-6 | L | **Corrigé** — bump par lignes ancrées `^version = ` ; `yaml-rust2` et `ravif` restent en 0.13.0. |
| F3-8 | L | **Corrigé** — troisième scénario E2E neuf (l'existant est `skip`) ; « aucun import de plus » par le `total` de l'API (liste plafonnée à 20) ; conduite écrite pour une violation axe préexistante. |
| F3-9 | L | **Corrigé** — AC14-3 à quatre champs ; `\b(43|35)\b` ajouté à l'inventaire ; `make_new_profile` (avec R3-6) ; corps du registre (avec R3-7) ; position de la garde (avec R3-10). |

Reclassement P3 : aucun. Symptômes grepés après patch (valeurs) : `importez à nouveau`, `index_of_migration`,
`relais`, `Import avec un profil existant` (hors bilans), `M1-M16`, `lignes vides non comptées` (CHANGELOG), et les
valeurs du relevé réel (par script, sans affichage) — résidus corrigés.

### Validation P4 ciblée et clôture de la validation

Passe ciblée (Haiku 4.5, une lentille « chasseur de régression », contexte frais, prompt versionné
`26-1-validate-prompt-p4-ciblee.md`, rapport `/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p4-ciblee.md`) sur le
seul commit de remédiation P3 : **0C/0H/1M/2L**.

| Finding | Sév. | Traitement |
|---|---|---|
| F4-1 | M | **Corrigé** — corps de C-26-1-23 au registre aligné sur C-26-1-25 (`d ≥ 2` et `e = d` ; un seul débit en défaut → message « lignes listées »). Le MEDIUM, vérifié par lecture (le `grep -F` du rapport ratait la l. 323 à cause du gras `**toutes**`), est un résidu de propagation de la P3. |
| F4-2 | L | **Corrigé** — message AC3 : « Tous les débits **lus** dans le fichier… » ; table AC13 alignée (« au moins deux »). |
| F4-3 | L | **Corrigé** — `EncodingMismatch` n'est plus classé deux fois dans les préséances (renvoi à (1)). |

**Trend de la validation** : P1 **1 HIGH / 7 MEDIUM** (Sonnet R/F) → P2 **0 HIGH / 5 MEDIUM** (Opus R/F) → P3 **0 HIGH /
2 MEDIUM** (Sonnet R/F) → P4 ciblée **0 HIGH / 1 MEDIUM** (Haiku, une lentille). Reclassement : aucun. La remédiation
finale ne touche **que du texte** (fiche, registre) — **aucune ligne de production** : la boucle se clôt (règle de la
passe ciblée). Symptômes regrepés sur `_bmad-output/` : `toutes les valeurs non nulles` (seul reste : le contexte
historique de C-26-1-25 et le prompt P3, légitimes), `Tous les débits du fichier` (0), `(ci-dessous), en-têtes` (0).
