# Story 39-11a : lecteur du CSV « relevé » de Wise (`kesh-import`)

Status: done

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`,
fusionné dans `epic/39-wise` au commit `9c8bc452`), revérifiés à la validation P1. Epic 39, **phase 1 KISS**
(décision du responsable de l'epic du 2026-11-20, § *Phase 1 KISS* du fichier d'epic, G1, G3, G4, G10). **Dépend de** : 38-11a
(lecteur PayPal, validée P1 : en-tête par noms et forme fermée des montants, que cette fiche partage). **Suivie
par** : 39-11b (import), 39-11c (rapprochement).*

## Story

En tant que **Comptable**, je veux que Kesh lise le relevé CSV qu'exporte Wise pour un solde (une devise) et une
période, ligne par ligne, avec le montant, la devise, le solde courant, les parties, la conversion et l'identifiant
Wise, afin que l'import (39-11b) n'ait plus qu'à ranger ces lignes au compte bancaire de la devise.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11a-1` | Lire le CSV Wise par les **noms** de colonnes, quel que soit leur ordre ; toutes les colonnes requises manquantes sont nommées dans le refus. | AC-1, AC-2 |
| `FR-39-11a-2` | Rendre chaque ligne avec identifiant, date, horodatage, montant, devise, solde courant, description, référence de paiement, payeur, bénéficiaire, numéro de compte du bénéficiaire, marchand, conversion, frais indiqués, sens et type. | AC-3 |
| `FR-39-11a-3` | Refuser un fichier à deux devises, un identifiant en double, ou un solde courant discontinu, en nommant la ligne. | AC-4, AC-5 |
| `FR-39-11a-4` | Classer l'identifiant par son **préfixe** (`TRANSFER`, `CARD`, `BALANCE`, `CONVERSION_ORDER`, …, et `FEE-` suivi du préfixe d'origine). | AC-3 |
| `NFR-39-11a-1` | Fonction pure, sans I/O, dans `kesh-import`, publiable sans dépendance interne ; aides communes partagées avec 38-11a, non recopiées. | AC-2, AC-4, AC-6 |
| `NFR-39-11a-2` | Aucune donnée réelle dans le dépôt : fixtures fictives. | AC-7 |

## Critères d'acceptation

1. **AC-1 — Entrée et décodage** (même patron que 38-11a AC-1). `kesh_import::wise::parse_wise_csv(bytes: &[u8]) ->
   Result<WiseCsv, WiseCsvError>` (module neuf `crates/kesh-import/src/wise.rs`, `pub mod wise;` et réexport à côté
   de `parse_csv`, `crates/kesh-import/src/lib.rs:45-55`). `detect_encoding(bytes)` rend `(encodage, bom_len)` ;
   `decode_bytes` (`crates/kesh-import/src/csv/encoding.rs:48`, `:107`) reçoit `&bytes[bom_len..]` (le relevé de
   Wise n'a pas de BOM — relevé sur les fichiers 2024 —, mais un BOM est admis et retranché). Crate `csv` déjà en
   dépendance : `has_headers(false)`, virgule, guillemets doubles, `flexible(true)` en réalité (la largeur se contrôle à la main dans `read_records`, ce qui permet de sauter la ligne vide de fin avant de la compter fausse ; même refus `FIELD_COUNT`) — `flexible(false)` à l'écriture de la fiche (nombre de cellules ≠ en-tête →
   erreur de ligne `FIELD_COUNT`) ; le numéro de ligne d'une erreur est celui du fichier (`record.position().line()`,
   1 = l'en-tête) ; lignes entièrement vides sautées. Fichier vide ou en-tête seul → `WiseCsvError::Empty`.
   **Type d'erreur fermé** : `WiseCsvError` = `Empty`, `Encoding(CsvError)`, `MissingColumns(Vec<String>)`,
   `DuplicateColumn(String)`, `MixedCurrencies(Vec<String>)`, `DuplicateId { id, lines }`, `RunningBalanceBreak {
   line, expected, found }`, `Lines(Vec<WiseLineError>)` ; chaque variante a un code stable (`WISE_CSV_EMPTY`,
   `…_ENCODING`, `…_MISSING_COLUMNS`, `…_DUPLICATE_COLUMN`, `…_MIXED_CURRENCIES`, `…_DUPLICATE_ID`,
   `…_RUNNING_BALANCE_BREAK`, `…_LINES`) que 39-11b place dans `details` de son refus et que 39-11d traduit.
