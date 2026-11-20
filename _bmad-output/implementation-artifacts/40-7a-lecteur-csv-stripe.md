# Story 40-7a : lecteur du CSV Stripe « Itemized payout reconciliation » (`kesh-import`)

Status: review

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, numéros de ligne relevés sur `origin/main` au commit `fd04099f`
(repris dans la branche par la fusion `7dcb54ab`). Epic 40, **phase 1 KISS** (décision de Beat du 2026-11-20, § *Phase 1
KISS* du fichier d'epic). **Dépend de** : 38-11a (lecteur du CSV PayPal, même crate : forme fermée des montants, patron des erreurs de
ligne, décodage et numéros de ligne) ; 39-11a partage la même aide d'en-tête (module `csv/by_name.rs`). **Suivie par** : 40-7b (import), 40-7c (rapprochement), 40-8a (le client de l'API rend les
mêmes lignes).*

## Story

En tant que **Comptable**, je veux que Kesh lise le rapport CSV « Itemized payout reconciliation » que Stripe exporte
pour une période, ligne par ligne — versement, transaction de solde, catégorie, devise, brut, frais, net, objet Stripe
lié —, afin que l'import (40-7b) n'ait plus qu'à ranger ces lignes dans le compte Stripe de chaque devise.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-7a-1` | Lire le CSV par les **noms** de colonnes, quel que soit leur ordre, sous les deux graphies de Stripe (rapport du tableau de bord à fuseau choisi, rapport de l'API en UTC) ; les colonnes requises manquantes sont toutes nommées dans le refus. | AC-1, AC-2 |
| `FR-40-7a-2` | Exiger l'identifiant de l'objet lié (`source_id` ou `charge_id`) : sans lui, aucune charge ne peut se rattacher à sa vente ; le refus dit comment ajouter la colonne dans Stripe. | AC-2 |
| `FR-40-7a-3` | Rendre chaque ligne avec versement, date du versement, transaction de solde, date, devise, brut, frais, net, catégorie, description, objet lié, charge liée. | AC-3 |
| `FR-40-7a-4` | Refuser une ligne incohérente (net ≠ brut − frais, montant ou date illisible, **montant à virgule**, devise invalide, identifiant invalide ou vide, versement à deux devises), en nommant la ligne et le champ. | AC-4, AC-5 |
| `NFR-40-7a-1` | Fonction pure, sans I/O, dans `kesh-import`, qui reste publiable sans dépendance interne. | AC-6 |
| `NFR-40-7a-2` | Aucune donnée réelle dans le dépôt : la fixture est fictive. | AC-7 |

## Critères d'acceptation

