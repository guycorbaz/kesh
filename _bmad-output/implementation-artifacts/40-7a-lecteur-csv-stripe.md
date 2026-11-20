# Story 40-7a : lecteur du CSV Stripe « Itemized payout reconciliation » (`kesh-import`)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, numéros de ligne relevés sur `origin/main` au commit `fd04099f`
(repris dans la branche par la fusion `7dcb54ab`). Epic 40, **phase 1 KISS** (décision de Beat du 2026-11-20, § *Phase 1
KISS* du fichier d'epic). **Dépend de** : 38-11a (lecteur du CSV PayPal, même crate : `parse_amount` passé `pub(crate)`,
patron des erreurs de ligne). **Suivie par** : 40-7b (import), 40-7c (rapprochement), 40-8a (le client de l'API rend les
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
| `FR-40-7a-4` | Refuser une ligne incohérente (net ≠ brut − frais, montant ou date illisible, devise invalide, versement sans identifiant, versement à deux devises), en nommant la ligne et le champ. | AC-4, AC-5 |
| `NFR-40-7a-1` | Fonction pure, sans I/O, dans `kesh-import`, qui reste publiable sans dépendance interne. | AC-6 |
| `NFR-40-7a-2` | Aucune donnée réelle dans le dépôt : la fixture est fictive. | AC-7 |

## Critères d'acceptation

1. **AC-1 — Entrée et décodage.** `kesh_import::stripe::parse_stripe_payout_csv(bytes: &[u8]) -> Result<StripePayoutCsv,
   StripeCsvError>` (module neuf `crates/kesh-import/src/stripe.rs`, réexporté par `lib.rs`, à côté des réexports
   `lib.rs:45-55`). Encodage par `detect_encoding` puis `decode_bytes` (`crates/kesh-import/src/csv/encoding.rs:48`,
   `:107`), comme le lecteur PayPal (38-11a AC-1) : BOM retiré s'il y en a un (le rapport de 2024 n'en porte pas),
   virgule, guillemets doubles, crate `csv` déjà en dépendance. Fichier vide ou sans ligne de données →
   `StripeCsvError::Empty`.
2. **AC-2 — En-tête par noms.** Noms comparés après `trim`, sans casse. **Requises** : `automatic_payout_id`,
   `automatic_payout_effective_at`, `balance_transaction_id`, `created` **ou** `created_utc`, `currency`, `gross`,
   `fee`, `net`, `reporting_category`, et **au moins une** de `source_id`, `charge_id`. **Facultatives** :
   `available_on` / `available_on_utc`, `description`, `customer_facing_amount`, `customer_facing_currency` (lues,
   gardées, non contrôlées). Toute autre colonne est ignorée. Manquantes → `StripeCsvError::MissingColumns(Vec<String>)`,
   toutes listées ; l'absence des deux colonnes d'objet lié s'y nomme `source_id|charge_id` (le message de 40-7b dit :
   « dans Stripe, Rapports → Rapprochement des versements → Détaillé, ajoutez la colonne *source_id* »). Deux colonnes de
   même nom, ou `created` et `created_utc` ensemble → `DuplicateColumn(nom)`. *(Le rapport de 2024 de Beat, exporté du
   tableau de bord avec ses colonnes par défaut d'alors, n'a ni `source_id` ni `charge_id` : il serait refusé ; la doc
   publique de Stripe met les deux parmi les colonnes par défaut de `payout_reconciliation.itemized.1`.)*
3. **AC-3 — Ligne rendue.** `StripePayoutRow { line: usize, payout_id: String, payout_date: NaiveDate,
   balance_transaction_id: String, date: NaiveDate, available_on: Option<NaiveDate>, currency: String, gross: Decimal,
   fee: Decimal, net: Decimal, category: String, description: Option<String>, source_id: Option<String>, charge_id:
   Option<String> }`. Cellule vide → `None`. `currency` en majuscules, trois lettres ASCII. `balance_transaction_id` et
   `payout_id` non vides, `[A-Za-z0-9_]` seulement après `trim` (40-7b s'en sert de clé de doublon et y accole `:FEE` :
   un identifiant Stripe n'a jamais de `:`). `category` en minuscules, rendue telle quelle (le classement est l'affaire
   de 40-7c). `StripePayoutCsv { rows: Vec<StripePayoutRow> }`. Le lecteur **ne filtre rien**.
