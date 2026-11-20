# Story 40-7a : lecteur du CSV Stripe « Itemized payout reconciliation » (`kesh-import`)

Status: drafted

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
   `ReaderBuilder::new().has_headers(false)`, virgule, guillemets doubles, `flexible(false)` (une ligne qui n'a pas
   autant de cellules que l'en-tête → erreur de ligne `FIELD_COUNT`) ; **numéro de ligne** = celui du fichier
   (`record.position().line()`, 1 = l'en-tête) ; lignes entièrement vides sautées. Fichier vide (`CsvError::EmptyFile`,
   rendu en `Empty`) ou sans ligne de données → `StripeCsvError::Empty`. Taille : la borne est celle de la route (40-7b).
   **Type d'erreur fermé** : `StripeCsvError` = `Empty`, `Encoding(CsvError)`, `MissingColumns(Vec<String>)`,
   `DuplicateColumn(String)`, `Lines(Vec<StripeLineError>)` (AC-5), chacun avec un code stable (`STRIPE_CSV_EMPTY`,
   `…_ENCODING`, `…_MISSING_COLUMNS`, `…_DUPLICATE_COLUMN`, `…_LINES`) que 40-7b place dans `details` de
   `STRIPE_CSV_INVALID` et que 40-7d traduit (une phrase par code, quatre locales).
2. **AC-2 — En-tête par noms.** Noms comparés après `trim`, sans casse. **Requises** : `automatic_payout_id`,
   `automatic_payout_effective_at`, `balance_transaction_id`, `created` **ou** `created_utc`, `currency`, `gross`,
   `fee`, `net`, `reporting_category`, et **au moins une** de `source_id`, `charge_id`. **Facultatives** :
   `available_on` / `available_on_utc`, `description`, `customer_facing_amount`, `customer_facing_currency` (lues,
   gardées, non contrôlées). Toute autre colonne est ignorée. Manquantes → `StripeCsvError::MissingColumns(Vec<String>)`,
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
   `PAYOUT_CURRENCY_MIXED` (sur la seconde ligne fautive). Agrégation sous `MAX_CSV_LINE_ERRORS` (`error.rs:49`,
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

- [ ] T1 — Module commun `csv/by_name.rs` (en-tête, forme fermée des montants, erreurs), déplacé de `paypal.rs` (AC-6).
- [ ] T2 — `stripe.rs` : types, en-tête, lignes, dates (AC-1 à AC-4).
- [ ] T3 — Cohérence et agrégation (AC-5).
- [ ] T4 — Fixtures fictives et tests ; `cargo publish --dry-run -p kesh-import` (AC-6, AC-7).

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