2. **AC-2 — En-tête par noms.** Comparaison après `trim`, sans casse, par l'**aide commune** de 38-11a (G10) : 38-11a
   l'écrit dans `paypal.rs` ; cette story la **déplace** dans un module partagé `crates/kesh-import/src/csv/by_name.rs`
   (index des colonnes par nom, requises manquantes listées dans l'ordre donné, doublon de colonne) et l'appelle des
   deux côtés, tests de 38-11a inchangés en assertions. **Requises** : `TransferWise ID`, `Date`, `Amount`,
   `Currency`, `Description`, `Running Balance`, `Transaction Type`, `Transaction Details Type`. **Facultatives**
   (absentes = vides) : `Date Time`, `Payment Reference`, `Exchange From`, `Exchange To`, `Exchange Rate`, `Exchange
   To Amount`, `Payer Name`, `Payee Name`, `Payee Account Number`, `Merchant`, `Total fees`, `Note`. `Card Last Four
   Digits`, `Card Holder Full Name`, `Attachment` et toute autre colonne sont **ignorées** (jamais lues ni rendues :
   quatre chiffres de carte et nom du porteur n'entrent pas dans Kesh). Requises absentes → `MissingColumns`, toutes
   listées dans l'ordre ci-dessus ; doublon de colonne → `DuplicateColumn`.
3. **AC-3 — Ligne rendue.** `WiseRow { line: usize, id: String, id_prefix: String, is_fee: bool, fee_of:
   Option<String>, date: NaiveDate, date_time: Option<NaiveDateTime>, amount: Decimal, currency: String,
   description: Option<String>, payment_reference: Option<String>, running_balance: Decimal, payer_name:
   Option<String>, payee_name: Option<String>, payee_account: Option<String>, merchant: Option<String>,
   exchange_from: Option<String>, exchange_to: Option<String>, exchange_rate: Option<Decimal>, exchange_to_amount:
   Option<Decimal>, total_fees: Option<Decimal>, direction: WiseDirection, details_type: String, note:
   Option<String> }` ; cellule vide (après `trim`) → `None`, texte gardé tel quel sinon (les descriptions de Wise
   portent des espaces fines insécables U+202F : ni retirées ni remplacées, aucune comparaison ne les lit). `id` : non
   vide, ASCII `[A-Z0-9_-]` après `trim` et majuscules, ≤ 64 caractères (le plus long des relevés 2024 en compte 28) ;
   `id_prefix` = la partie avant le premier chiffre, sans tiret final (`TRANSFER`, `CARD`, `BALANCE`,
   `CONVERSION_ORDER`, `FEE-TRANSFER`, `FEE-BALANCE`, `FEE-CONVERSION_ORDER`, …) ; `is_fee` = l'identifiant commence
   par `FEE-` ; `fee_of` = l'identifiant sans `FEE-`. `direction` = `Credit` | `Debit` (colonne `Transaction Type`,
   `CREDIT` / `DEBIT` dans les relevés, comparée sans casse ; autre → erreur de ligne) ; `currency` trois lettres
   ASCII, majuscules ; `details_type` tel quel (`TRANSFER`, `CARD`, `CONVERSION`, `DEPOSIT` relevés). Le lecteur **ne
   classe rien d'autre** : conversions, virements, versements Stripe sont l'affaire de 39-11c.
4. **AC-4 — Dates, montants, signes.** `Date` au format `JJ-MM-AAAA` (relevés 2024) ou `AAAA-MM-JJ` ; autre → erreur de
   ligne `DATE` ; année hors `ANNEE_MIN..=ANNEE_MAX` (`lib.rs:35-43`, `annee_recevable`) → erreur. `Date Time` au
   format `JJ-MM-AAAA HH:MM:SS` suivi ou non de `.fff` (exactement trois décimales ; relevés : millisecondes ; seconde intercalaire `:60` et forme ISO refusées, remédiation P1 L3), lu en `NaiveDateTime` **sans
   conversion de fuseau** (limite au manuel par 39-11d) ; autre forme → erreur `DATE_TIME`. **Montants** : la forme
   fermée de 38-11a AC-5 (`^-?(\d+|\d{1,3}(,\d{3})+)(\.\d{1,2})?$`, virgule seulement en séparateur de milliers,
   bornée à 15 chiffres : fusion `ff5b690e`, `csv/amount.rs`), par la **même fonction**, restée dans `csv/amount.rs` (venue de 38-11a ; l'aide d'en-tête est dans `csv/by_name.rs`) (⚠️ **pas** `parse_amount`,
   `csv/parser.rs:816`, privé, qui retire toute virgule sans la contrôler : 38-11a AC-5 l'écarte pour cette raison,
   et ne le rend pas `pub(crate)`) — pour `Amount`, `Running Balance`, `Total fees`, `Exchange To Amount` (les
   relevés portent `-9.99`, `9.99` et des entiers sans décimale) ; `Exchange Rate` : `^\d+(\.\d{1,10})?$` (cinq
   décimales relevées). Contrôles : `Credit` avec `amount < 0` ou `Debit` avec `amount > 0` → `SIGN_MISMATCH` ;
   ligne de frais créditée → `FEE_SIGN` (aucune dans les relevés 2024). Erreurs agrégées (`WiseLineError { line,
   field, code, value }`, valeur tronquée à `MAX_CSV_LINE_ERROR_VALUE_CHARS`, plafond `MAX_CSV_LINE_ERRORS`,
   `crates/kesh-import/src/error.rs:55`, `:49`) ; une erreur refuse le fichier entier.
5. **AC-5 — Contrôles du fichier.** Toutes les lignes de la même devise, sinon `MixedCurrencies` ; un identifiant en
   double dans le fichier → `DuplicateId` (une conversion porte le même identifiant dans **deux** fichiers, jamais deux
   fois dans un) ; lignes **triées par ordre chronologique** : `date_time` (valeur lue, jamais la chaîne : `JJ-MM-AAAA`
   ne se trie pas en texte) quand **toutes** les lignes en ont un, à égalité, ou dès qu'une seule en manque (colonne
   absente comprise), ordre **inverse** du fichier (Wise écrit la plus récente en
   tête ; une ligne de frais précède d'une milliseconde sa ligne d'origine) ; puis **continuité** : chaque
   `running_balance` = le précédent + `amount`, sinon `RunningBalanceBreak`. `WiseCsv { currency, rows
   (chronologiques), opening_balance = premier solde − premier montant, closing_balance = dernier solde, first_date,
   last_date }`.