1. **AC-1 — Entrée et décodage.** `kesh_import::stripe::parse_stripe_payout_csv(bytes: &[u8]) -> Result<StripePayoutCsv,
   StripeCsvError>` (module neuf `crates/kesh-import/src/stripe.rs`, `pub mod stripe;` et réexport à côté des réexports
   `lib.rs:45-55`). Décodage, comme 38-11a AC-1 : `detect_encoding(bytes)` rend `(encodage, bom_len)` ;
   `decode_bytes` (`crates/kesh-import/src/csv/encoding.rs:48`, `:107`) reçoit **`&bytes[bom_len..]`** (il ne retire pas
   le BOM ; le rapport de 2024 n'en porte pas, mais un BOM est admis) ; UTF-8 et ISO-8859-1. Erreur de détection ou de
   décodage → `StripeCsvError::Encoding(CsvError)`. Lecture par le crate `csv` déjà en dépendance :
   `ReaderBuilder::new().has_headers(false)`, virgule, guillemets doubles, `flexible(false)` *(écart déclaré : le code use de `flexible(true)` et d'un contrôle
   explicite, même effet, voir Dev Notes)* (une ligne qui n'a pas
   autant de cellules que l'en-tête → erreur de ligne `FIELD_COUNT`) ; **numéro de ligne** = celui du fichier
   (`record.position().line()`, 1 = l'en-tête), sauf après une ligne vide au milieu du fichier (§ *Limite connue*, amendé
   le 2026-11-20 après la revue P2) ; lignes entièrement vides sautées. Fichier vide (`CsvError::EmptyFile`,
   rendu en `Empty`) ou sans ligne de données → `StripeCsvError::Empty`. Taille : la borne est celle de la route (40-7b).
   **Type d'erreur fermé** : `StripeCsvError` = `Empty`, `Encoding(CsvError)`, `MissingColumns(Vec<String>)`,
   `DuplicateColumn(String)`, `Lines(Vec<StripeLineError>)` (AC-5), chacun avec un code stable (`STRIPE_CSV_EMPTY`,
   `…_ENCODING`, `…_MISSING_COLUMNS`, `…_DUPLICATE_COLUMN`, `…_LINES`) que 40-7b place dans `details` de
   `STRIPE_CSV_INVALID` et que 40-7d traduit (une phrase par code, quatre locales).
2. **AC-2 — En-tête par noms.** Noms comparés après `trim`, sans casse. **Requises** : `automatic_payout_id`,
   `automatic_payout_effective_at`, `balance_transaction_id`, `created` **ou** `created_utc`, `currency`, `gross`,
   `fee`, `net`, `reporting_category`, et **au moins une** de `source_id`, `charge_id`. **Facultatives** :
   `available_on` / `available_on_utc`, `description`, `customer_facing_amount`, `customer_facing_currency` (déclarées,
   non contrôlées ; **non gardées** : `StripePayoutRow` suit l'AC-3, qui ne les porte pas — écart L2 de la P1). Toute autre colonne est ignorée. Manquantes → `StripeCsvError::MissingColumns(Vec<String>)`,
   toutes listées dans l'ordre ci-dessus ; l'absence des deux colonnes d'objet lié s'y nomme `source_id|charge_id` (le message de 40-7b dit :
   « dans Stripe, Rapports → Rapprochement des versements → Détaillé, ajoutez la colonne *source_id* »). Deux colonnes de
   même nom, ou `created` et `created_utc` ensemble → `DuplicateColumn(nom)`. *(Le rapport de 2024 de Beat, exporté du
   tableau de bord avec ses colonnes par défaut d'alors, n'a ni `source_id` ni `charge_id` : il serait refusé ; la doc
   publique de Stripe met les deux parmi les colonnes par défaut de `payout_reconciliation.itemized.1`.)*
3. **AC-3 — Ligne rendue.** `StripePayoutRow { line: usize, payout_id: String, payout_date: NaiveDate,
   balance_transaction_id: String, date: NaiveDate, available_on: Option<NaiveDate>, currency: String, gross: Decimal,
   fee: Decimal, net: Decimal, category: String, description: Option<String>, source_id: Option<String>, charge_id:
   Option<String> }`. Cellule vide → `None`. `currency` en majuscules, trois lettres ASCII. `balance_transaction_id` et
   `payout_id` non vides (un `automatic_payout_id` vide — rapport d'un autre type, versements manuels — est refusé,
   erreur de ligne `ID`, la phrase de 40-7d dit de prendre le rapport des versements automatiques),
   `[A-Za-z0-9_]` seulement après `trim` (40-7b s'en sert de clé de doublon et y accole `:FEE` :
   un identifiant Stripe n'a jamais de `:`). `category` en minuscules, rendue telle quelle (le classement est l'affaire
   de 40-7c). `StripePayoutCsv { rows: Vec<StripePayoutRow> }`. Le lecteur **ne filtre rien**.
4. **AC-4 — Dates.** Format Stripe `AAAA-MM-JJ HH:MM:SS` (et `AAAA-MM-JJ` seul) ; seule la date est gardée, **sans
   conversion de fuseau** : c'est celle du fuseau choisi à l'export (limite écrite au manuel par 40-7d, comme pour
   PayPal). Année hors `ANNEE_MIN..=ANNEE_MAX` (`crates/kesh-import/src/lib.rs:35-42`, `annee_recevable`) → erreur de ligne
   `DATE`. Une date d'un autre format → `DATE`.
5. **AC-5 — Montants et cohérence.** `gross`, `fee`, `net` : **forme fermée**, la fonction de 38-11a AC-5 (expression
   régulière, `Decimal::from_str_exact`), appelée **sans** séparateur de milliers : `^-?\d+(\.\d{1,2})?$` après `trim`
   (Stripe n'en écrit jamais). ⚠️ **Pas `parse_amount(raw, '.')`** (`csv/parser.rs:816`, **privé**, et 38-11a le laisse
   tel) : il retire toute virgule sans la contrôler, si bien qu'un rapport demandé à virgule décimale (`12,50`) se
   lirait `1250` ; avec des frais nuls (`fee = 0`), `gross` et `net` restent égaux et **aucun contrôle de cohérence ne
   voit** l'erreur de facteur 100. Toute autre forme (virgule, espace, apostrophe, signe `+`, trois décimales —
   monnaies à trois décimales non servies en phase 1, limite écrite au manuel) → erreur de ligne `AMOUNT`. Contrôles par
   ligne, chacun une erreur nommée (`StripeLineError { line, field, code, value }`, valeur tronquée à
   `MAX_CSV_LINE_ERROR_VALUE_CHARS`) : `net ≠ gross − fee` → `NET_MISMATCH` (convention Stripe : frais **positifs**
   quand ils sont prélevés, négatifs quand Stripe les rend — remboursement ; ce n'est pas celle de PayPal, où net =
   brut + frais ; aucun contrôle sur le signe des frais) ; un même `payout_id` sur deux devises →
   `PAYOUT_CURRENCY_MIXED` (sur la seconde ligne fautive). **Codes de ligne, liste fermée** (40-7d en traduit chacun) : `FIELD_COUNT`, `ID`, `DATE`, `CURRENCY` (devise qui n'est pas trois lettres ASCII), `AMOUNT`, `NET_MISMATCH`,
   `PAYOUT_CURRENCY_MIXED`. Agrégation sous `MAX_CSV_LINE_ERRORS` (`error.rs:49`,
   réexporté `lib.rs:51-54`) dans `StripeCsvError::Lines(Vec<StripeLineError>)` ; une erreur refuse le fichier entier.
6. **AC-6 — DRY et pureté.** 38-11a n'a pas de module commun : ses briques (lecture d'en-tête par noms avec requises /
   facultatives et refus listé, forme fermée des montants, agrégation plafonnée des erreurs de ligne, troncature des
   valeurs) vivent dans `paypal.rs`. 40-7a les **déplace** dans `crates/kesh-import/src/csv/by_name.rs` (le nom
   qu'emploie 39-11a, G10 ; si 39-11a est déjà passée, elle l'a fait : 40-7a l'utilise) et `paypal.rs` les appelle —
   tests de 38-11a **inchangés en assertions**. Aucune dépendance neuve, aucune référence à un autre crate du
   workspace ; `parse_amount` n'est pas touché ; `cargo publish --dry-run -p kesh-import` vert.
7. **AC-7 — Fixture fictive.** `crates/kesh-import/tests/fixtures/stripe-itemized-payout-fictif.csv` (et une ligne au
   catalogue de `crates/kesh-import/tests/fixtures/README.md`, qui dit tous ses fichiers synthétiques) : les onze
   colonnes du rapport de 2024 (`automatic_payout_id`, `automatic_payout_effective_at`, `balance_transaction_id`,
   `created`, `available_on`, `currency`, `gross`, `fee`, `net`, `reporting_category`, `description`) **plus** `source_id`
   et `charge_id` (celles que doit ajouter Beat à l'export ; la doc publique de `payout_reconciliation.itemized.1` les met
   parmi les colonnes par défaut), sans BOM, devises en minuscules, deux décimales, `net = gross − fee` partout.
   **Aucune donnée réelle** : identifiants `txn_TEST…`, `po_TEST…`, `ch_TEST…`, `re_TEST…` inventés, descriptions
   inventées. **Toutes les dates en 2020** (`AAAA-MM-JJ HH:MM:SS`) : c'est l'exercice du seed `with-company` des E2E
   (ouvert au 01.01.2020), si bien que les écritures de 40-7c s'y passent sans créer d'exercice. Trois devises (`chf`,
   `eur`, `usd`), au moins quatre versements, et au moins : des `charge` avec frais, une `fee` seule (brut négatif, frais
   nuls) et une `fee` à frais non nuls (cas relevé en 2024), un `refund` (frais négatif), **un versement qui ne porte
   qu'un `payout_reversal`** (cas relevé : 2 versements sur 100 en 2024), une charge dont la vente n'existe pas dans le
   jeu de 40-7c, une vente sans charge dans ce jeu (décrite dans 40-7c), et **un versement EUR dont le net égale
   l'« Argent reçu de Stripe » EUR de la fixture de 39-11a** (montant lu à T0 ; si 39-11a n'est pas fusionnée, le montant
   est écrit ici et 39-11a s'y aligne). Une variante `stripe-itemized-payout-sans-source-fictif.csv` reprend l'en-tête du
   rapport de 2024 (`created`, `available_on`, sans `source_id` ni `charge_id`) pour le refus. Elles servent aussi à
   40-7b, 40-7c, 40-7d (E2E) et 40-8a (mêmes lignes attendues).

## Tâches

- [x] T1 — Module commun `csv/by_name.rs` : **déjà écrit par 38-11a** (`by_name.rs`, `amount.rs`, fusionnés de `epic/38-paypal`) ; réutilisé, rien déplacé.
- [x] T2 — `stripe.rs` : types, en-tête, lignes, dates (AC-1 à AC-4).
- [x] T3 — Cohérence et agrégation (AC-5).
- [x] T4 — Fixtures fictives et tests ; `cargo publish --dry-run -p kesh-import` (AC-6, AC-7).

## Stratégie de tests

Unitaires dans `stripe.rs` ; intégration `crates/kesh-import/tests/stripe_csv.rs` sur les fixtures : nombre de lignes et
valeurs (brut, frais, net, catégorie, objet lié) ; colonnes dans un autre ordre ; `created_utc` au lieu de `created` ;
fixture sans `source_id` → `MissingColumns(["source_id|charge_id"])` ; colonne en double ; `NET_MISMATCH` ;
`PAYOUT_CURRENCY_MIXED` ; **virgule décimale `12,50` refusée (`AMOUNT`), jamais lue `1250`**, y compris à frais nuls ; trois
décimales refusées ; BOM retiré (nom de la première colonne exact) ; ISO-8859-1 ; ligne à nombre de cellules faux ;
`automatic_payout_id` vide ; devise `eu` ; identifiant avec `:` ; plafond d'erreurs. Tests du lecteur
PayPal relancés (AC-6). Gate ciblé `binary(stripe_csv)` + tests de `kesh-import` ; gate complet au push.

## Modules touchés (1)

`kesh-import` seul.

## Note de version

*Ajouté* — Kesh lit le rapport CSV « Itemized payout reconciliation » de Stripe ; un rapport sans la colonne de l'objet
lié (`source_id` ou `charge_id`) est refusé avec la marche à suivre pour l'ajouter.

## Notes de développement

- Format relevé sur les rapports 2024 de Beat (notes sans donnée personnelle,
  `kesh-gate-logs/epic-39-40-kiss/wise-stripe-2024-comprendre.md`, et relevé de forme du 2026-11-20 : 11 colonnes, sans
  BOM, devises en minuscules, deux décimales, `net = gross − fee` sur toutes les lignes, catégories `charge`, `fee`,
  `refund`, `payout_reversal`). Aucune valeur réelle n'est reprise ici.
- Colonnes par défaut et facultatives de `payout_reconciliation.itemized.1` : docs.stripe.com/reports/report-types/
  payout-reconciliation, lu le 2026-11-20. **Non confirmé** : la liste exacte des colonnes par défaut quand le rapport
  est téléchargé du **tableau de bord** (le fichier de 2024 en avait moins) ; la valeur des colonnes d'une ligne `fee`
  seule (`source_id` vide ?).
- ⚠️ Ce que le rapport de 2024 apprend de plus (relevé de forme, sans valeur) : 100 versements pour 100 lignes `fee`
  (frais de versement en devise alternative), aucun versement à deux devises, deux versements ne portant que des
  `payout_reversal`, `automatic_payout_effective_at` ≥ `created` partout et ≠ `available_on` une fois sur deux ;
  aucune description de charge ne porte `ch_`/`py_` (d'où `source_id`).
- Remplace, pour la phase 1, la lecture de l'API de l'ancienne 40-1 (`superseded`) ; l'API revient en 40-8a et rend les
  mêmes `StripePayoutRow`.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 2 LOW, remédiés
  (rapport `kesh-gate-logs/epic-39-40-kiss/validation/40-7a-lecteur-csv-stripe-P1.md`). HIGH : l'AC-5 lisait les montants
  par `parse_amount(raw, '.')`, que 38-11a laisse privé et qui lit `12,50` comme `1250` (invisible des contrôles à frais
  nuls) → forme fermée de 38-11a. MEDIUM : BOM non retranché des octets passés à `decode_bytes`, type d'erreur non
  fermé, numéros de ligne et `FIELD_COUNT` non repris de 38-11a ; module commun nommé `csv/by_name.rs` (39-11a), 38-11a
  n'en ayant pas écrit ; fixture sans dates de 2020 (E2E) ni versement `payout_reversal` seul ni versement EUR
  égal à la réception Wise. LOW : catalogue `fixtures/README.md` ; ordre des colonnes manquantes.
- 2026-11-20 — validation P2 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW, remédié (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/40-7a-lecteur-csv-stripe-P2.md`). Remédiation P1 vérifiée au code de
  `origin/main` (`decode_bytes` sans retrait du BOM, `detect_encoding` rendant `bom_len`, `parse_amount` privé et
  laxiste, `annee_recevable`, réexports) et contre 38-11a et 39-11a (`csv/by_name.rs`, forme fermée). LOW : la liste des
  codes d'erreur de ligne n'était pas fermée (la devise invalide n'avait pas de code) → liste écrite.
- 2026-11-20 — développement (Claude Sonnet 5.5), voir Dev Agent Record. Statut `review`.

## Dev Agent Record

*Développement du 2026-11-20, branche `epic/40-stripe`, après fusion de `epic/38-paypal` (commit `9b48621d`).*

### Fusion de `epic/38-paypal`

Huit fichiers en conflit, tous résolus en gardant les deux intentions : `sprint-status.yaml` (bloc Epic 38 de la 38, blocs
39 et 40 de la branche), `docs/api-external.md`, `audit_route_registry.rs`, `migrations_upgrade_path.rs`,
`migrations.sha384`, `docs/migrations-idempotence-audit.md` et `0001_schema_squash.sql` (le côté Epic 38, qui porte la
migration `paypal_import`, la branche Stripe n'ayant aucune migration ni route nouvelle), `admin-manual.tex` (compteur de
routes de l'Epic 38). Compteurs : 96 migrations (`ls crates/kesh-db/migrations/*.sql | wc -l`), tous ceux des fichiers
repris tiennent déjà ce total. Le test-schéma et les gates complets de `kesh-db`/`kesh-api` **n'ont pas été rejoués** ici
(story `kesh-import` seule) ; ils le sont au gate complet de l'epic.

### Réalisé

- `crates/kesh-import/src/stripe.rs` (module neuf, réexporté par `lib.rs`) : `parse_stripe_payout_csv`, `StripePayoutRow`,
  `StripePayoutCsv`, `StripeCsvError` (5 variantes à code stable `STRIPE_CSV_*`), `StripeLineError(Code)` (7 codes fermés).
  Construit sur `csv::ColumnsByName` et `parse_dot_decimal_amount` de 38-11a, **sans duplication** ; le patron de
  `paypal.rs` (décodage avec `bom_len`, `flexible(true)` + contrôle du nombre de cellules, numéros de ligne,
  collecte plafonnée) est repris tel quel.
- Fixtures fictives `stripe-itemized-payout-fictif.csv` (9 lignes, 4 versements, 3 devises, 2020) et
  `stripe-itemized-payout-sans-source-fictif.csv` ; catalogue au `fixtures/README.md`.
- Tests : `tests/stripe_csv.rs` (17 tests) et 4 tests unitaires dans `stripe.rs`.

### Écarts et choix, dits

- **AC-6 / T1** : 38-11a avait déjà mis en commun `by_name.rs` et `amount.rs` ; la boucle de lecture, la collecte plafonnée des erreurs et la troncature des valeurs sont **recopiées** de `paypal.rs`, non mises en commun (corrigé à la revue P1, M2) : reclassé en dette technique, voir § *Dette technique*. `paypal.rs` n'est pas touché.
- **Montants** : `parse_dot_decimal_amount` admet la virgule des milliers ; Stripe n'en écrit pas (AC-5), donc `parse_amount`
  de `stripe.rs` refuse toute cellule qui contient une virgule avant d'appeler l'aide commune.
- **« L'une ou l'autre »** (`created`/`created_utc`, `source_id`/`charge_id`) : l'aide commune ne connaît que des
  requises à nom unique ; les quatre sont déclarées facultatives et l'exigence se contrôle dans `stripe.rs`, qui refait
  l'ordre de la liste des manquantes (`created|created_utc` après `balance_transaction_id`, `source_id|charge_id` en
  dernier). Test de l'ordre complet.
- **`FIELD_COUNT`** : `flexible(true)` et contrôle explicite, comme `paypal.rs`, plutôt que `flexible(false)` de la fiche —
  même résultat (erreur de ligne, numéro de ligne, ligne vide finale sautée), sans dépendre de ce que le lecteur `csv`
  rend après une erreur de longueur.
- **Catégorie vide** : la fiche n'a pas de code pour elle et la liste est fermée ; elle prend le code `ID`, champ
  `reporting_category`.
- **`source_id` / `charge_id`** : non contrôlés au jeu de caractères (la fiche ne le demande que pour le versement et la
  transaction de solde) ; ils sont gardés tels quels, `None` si vides.
- **Date de disponibilité** : si `available_on` et `available_on_utc` sont toutes deux présentes, `available_on` prime.
- **Versement EUR = « Argent reçu » Wise** : la fixture de 39-11a n'est pas dans cette branche ; le net EUR de
  `po_TEST0002` est écrit ici (**270.30**, tenu par un test) et 39-11a s'y alignera.
- **Mutation vérifiée** : en neutralisant le refus de la virgule dans `parse_amount`, `decimal_comma_is_refused_never_read_as_hundred_times` rougit, **mais seul le sous-cas des milliers `1,234.50`** : `12,50` reste refusé par l'expression régulière commune (`csv/amount.rs`, milliers en groupes de trois), de sorte que le refus propre à `stripe.rs` ne tient que les milliers (journal `kesh-gate-logs/s40-7a/mutation-et-reel.log`) ; code restauré.

### Rapport réel de Beat, lecture locale

Lecture seule, par un test jetable (supprimé, non commité) ; aucune valeur copiée ni citée. Les quatre CSV (toutes devises,
CHF, EUR, USD) sont **refusés tels quels** par `MissingColumns(["source_id|charge_id"])` — le refus ne nomme que cette
colonne, donc les huit autres requises et `created` y sont. Une fois la colonne `source_id` ajoutée (vide) par un test, ils
se lisent : 52, 336, 4 et 280 lignes pour 52, 336, 4 et 280 enregistrements de données (comptés par le crate `csv`,
guillemets compris), aucune erreur de ligne (net, formes, dates, versements à une devise).

### Gate

Gate ciblé `kesh-import` (clippy `-D warnings` + `cargo test -p kesh-import`) ; journal
`kesh-gate-logs/s40-7a/test-import-2.log`. Le gate complet n'avait pas été joué par le développement ; la revue P1 l'a joué au dernier commit de code (`scripts/test-fast.sh --ci` : 6241 exécutés, 6241 passés, 14 ignorés ; `cargo publish --dry-run -p kesh-import` vert ; journaux `kesh-gate-logs/s40-7a/`), y compris le test-schéma et les gates `kesh-db`/`kesh-api` de la fusion. Il se rejoue au gate complet de l'epic.

### Remédiation de la revue P1 (2026-11-20)

M1 : `gross - fee` passé en `checked_sub` (un débordement est une erreur de ligne `NET_MISMATCH`, plus une panique) ; test `extreme_amounts_never_panic_on_the_net_check`. M2 : reclassé en dette (ci-dessous). L1 : test `several_line_errors_come_out_sorted_by_line`. L2 : AC-2 alignée sur le code. L3 : `norm` remplacé par `csv::by_name::normalize` (`pub(crate)`). L4, L5 : textes ci-dessus. L6 : test `line_after_a_blank_line_takes_the_blank_line_number`, qui tient la limite des numéros de ligne (ci-dessous).

### Gate de la remédiation (2026-11-20)

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p kesh-import` (140 + 5 + 6 + 5 + 29 + 10 + 20 passés, dont `stripe_csv` 20/20 ; 1 ignoré) et `cargo publish --dry-run --allow-dirty -p kesh-import` (`--allow-dirty` : arbre non commité au moment du gate) verts ; journal `kesh-gate-logs/s40-7a/p1-remed2.log`. Mutation : `net != gross - fee` rétabli, `extreme_amounts_never_panic_on_the_net_check` rougit (panique « Subtraction overflowed » de `rust_decimal`) ; code restauré. *(Annoté le 2026-11-20, revue P2 : cette mutation n'est pas au journal `p1-remed2.log` ; elle a été rejouée par la passe P2 sur une copie hors dépôt — M1 rétabli rougit, tri retiré rougit `several_line_errors_come_out_sorted_by_line`, restauration 20/20 — `kesh-gate-logs/s40-7a/revue-p2-rapport.md`.)* Gate complet non rejoué (aucun code hors `kesh-import`).

### Remédiation de la revue P2 ciblée (2026-11-20, orchestrateur, fiche seulement)

P2 ciblée (Haiku) sur `3bcaf532` : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW (`kesh-gate-logs/s40-7a/revue-p2-rapport.md`). F1 (dette sans propriétaire nommé ni story planifiée) : propriétaire Beat, story `40-9-solde-dette` ajoutée au registre, issue à ouvrir. F2 (AC-1 contredite par le test de la ligne vide) : AC-1 amendée pour renvoyer à la limite, issue à ouvrir. F3 : plafond puis tri écrits. F4 : mutation annotée (rejouée par la P2). Aucune ligne de code touchée ; une passe ciblée P3 suit (règle de clôture). P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW — M1 (40-9 absente du fichier d'epic, plafond d'issues et dépassement de la borne de six non déclarés) remédié dans `epic-40-stripe.md` ; L1 (« ordre de collecte ») et L2 (`flexible`) corrigés ; P4 ciblée à suivre.

## Limite connue : numéro de ligne après une ligne vide

Le crate `csv` donne à un enregistrement qui suit une ligne vide au milieu du fichier le numéro de cette ligne vide : l'erreur est rapportée une ligne trop tôt par ligne vide précédente. Limite partagée avec PayPal et Wise (documentée dans `read_records`, `csv/by_name.rs`, branche `epic/39-wise`) ; le lecteur ne la contourne pas, sur décision du coordinateur. Tenue par le test ci-dessus ; les lignes vides de fin de fichier ne comptent pas. Issue à ouvrir avec l'accord de Beat (même liste que la dette ci-dessous), commune aux trois lecteurs ; le test `line_after_a_blank_line_takes_the_blank_line_number` fige ce comportement pour qu'une correction future se voie.

Plafond des erreurs de ligne : la collecte garde les cent premières erreurs **dans l'ordre de collecte** — d'abord toutes les erreurs `FIELD_COUNT` du fichier, puis les erreurs de cellule ligne par ligne —, puis les trie par ligne (`stripe.rs`, `struct Errors`) : au-delà de cent, les erreurs rendues ne sont pas forcément les premières du fichier.

## Dette technique

- **Boucle de lecture, collecte plafonnée des erreurs de ligne et troncature des valeurs, recopiées de `paypal.rs`** (`stripe.rs` : troncature de `StripeLineError::new`, `struct Errors`, décodage et lecture des enregistrements ; trois copies avec le lecteur Wise de la 39-11a). Propriétaire : Beat (contributeur des Epics 38 à 40). Remédiation planifiée : story `40-9-solde-dette` (`backlog`, `sprint-status.yaml`), extraction dans `kesh-import/src/csv/` une fois les trois lecteurs (PayPal, Wise 39-11a, Stripe) en place ; issue à ouvrir avec l'accord de Beat (`kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`).

