# Story 26.1 : Importer le CSV d'UBS tel quel — sens des montants et lignes à ignorer

## Status

**ready-for-dev** — fiche créée le 2026-10-10 en autonomie (skill `bmad-create-story`) ; **validation P1 (Sonnet
R/F : 1 HIGH, 7 MEDIUM, 12 LOW, après dédoublonnage) remédiée** le 2026-10-10 ; validation P2 (Opus R/F) à suivre
(prompt `26-1-validate-prompt-p2.md`). Choix tranchés : **C-26-1-1 à C-26-1-22**, registre
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

- `kesh-import` : `parse_csv_collect` continue de **collecter** toutes les erreurs de ligne (plafond
  `MAX_CSV_LINE_ERRORS`), puis, si au moins une est `AmountSignMismatch`, rend
  `ParseCsvOutcome::HardFailure(CsvError::AmountSignMismatch { errors, total_errors, truncated })` — nouvelle variante
  documentée, où `errors` ne garde **que** les erreurs de signe (les autres erreurs de ligne du même fichier ne sont
  pas rapportées : régler le profil d'abord) et `total_errors` les compte toutes (de signe). `parse_csv` (wrapper) rend
  la même `Err`. La décision vit dans `kesh-import`, pas dans la route : tout appelant en hérite.
- `kesh-api` : bras **explicite** dans `map_csv_error` → `AppError::BankCsvAmountSignMismatch { lines, total_errors,
  truncated }` → **`422 BANK_CSV_AMOUNT_SIGN_MISMATCH`**, message traduit `bank-import-csv-errors-amount-sign-refused`
  (argument `{ $count }` : « { $count } ligne(s) portent un montant de signe contraire à leur colonne. Réglez le sens
  des débits du profil (Administration → Profils bancaires), puis importez à nouveau. »), `details: { lines: [ … ],
  totalErrors, truncated }` (même forme de ligne que `BANK_CSV_PARSE_PARTIAL_FAILURE`, via
  `csv_line_error_to_payload`). Rendu **en preview comme en import**, que `confirmPartialImport` vaille `true` ou
  `false` — les deux chemins testés (AC15).
- Écran (`BankImportUpload.svelte`) : le panneau d'erreur existant (`data-testid="bank-import-error"`,
  `data-error-code`) montre le message ; **aucune case de confirmation** n'est offerte pour ce code (vérifier que le
  panneau ne la dérive pas de la présence de `details.lines`). Si le panneau d'erreur liste déjà `details.lines` pour
  `BANK_CSV_PARSE_PARTIAL_FAILURE`, il les liste aussi ici avec leur message traduit (AC3-bis).

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
- Le panneau des lignes invalides (aperçu, `warnings.invalidLines`) **et** celui d'AC3 affichent, pour chaque ligne,
  `i18nMsg(line.messageI18nKey, line.code)` à côté du code. Les en-têtes en dur du tableau (« Ligne », « Code »,
  « Valeur ») restent hors périmètre.
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
qui vaut la ligne suivante (F12, C-26-1-21). Ce n'est pas une détection de la ligne de titres : le nombre reste celui
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
  `grep -h '^version = ' crates/*/Cargo.toml | sort | uniq -c` → `10 version = "0.13.1"`.
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
  -f1`), **après la dernière modification du fichier** — l'y inscrire, c'est le déclarer publié (P8) ; le test
  `published_migrations_keep_their_checksums` (`test_schema_guard.rs` ≈ l. 562, 648-654) rougit sinon. Au rebase de la
  26-2 : deux lignes, union.
- **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et inspection de chaque site
  (`crates/kesh-db/tests/migrations_upgrade_path.rs`, `lettering_documents_backfill.rs`,
  `audit_log_company_id_backfill.rs`, `common/mod.rs`) ; un site positionnel sans garde fail-loud est un MEDIUM.
  **Deux littéraux nommés, à bouger du même pas** (F6/R-4) dans `crates/kesh-db/tests/migrations_upgrade_path.rs` :
  `assert_eq!(total, 78, …)` (l. ≈ 110, et son message « 78 migrations attendues … ») → **79**, et
  `let n_before_upgrade_window = total - 44;` (l. ≈ 194, et `.expect("apply_migrations_up_to(total - 44) failed")`)
  → **`total - 45`**, frontière **inchangée** (79 − 45 = 34) ; une ligne d'historique au commentaire (« Story 26-1 :
  44 → 45, frontière inchangée (79 - 45 = 34) ») et le texte du `assert!` de `common/mod.rs` (l. ≈ 36-46 : `total -
  44`, `total == 78`) suivent. Bumper `total` seul ferait glisser la frontière et rougir sur `COUNT(accounts)`, échec
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
base **refuse** `header_row_count = 21` et `debit_sign = 'autre'` par ses CHECK (`DbError::CheckConstraintViolation`).
Les `INSERT` SQL bruts des tests (`crates/kesh-db/tests/companies_repository.rs` ≈ l. 938,
`crates/kesh-api/tests/exports_global_e2e.rs` ≈ l. 1566) restent valides par le `DEFAULT` — ne les toucher que si un
test l'exige.

**AC9 — Le profil d'avant reste ce qu'il était (preuve par la migration).** Un test **dans
`crates/kesh-db/tests/migrations_upgrade_path.rs`** — fichier déjà inscrit à `ALLOWED_REAL_MIGRATOR_FILES`
(`test_schema_guard.rs` l. ≈ 38-62), ce qui évite d'élargir cette liste (F7, C-26-1-20) — applique les migrations **jusqu'à la précédente** (`apply_migrations_up_to(&pool, migrations_before(<version>,
"bank_profiles_debit_sign"))`, résolution **par version**, P6), insère un profil `header_row_count = 5` en mode
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
- **Vitest** (nouveau `BankProfileForm.test.ts`, patron des tests de composants voisins — vérifier qu'il en existe un et
  le suivre) : (a) mode séparé + « signe moins » → la requête porte `debitSign: 'negative'` et
  `debit_credit_split` ; (b) mode colonne unique → `debitSign: 'positive'` même après avoir choisi « signe moins » puis
  décoché ; (c) un profil existant `negative` s'ouvre avec le bon bouton coché ; (d) le champ des lignes porte
  `max="20"`.

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
| `bank-import-csv-errors-amount-sign-refused` | neuve (AC3, message **du refus global** `422 BANK_CSV_AMOUNT_SIGN_MISMATCH`), avec `{ $count }` |
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
   false })`, `errors` ne portant **que** des `AmountSignMismatch` (9) — et **aucune** transaction rendue : #620 ne
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
   acceptés (`0.00`, `-0.00`) ;