6. **AC-6 — Pureté et publication.** Aucune dépendance neuve ni interne ; `cargo publish --dry-run -p kesh-import`
   vert ; tests de 38-11a et de `parser.rs` inchangés en assertions ; `parse_amount` non touché.
7. **AC-7 — Fixtures fictives.** `crates/kesh-import/tests/fixtures/wise-releve-{chf,eur,usd}-fictif.csv` (et leur
   ligne au catalogue `crates/kesh-import/tests/fixtures/README.md`, qui dit tous ses fichiers synthétiques) : 23
   colonnes dans l'ordre de Wise (G1), **aucune donnée réelle** (noms inventés, IBAN d'exemple publiés comme `CH93
   0076 2011 6238 5295 7`, identifiants inventés), **toutes les dates en 2020** (exercice du seed `with-company`
   ouvert au 01.01.2020, comme la fixture de 38-11a : l'E2E de 39-11d importe sans créer d'exercice), solde courant
   continu, et au moins : une conversion CHF↔EUR (même `BALANCE-…` dans deux fichiers, le montant cible égal à
   `Exchange To Amount` de la ligne source) avec sa ligne `FEE-BALANCE-…` dans la devise source, horodatée une
   milliseconde avant ; une conversion EUR→USD avec ses frais ; une conversion `CONVERSION_ORDER-…` ; un virement
   sortant vers l'IBAN fictif de la société avec sa ligne `FEE-TRANSFER-…` ; un virement sortant vers un numéro de
   compte non IBAN ; deux crédits `DEPOSIT` « Argent reçu de Stripe » (payeur `Stripe Payments Europe`, fictif) ;
   deux achats par carte (marchands inventés) ; un paiement à un tiers ; une description avec une espace U+202F ; deux
   lignes de jours différents dont l'ordre en texte (`JJ-MM-AAAA`) est l'inverse de l'ordre chronologique. Elles
   servent à 39-11b, 39-11c et à l'E2E de 39-11d.