4. **AC-4 — Dates.** Format Stripe `AAAA-MM-JJ HH:MM:SS` (et `AAAA-MM-JJ` seul) ; seule la date est gardée, **sans
   conversion de fuseau** : c'est celle du fuseau choisi à l'export (limite écrite au manuel par 40-7d, comme pour
   PayPal). Année hors `ANNEE_MIN..=ANNEE_MAX` (`crates/kesh-import/src/lib.rs:35-42`) → erreur de ligne.
5. **AC-5 — Montants et cohérence.** `gross`, `fee`, `net` lus par `parse_amount(raw, '.')`
   (`crates/kesh-import/src/csv/parser.rs:816`, passé `pub(crate)` par 38-11a ; si 40-7a est développée avant, c'est elle
   qui le passe) ; deux décimales au plus. Le rapport se demande avec le point décimal (paramètre `decimal_separator` de
   Stripe) : une virgule décimale est refusée (`AMOUNT_UNREADABLE`). Contrôles par ligne, chacun une erreur nommée
   (`StripeLineError { line, field, code, value }`, valeur tronquée à `MAX_CSV_LINE_ERROR_VALUE_CHARS`) :
   `net ≠ gross − fee` → `NET_MISMATCH` (convention Stripe : frais **positifs** quand ils sont prélevés, négatifs quand
   Stripe les rend ; ce n'est pas celle de PayPal, où net = brut + frais) ; un même `payout_id` sur deux devises →
   `PAYOUT_CURRENCY_MIXED` (sur la seconde ligne fautive). Agrégation sous `MAX_CSV_LINE_ERRORS` (`lib.rs:51-54`) dans
   `StripeCsvError::Lines(Vec<StripeLineError>)` ; une erreur refuse le fichier entier.
6. **AC-6 — DRY et pureté.** Les briques communes avec le lecteur PayPal (lecture d'en-tête par noms avec
   requises / facultatives et refus listé, agrégation plafonnée des erreurs de ligne, troncature des valeurs) ne sont
   **pas recopiées** : si 38-11a les a écrites dans `paypal.rs`, 40-7a les déplace dans un module commun
   (`crates/kesh-import/src/csv/columns.rs` ou équivalent) et `paypal.rs` les appelle — tests de 38-11a inchangés en
   assertions. Aucune dépendance neuve, aucune référence à un autre crate du workspace ; `cargo publish --dry-run -p
   kesh-import` vert.
7. **AC-7 — Fixture fictive.** `crates/kesh-import/tests/fixtures/stripe-itemized-payout-fictif.csv` : colonnes par
   défaut de `payout_reconciliation.itemized.1` (dont `source_id`, `charge_id`), **aucune donnée réelle** (identifiants
   `txn_TEST…`, `po_TEST…`, `ch_TEST…`, `re_TEST…` inventés, descriptions inventées), trois devises (`chf`, `eur`,
   `usd`), au moins quatre versements, et au moins : des `charge` avec frais, une `fee` seule (brut négatif), un
   `refund` (frais négatif), un `payout_reversal`, une charge dont la vente n'existe pas dans le jeu de 40-7c, une vente
   sans charge dans ce jeu (décrite dans 40-7c). Une variante `stripe-itemized-payout-sans-source-fictif.csv` reprend
   l'en-tête du tableau de bord de 2024 (`created`, `available_on`, sans `source_id` ni `charge_id`) pour le refus.
   Elles servent aussi à 40-7b, 40-7c, 40-7d (E2E) et 40-8a (même lignes attendues).

## Tâches

- [ ] T1 — Module commun d'en-tête et d'erreurs, déplacé de `paypal.rs` s'il existe (AC-6).
- [ ] T2 — `stripe.rs` : types, en-tête, lignes, dates (AC-1 à AC-4).
- [ ] T3 — Cohérence et agrégation (AC-5).
- [ ] T4 — Fixtures fictives et tests ; `cargo publish --dry-run -p kesh-import` (AC-6, AC-7).

## Stratégie de tests

Unitaires dans `stripe.rs` ; intégration `crates/kesh-import/tests/stripe_csv.rs` sur les fixtures : nombre de lignes et
valeurs (brut, frais, net, catégorie, objet lié) ; colonnes dans un autre ordre ; `created_utc` au lieu de `created` ;
fixture sans `source_id` → `MissingColumns(["source_id|charge_id"])` ; colonne en double ; `NET_MISMATCH` ;
`PAYOUT_CURRENCY_MIXED` ; virgule décimale ; devise `eu` ; identifiant avec `:` ; plafond d'erreurs. Tests du lecteur
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
- Remplace, pour la phase 1, la lecture de l'API de l'ancienne 40-1 (`superseded`) ; l'API revient en 40-8a et rend les
  mêmes `StripePayoutRow`.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