8. AC1 : `validate` refuse `Negative` en colonne unique, accepte `Positive` ; `DebitSign` serde : `"positive"` /
   `"negative"` acceptés, `"Negative"` et `"autre"` refusés ;
9. AC3-bis (`error.rs`) : pour **chaque** variant de `CsvLineErrorCode`, `as_str()` ==
   `serde_json::to_value(code)` (liste des variants énumérée dans le test ; un variant ajouté sans y figurer doit
   faire échouer la compilation du `match` d'`as_str`, pas le test).

**AC15 — Test d'intégration de route.** Dans `crates/kesh-api/tests/bank_imports_e2e.rs` (ou `bank_profiles_e2e.rs`,
au plus près de l'existant) : création du profil UBS **par `POST /api/v1/bank-profiles`** (pas par le dépôt — c'est le
chemin DTO → base → `db_profile_to_csv_profile` qu'on veut traverser), réponse `debitSign == "negative"`,
`headerRowCount == 9` ; import de la fixture par le multipart existant (`build_csv_multipart`) ; en base,
`SELECT COUNT(*), SUM(amount), SUM(amount < 0) FROM bank_transactions WHERE …` → `11`, `-236.25`, `9`. Plus (statuts
réels, AC10) : une requête sans `debitSign` → **422** (rejet `Json` d'axum), rien en base ; `negative` en colonne
unique → **422 `BANK_CSV_PROFILE_INVALID`** ; `headerRowCount: 21` → **422 `BANK_CSV_PROFILE_INVALID`** ; le profil UBS
avec `headerRowCount: 10` → **422 `BANK_CSV_HEADER_SKIPS_DATA`**, `details.line == 11`, en preview **et** en import ;
le profil UBS réglé `positive` → **422 `BANK_CSV_AMOUNT_SIGN_MISMATCH`**, `details.totalErrors == 9`,
`details.lines[0].code == "AMOUNT_SIGN_MISMATCH"` (graphie exacte sur le fil, AC3-bis), en preview **et** en import
avec `confirmPartialImport=true` — **aucune** transaction écrite (`COUNT(*) = 0`) ; et un CSV à `INVALID_DATE`
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
`data-error-code="BANK_CSV_AMOUNT_SIGN_MISMATCH"` et le **message traduit** (« … réglez le sens des débits … ») ;
**aucune** case « importer les lignes valides quand même » n'est offerte ; aucun import n'apparaît dans la liste.
Un troisième contrôle, dans un scénario existant ou neuf à `INVALID_DATE` : le tableau des lignes invalides affiche le
message traduit à côté du code (AC3-bis).

**Accessibilité** (R-6) : aucune spec existante ne scanne le formulaire de profil (le seul `AxeBuilder` du domaine
porte sur `/bank-import`, `bank-import.spec.ts` l. ≈ 225-231). Un test axe **neuf** sur `/bank-import/profiles/new`,
**mode séparé coché** (groupe radio visible), zéro violation. Les specs existantes qui remplissent le formulaire (mode
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
  est refusé en entier** (régler le sens des débits, puis réimporter) ; le numéro de ligne annoncé pour une ligne qui
  suit une ligne vide peut être celui de la ligne vide ; la garde contre un nombre trop grand ne voit qu'une ligne
  **datée** ; **la colonne Devise n'est pas lue** : tout import CSV est en CHF.
- `CHANGELOG.md` : section **`## [0.13.1] — Non publié`** (forme exacte attendue par `scripts/prepare-release.sh`
  l. 237 ; à créer si absente, à **compléter par union** si la 26-2 ou la 26-3 l'a créée entre-temps) : *Corrigé* —
  #620 (le sens des débits devient un réglage ; un fichier dont un montant a un signe contraire à sa colonne est
  refusé, sans import partiel possible) ; *Modifié* — #621 (jusqu'à 20 lignes à ignorer, lignes vides non comptées ;
  refus d'un nombre qui avalerait une transaction datée) ; *Modifié* — **graphie des codes d'erreur de ligne** de
  l'import CSV (`warnings.invalidLines.lines[].code`, `details.lines[].code`) : `INVALIDDATE` → `INVALID_DATE`,
  `INVALIDAMOUNT` → `INVALID_AMOUNT`, `AMBIGUOUSDEBITCREDIT` → `AMBIGUOUS_DEBIT_CREDIT`, `EMPTYMANDATORYFIELD` →
  `EMPTY_MANDATORY_FIELD`, `ROWTOOSHORT` → `ROW_TOO_SHORT` (changement visible d'un client d'API) ; un encadré **« À faire en passant à la 0.13.1 »** : sauvegarder avant ; la base passe à
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
| M16 | le panneau n'affiche plus `messageI18nKey` | AC16 (message traduit) — et un Vitest si le composant en a un |

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
- le contrôle AC17 sur le vrai fichier ;
- **ne déclarer que ce qui a tourné**, avec ses chiffres et son périmètre ; recompter tout décompte écrit.

## Tâches

- [ ] **T1 — `kesh-import`** (AC1-5, AC3-bis, AC14 tests 1-9) : `DebitSign`, champ, `validate`, `parse_row`, garde
  AC5, variants `CsvLineErrorCode::AmountSignMismatch`, `CsvError::AmountSignMismatch` / `HeaderSkipsData`,
  `CsvLineErrorCode::as_str()`, mise à jour des **tous** les littéraux `CsvProfile { … }` des tests
  (`profile.rs` ≈ l. 244, `parser.rs` ≈ l. 575 et 1040), fixture + README des fixtures.
- [ ] **T2 — Migration et base** (AC6, AC7, AC9) : migration, squash régénéré, ligne de `migrations.sha384`,
  P5/P6 (`total` 78 → 79 **et** `total - 44` → `total - 45`)/P7, assertions de version, test de migration dans
  `migrations_upgrade_path.rs` ; bump des dix `Cargo.toml` + `Cargo.lock` dans le **même commit** que la migration (P2-bis).
- [ ] **T3 — Entité, dépôt** (AC8).
- [ ] **T4 — API** (AC3, AC3-bis, AC5 côté API, AC10, AC11, AC15) : DTO, réponse, conversion,
  `AppError::BankCsvHeaderSkipsData` et `AppError::BankCsvAmountSignMismatch` (bras explicites de `map_csv_error`),
  `csv_line_error_to_payload` par `as_str()`, export CSV, test `is_required`, tests d'intégration.
- [ ] **T5 — Frontend et i18n** (AC3, AC3-bis, AC12, AC13) : types, formulaire, panneaux d'erreur (message traduit,
  pas de case de confirmation pour `BANK_CSV_AMOUNT_SIGN_MISMATCH`), Vitest, quatre locales.
- [ ] **T6 — E2E** (AC16).
- [ ] **T7 — Documentation** (AC18) : manuel + PDF, CHANGELOG (dont la graphie des codes) ; grep de la valeur « 5 »
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

**La devise forcée à CHF** (F10) : `parse_row` écrit `currency: "CHF"` pour toute ligne CSV, la colonne `Devise`
d'UBS (col. 4) et la clé `Évaluation en:` ne sont pas lues — un compte en devise étrangère s'importerait en CHF sans un
mot. Défaut **préexistant**, de la même famille que #620 mais hors de #620/#621 : l'orchestrateur ouvre une issue ; la
story se borne à l'écrire au manuel (AC18). Lecture du préambule (IBAN, soldes, période — #621 reste ouverte) ; profil UBS prêt à l'emploi (C-26-1-10) ; MT940
(#618) ; détection de la ligne de titres (décision 2) ; réglage du sens en colonne unique (C-26-1-2) ; colonne
`counterparty` dans le formulaire ; traduction des libellés de colonnes en dur.

### Points de vigilance

- **Clés i18n dynamiques** (AC3-bis) : le panneau lit `line.messageI18nKey` à l'exécution ; `npm run
  lint-i18n-ownership` et `i18n-keys.test.ts` ne voient que des clés littérales. Vérifier que les six clés
  `bank-import-csv-errors-*` des codes de ligne sont bien livrées au frontend par le mécanisme de chargement des
  messages (et pas seulement utilisées côté serveur) ; si le lint exige une déclaration de propriété pour une clé lue
  dynamiquement, la faire, ne pas contourner.

- **`kesh-import` est « publiable »** (CI : `cargo publish --dry-run -p kesh-import`). Ajouter un champ à la struct
  publique `CsvProfile` et des variants aux enums publics est une **rupture d'API** au sens semver, sous un bump
  0.13.0 → 0.13.1. Vérifier si le crate est réellement publié sur crates.io (`cargo search kesh-import`, lecture
  seule) : s'il ne l'est pas, le noter au Dev Agent Record et passer ; s'il l'est, le **signaler** à l'orchestrateur
  (question de version propre au crate, hors de cette story). Ne **pas** ajouter `#[non_exhaustive]` de son propre chef
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
- **Conflits attendus au rebase** avec la 26-2 (migration, `docs/migrations-idempotence-audit.md`, compteurs, squash,
  `EXEMPT_MIGRATIONS` si elle en ajoute, CHANGELOG) et la 26-3 (manuel FR, même section *Import bancaire*, CHANGELOG) :
  **union** partout, compteurs **recomptés**, PDF régénéré après union.

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
- Registre : `_bmad-output/implementation-artifacts/epic-26-choix-autonomes.md`, C-26-1-1 à C-26-1-22.
- CLAUDE.md : § *Migration breaking policy* (P1-P8), § *Test Locally First*, § *Review Iteration Rule*, § *Règle de
  commit et push* (CHANGELOG, manuels).
- Précédent de bump avec version Cargo dans la story : `0724904c` (Story 15-1a-i), migration
  `20261009000001_journal_entry_lines_lettering.sql`, entrée `EXEMPT_MIGRATIONS` correspondante.
- csv-core 0.1.13 : `src/reader.rs` l. 94 (lignes vides ignorées), l. 665/724 (comptage des lignes) ; csv 1.4.0 :
  `src/reader.rs` l. 1626 (position posée avant la lecture), l. 1323-1326 (`headers()` et sa position).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche créée le 2026-10-10 (Opus 5.5, `bmad-create-story`, autonomie) : analyse des sources, structure du fichier UBS
  relevée **en formes** (aucune donnée copiée), choix C-26-1-1 à 13 consignés au registre.
- Validation P1 remédiée le 2026-10-10 (Opus 5.5) : AC3-bis ajouté (21 AC), choix C-26-1-14 à 22 ; bilan au Change Log.

### File List

## Change Log

| Date | Auteur | Changement |
|---|---|---|
| 2026-10-10 | Opus 5.5 (create-story) | Création de la fiche ; 20 AC ; statut ready-for-dev ; validation à suivre. |
| 2026-10-10 | Opus 5.5 (remédiation validation P1) | Validation P1 (Sonnet, lentilles R et F, contexte frais) : **R 0C/0H/3M/7L** (R-1 à R-12 ; R-5 classé MEDIUM dans son corps), **F 0C/1H/5M/8L** (F1 à F14). Après dédoublonnage : **1 HIGH, 7 MEDIUM, 12 LOW distincts** (MEDIUM : F2, F3=R-1, F4, F5, F6=R-4, R-3, R-5=F8). Bilan finding par finding ci-dessous. Nouvel AC3-bis (21 AC) ; choix C-26-1-14 à C-26-1-22. |

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