## Tâches

- [x] T1 — Aide « en-tête par noms » et forme fermée des montants, déplacées de `paypal.rs` dans `csv/by_name.rs`,
  appelées des deux côtés (AC-2, AC-4, G10).
- [x] T2 — `wise.rs` : types, erreurs fermées, lecture, préfixes (AC-1, AC-3).
- [x] T3 — Dates et horodatages, montants, signes ; tri et continuité (AC-4, AC-5).
- [x] T4 — Fixtures fictives, tests, `cargo publish --dry-run -p kesh-import` (AC-6, AC-7).

## Stratégie de tests

Un unitaire dans `wise.rs` (débordement du chaînage du solde, devenu inatteignable par un fichier) ; le reste en intégration `crates/kesh-import/tests/wise_csv.rs` sur les fixtures : nombre de lignes,
valeurs de chacune, préfixes, `fee_of`, soldes d'ouverture et de clôture ; colonnes dans un autre ordre ; colonnes
de carte absentes du résultat ; requises manquantes ; doublon de colonne ; deux devises ; identifiant en double ;
rupture du solde courant ; tri sur l'horodatage lu (cas des deux jours de la fixture) ; signe contraire ; date
`MM/JJ/AAAA` refusée ; montant `1.500,00` et `1,50` refusés ; cours à onze décimales refusé ; BOM admis ; U+202F gardé
dans la description ; plafond d'erreurs. Une ligne sans `Date Time` dans un fichier qui en a ailleurs → tri sur l'ordre inverse du fichier seul. Tests de 38-11a verts sans
changement d'assertion après le déplacement des aides. Gate ciblé `binary(wise_csv)` et tests de `kesh-import` ; gate complet au push.

## Modules touchés (1)

`kesh-import` seul.

## Note de version

*Ajouté* — Kesh lit le relevé CSV d'un solde Wise (colonnes en anglais) ; un fichier d'une autre forme est refusé en
nommant les colonnes qui manquent, et un relevé dont le solde courant ne se suit pas est refusé.

## Notes de développement

- Format relevé sur les relevés 2024 du responsable de l'epic (notes sans donnée personnelle, `kesh-gate-logs/epic-39-40-kiss/`) :
  aucune rupture du solde courant (ni dans l'ordre des horodatages, ni dans l'ordre inverse du fichier), frais
  toujours à part, dans la devise source pour une conversion, et égaux au `Total fees` de leur ligne ; `Total fees`
  n'est donc jamais compté (39-11b). Aucun montant à séparateur de milliers relevé.
- Ancienne 39-3 (relevé JSON par l'API, `superseded`) : seule l'idée du contrôle du solde courant survit, ici.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 3 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11a-lecteur-csv-wise-P1.md`). HIGH : montants par
  `parse_amount` « rendu `pub(crate)` par 38-11a », contraire à 38-11a validée (forme fermée, `parse_amount` privé et
  dangereux pour la virgule) → forme fermée partagée. MEDIUM : tri sur `Date Time` en texte (`JJ-MM-AAAA` ne se trie
  pas) → horodatage lu ; fixtures sans exercice d'accueil en E2E → dates de 2020 ; BOM, numéros de ligne et type
  d'erreur fermé non définis → repris de 38-11a. LOW : U+202F des descriptions, lignes citées de `lib.rs`
  (`MAX_CSV_LINE_ERRORS` défini à `error.rs:49`), catalogue des fixtures.
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW, remédié (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-11a-lecteur-csv-wise-P2.md`). Remédiation P1 relue au code d'`origin/main`
  (`fd04099f`) et aux relevés 2024 en lecture seule (agrégats : montants et cours conformes aux formes fermées, identifiants
  `[A-Z0-9_-]`, aucun doublon d'identifiant par fichier, `Date` = date de `Date Time`, solde courant continu dans l'ordre des
  horodatages, aucun horodatage double) : rien à reprendre. LOW : tri sur `Date Time` quand une seule ligne en manque,
  règle écrite (ordre inverse du fichier seul).

## Dev Agent Record

### Implementation Plan

- Fusion de `epic/38-paypal` (commit `303bbc2c`) : conflits sur `sprint-status.yaml` (les deux blocs gardés : bloc 38 de
  la branche 38, bloc 39 de la nôtre), fichiers générés (`migrations.sha384`, schéma de test, 96 migrations recomptées
  par `ls`), `migrations_upgrade_path.rs`, `audit_route_registry.rs`, `api-external.md`, manuel d'administration,
  `migrations-idempotence-audit.md` : pris côté 38 (aucune migration ici, aucune de ces pièces touchée par la 39-11a).
- `csv/by_name.rs` : aides de 38-11a réutilisées telles quelles (`ColumnsByName`, `parse_dot_decimal_amount`) ; **extraits**
  de `paypal.rs` pour être partagés : `read_records` (boucle de lecture des enregistrements) et, dans `error.rs`,
  `truncate_line_error_value` ; `paypal.rs` les appelle (assertions de ses tests inchangées).
- `wise.rs` : `parse_wise_csv`, erreurs fermées, tri chronologique, continuité du solde ; tous les calculs de contrôle
  en `checked_*` (débordement : erreur de ligne `OVERFLOW`, consigne de l'orchestrateur du 2026-11-20).

### Debug Log

- Premier gate complet : « unresolved import kesh_import::wise » à `cargo clippy --workspace` : cible cargo partagée
  entre worktrees (artefact d'une autre branche pris pour frais). Rejoué sur `kesh-targets/kesh-wt39`.
- Deux sondes négatives survivaient (signe d'un crédit, tri sur l'horodatage lu) : deux tests ajoutés, elles rougissent.
- Le crate `csv` date un enregistrement qui suit une ligne vide **au milieu** du fichier du numéro de cette ligne vide
  (limite documentée à `read_records`, partagée avec PayPal ; sans effet sur une ligne vide finale) : à tracer en issue.
- Gate complet sur la cible propre : 163 échecs, **tous** « failed to apply migrations » (`No more room in record file`) :
  le tmpfs de `kesh-mariadb-dev2` est plein (8,0 Go, 171 bases `_sqlx_test_*` laissées). Le nettoyage a été refusé par le
  classificateur d'autorisations ; aucun conteneur ni base touché.

### Completion Notes

- Tests ajoutés, de `303bbc2c` à `HEAD` (recomptés par `grep -c '#\[test\]'`) : `tests/wise_csv.rs` 28 (dont 1 `#[ignore]`,
  lecture des relevés réels), `csv/by_name.rs` +2 ; ciblé `cargo test -p kesh-import` sur la cible propre : 138 + 28 passés
  (27 + 1 ignoré pour `wise_csv`), 0 échec.
- Relevés réels 2024 du responsable de l'epic lus en local (`KESH_WISE_REAL_DIR`, test ignoré) : trois fichiers lus entièrement
  (64 + 68 + 129 lignes, nombre de lignes lues = nombre de lignes de données), solde courant continu ; aucune valeur copiée.
- Sondes négatives (mutations temporaires, `git checkout` après) : continuité du solde, signe d'un crédit, tri, une seule
  devise, identifiant unique : chacune rougit un test (journal `kesh-gate-logs/s39-11a/gate-1.console`/`gate.console`).
  La sonde du débordement n'a pas été rejouée.
- `cargo publish --dry-run -p kesh-import` vert (avant le correctif `checked_*`, non rejoué).
- **Gate backend complet : NON obtenu au dernier commit de code.** Essai sur la cible partagée (avant `checked_*`) :
  6247/6248, l'échec étant un test mal écrit de la story (corrigé) et une instabilité d'environnement
  (`admin_auto_backup_e2e::disque_sature_pendant_l_ecriture`, passée au second essai). Essai sur la cible propre au dernier
  commit : disque du MariaDB plein, 163 échecs de création de base, aucun autre. À rejouer après nettoyage des bases
  `_sqlx_test_*` de `kesh-mariadb-dev2` (journaux `gate-backend*.log`).
- Non exécutés : frontend, E2E complet, `scripts/gate-ci.sh` (la story ne touche que `kesh-import`).
- Point ouvert : `detect_encoding` n'examine que 1024 octets (cf. 38-11a) ; la limite des lignes vides ci-dessus.
- Prompt de la revue P1 : `kesh-gate-logs/s39-11a/revue-p1-prompt.md`.

### File List

- `crates/kesh-import/src/wise.rs` (neuf), `src/csv/by_name.rs`, `src/csv/mod.rs`, `src/error.rs`, `src/paypal.rs`, `src/lib.rs`
- `crates/kesh-import/tests/wise_csv.rs` (neuf), `tests/fixtures/wise-releve-{chf,eur,usd}-fictif.csv` (neufs), `tests/fixtures/README.md`
- `_bmad-output/implementation-artifacts/39-11a-lecteur-csv-wise.md`, `sprint-status.yaml`

## Change Log (développement)

- 2026-11-20 — développement : lecteur Wise, extraction de `read_records` et de la troncature, fixtures fictives, 28 tests.
- 2026-11-20 — remédiation de la revue P1 (`kesh-gate-logs/s39-11a/revue-p1-rapport.md` : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 9 LOW) :
  - M1 : `check_unique_ids` linéaire (`HashMap` identifiant -> ligne, même message, premier doublon de l'ordre du fichier, toutes ses
    lignes) ; tests `first_duplicate_in_file_order_is_cited_with_all_its_lines` et `a_large_file_is_checked_in_linear_time`
    (100 000 lignes synthétiques, deux lectures en 4,3 s au total, borne 20 s ; la sonde n'a pas été rejouée contre la version
    quadratique : estimation de son coût, non mesure).
  - L1 : `epic/38-paypal` reprise (borne de 15 chiffres, `ff5b690e`), sans conflit ; `extreme_amounts_are_refused_without_panicking`
    attend désormais deux `AMOUNT` ; le chemin `OVERFLOW` du chaînage garde un test unitaire (`wise.rs`,
    `running_balance_overflow_is_a_line_error`) ; le débordement de `opening_balance` (`checked_sub`, inline) n'a pas de preuve : inatteignable par fichier.
  - L3 : `Date Time` strict — `JJ-MM-AAAA`, exactement trois décimales, `:60` refusé, forme ISO refusée (les trois relevés réels :
    uniquement `JJ-MM-AAAA HH:MM:SS.fff`, formes comptées sans valeur). L4 : cas ajoutés (heure sur un chiffre, décimales non chiffres, deux ou
    quatre décimales, `:60`, ISO, devise `C1F`). L5 : test du tri inexpugnable retiré, son commentaire versé à l'autre. L6 : le
    test `#[ignore]` échoue sans variable ou sans fichier lu, imprime `e.code()` et le nombre de fichiers (3 lus : 64, 68, 129 lignes).
    L7 : test d'un fichier à toutes cellules guillemetées avec une virgule dans une description. L9 : `read_records` et
    `RawRecords` en `pub(crate)` ; fiche alignée (`flexible(true)`, emplacement des montants, tests unitaires).
  - L2 et L8 : à ouvrir en issues (`kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`, nos 12 et 13), sans code.
  - Gate ciblé (cible propre `kesh-wt39`) : fmt `--check` vert, clippy workspace `-D warnings` vert, `cargo test -p kesh-import` vert
    (lib 140, `wise_csv` 29 passés + 1 ignoré, `paypal_csv` 11, doctests 0), `cargo publish --dry-run -p kesh-import` vert (journaux `g4-*.log`).
    Le gate backend complet du 18:25 (`gate-backend.log`) est **non joué** (disque de `kesh-mariadb-dev2` plein, arrêté par l'orchestrateur) ;
    la fusion ne touche que `kesh-import` et des fiches.
- 2026-11-20 — revue de code P2 (Claude Sonnet 5.5, contexte frais ; `kesh-gate-logs/s39-11a/revue-p2-rapport.md`) : 0 CRITICAL, 0 HIGH,
  0 MEDIUM, 4 LOW. Trend des revues de code : P1 (Opus) 0/0/1 MEDIUM/9 LOW -> P2 (Sonnet) 0/0/0/4 LOW ; boucle close (aucun finding
  au-dessus de LOW). Remédiation de la P2, tests et fiches seulement (aucune ligne de production) :
  - L1 : `dates_are_strict` reçoit `"15-03-2020 08:00:00x"` et `"15-03-2020 +8:00:00"` ; mutations sur copie hors dépôt (voir Dev Agent Record).
  - L3 : forme du montant de la fiche (AC-4) alignée sur `csv/amount.rs` (borne de 15 chiffres, fusion `ff5b690e`).
  - L2 : `check_single_currency` quadratique en nombre de devises distinctes (`wise.rs:507`) : dette tracée (voir § Dette ci-dessous).
  - L4 : borne murale du test de volume (20 s pour ~4 s mesurées) : signalé par la revue, sans remédiation.

### Dette (Change Log)

- `check_single_currency` quadratique en nombre de devises distinctes (`wise.rs`) : `technical-debt`, P4, propriétaire : le responsable de l'epic ;
  remédiation par la story de solde de l'Epic 39, `39-13-solde-dette` (`backlog` au registre, sur le modèle de `40-9-solde-dette`).
  Entrée au registre des issues à ouvrir : `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`.
- Restent en issue (même fichier) : L2 de la P1 (numéro de ligne après une ligne vide, no 12) et L8 de la P1 (plafond de
  100 erreurs et ordre de collecte, no 13).

### Gate de clôture

- Gate backend complet au dernier commit de code `f3530162` (fiche `75d042a0`) : `scripts/test-fast.sh --ci`, **6254 passés, 15 ignorés, 875 s**,
  cible propre `kesh-targets/kesh-wt39`, base `kesh-mariadb-dev2` (journal `kesh-gate-logs/s39-11a/gate-backend.log`, `gate2.console` rc=0).
  Périmètre : workspace backend complet ; ni frontend ni E2E (la story ne touche que `kesh-import`).
  Ce gate remplace la mention « NON obtenu » du Dev Agent Record ci-dessus (relevé daté du premier essai, laissé tel quel).
- Les commits de remédiation P2 ne touchent que `tests/wise_csv.rs` et des fiches : gate ciblé (fmt `--check`, clippy workspace
  `-D warnings`, `cargo test -p kesh-import`, `cargo publish --dry-run -p kesh-import`) ; le gate complet ci-dessus reste celui du
  dernier commit de code (règle de `CLAUDE.md`).
- Modèles : validation P1 Opus 5.5, P2 Sonnet 5.5 ; revue de code P1 Opus, P2 Sonnet.
- Mutations de la remédiation L1 (copie hors dépôt, `dates_are_strict` seul) : base verte ; `b.len() == 8` -> `>= 8` : ROUGE ;
  retrait du contrôle des six chiffres ASCII : ROUGE ; chacune rougit grâce aux deux cas ajoutés.
- Gate ciblé de la remédiation P2 (cible `kesh-targets/kesh-wt39`, journal `kesh-gate-logs/s39-11a/g5-cible.log`, relevé après le commit
  de remédiation) : fmt `--check` vert, clippy workspace `-D warnings` vert, `cargo test -p kesh-import` vert (lib 140, `wise_csv` 29 passés
  et 1 ignoré, `paypal_csv` 11, 0 échec), `cargo publish --dry-run -p kesh-import` vert (« aborting upload due to dry run »).
