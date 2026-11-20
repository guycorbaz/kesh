# Story 38-11a : lecteur du CSV PayPal « Balance affecting transactions » (`kesh-import`)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`, numéros de ligne relevés après la fusion de `origin/main`
(v0.19.0 et suivants, commit de fusion `43ceb541`). Epic 38, **phase 1 KISS** (décision de Beat du 2026-11-20,
§ *Phase 1 KISS* du fichier d'epic). **Dépend de** : rien. **Suivie par** : 38-11b (import), 38-11c2 (classement).*

## Story

En tant que **Comptable**, je veux que Kesh lise le CSV « Balance affecting transactions » qu'exporte PayPal pour un
exercice, ligne par ligne, avec le brut, les frais, le net, le type, le sens, le nom, l'e-mail et les identifiants de
transaction, afin que l'import (38-11b) n'ait plus qu'à ranger ces lignes dans les comptes bancaires de chaque devise.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-11a-1` | Lire le CSV PayPal par les **noms** de colonnes (anglais), quel que soit leur ordre ; les colonnes requises manquantes sont toutes nommées dans le refus. | AC-1, AC-2 |
| `FR-38-11a-2` | Rendre chaque ligne avec date, nom, type, statut, devise, brut, frais, net, e-mails, identifiant, identifiant lié, sens, solde, titre, objet, note. | AC-3 |
| `FR-38-11a-3` | Refuser une ligne incohérente (net ≠ brut + frais, signe contraire au sens, montant ou date illisible, montant à virgule non milliers), en nommant la ligne et le champ. | AC-4, AC-5 |
| `NFR-38-11a-1` | Fonction pure, sans I/O, dans `kesh-import`, qui reste publiable sans dépendance interne (`cargo publish --dry-run -p kesh-import`). | AC-6 |
| `NFR-38-11a-2` | Aucune donnée réelle dans le dépôt : la fixture est fictive. | AC-7 |

## Critères d'acceptation

1. **AC-1 — Entrée et décodage.** `kesh_import::paypal::parse_paypal_csv(bytes: &[u8]) -> Result<PaypalCsv,
   PaypalCsvError>` (module neuf `crates/kesh-import/src/paypal.rs`, `pub mod paypal;` et réexport à côté de
   `parse_csv`, `lib.rs:47-55`). Décodage : `detect_encoding(bytes)` rend `(encodage, bom_len)` ; `decode_bytes`
   (`crates/kesh-import/src/csv/encoding.rs:48`, `:107`) reçoit **`&bytes[bom_len..]`** (il ne retire pas le BOM :
   le CSV PayPal en porte un, ne pas le retrancher laisserait `\u{FEFF}` dans le nom de la première colonne,
   `Date`, et ferait refuser tout fichier réel) ; UTF-8 et ISO-8859-1 admis, comme au CSV bancaire. Une erreur de
   détection ou de décodage devient `PaypalCsvError::Encoding(CsvError)` (le type existe, `error.rs`, sans I/O).
   Lecture par le crate `csv` déjà en dépendance : `ReaderBuilder::new().has_headers(false)`, virgule, guillemets
   doubles (une cellule `Subject` ou `Note` peut contenir un saut de ligne entre guillemets), `flexible(false)` :
   une ligne qui n'a pas autant de cellules que l'en-tête → erreur de ligne `FIELD_COUNT`. Le **numéro de ligne**
   d'une erreur est celui du fichier (`record.position().line()`, 1 = l'en-tête), non l'indice de l'enregistrement.
   Fichier vide (`detect_encoding` : `CsvError::EmptyFile`, rendu en `PaypalCsvError::Empty`) ou en-tête seul sans
   ligne de données → `PaypalCsvError::Empty`. Les lignes entièrement vides (fin de fichier) sont sautées. Taille :
   la borne est celle de la route (38-11b), non du lecteur.
2. **AC-2 — En-tête par noms.** La première ligne est l'en-tête ; les noms se comparent après `trim` et sans casse.
   **Requises** (celles qu'un maillon aval lit : 38-11b et 38-11c2) : `Date`, `Name`, `Type`, `Status`, `Currency`,
   `Gross`, `Fee`, `Net`, `From Email Address`, `To Email Address`, `Transaction ID`, `Reference Txn ID`,
   `Balance Impact`. **Facultatives** (absentes = vides) : `Time`, `TimeZone` (affichées seulement, aucune
   conversion), `Item Title`, `Subject`, `Note`, `Balance`, `Invoice Number`. Les 41 colonnes du fichier
   réel ne gênent pas : les autres sont ignorées. Requises absentes → `PaypalCsvError::MissingColumns(Vec<String>)`,
   **toutes** listées, dans l'ordre ci-dessus (un CSV d'une autre langue ou d'un autre rapport s'arrête là, avec un
   refus qui dit pourquoi). Deux colonnes de même nom → `DuplicateColumn(nom)`.
   **Le type d'erreur est fermé** : `PaypalCsvError` = `Empty`, `Encoding(CsvError)`, `MissingColumns(Vec<String>)`,
   `DuplicateColumn(String)`, `DateOrderConflict`, `DateOrderAmbiguous`, `Lines(Vec<PaypalLineError>)` (AC-4, AC-5) ;
   chaque variante a un code stable (`PAYPAL_CSV_EMPTY`, `…_ENCODING`, `…_MISSING_COLUMNS`, `…_DUPLICATE_COLUMN`,
   `…_DATE_ORDER_CONFLICT`, `…_DATE_ORDER_AMBIGUOUS`, `…_LINES`) que 38-11b place dans `details` de `PAYPAL_CSV_INVALID`
   et que 38-11d traduit (une phrase par code, quatre locales).
3. **AC-3 — Ligne rendue.** `PaypalRow { line: usize, date: NaiveDate, time: Option<String>, time_zone:
   Option<String>, name: Option<String>, kind: String, status: String, currency: String, gross: Decimal, fee:
   Decimal, net: Decimal, from_email: Option<String>, to_email: Option<String>, transaction_id: String,
   reference_txn_id: Option<String>, balance_impact: BalanceImpact, balance: Option<Decimal>, item_title:
   Option<String>, subject: Option<String>, note: Option<String>, invoice_number: Option<String> }` ; cellule vide
   (après `trim`) → `None` ; `currency` en majuscules, trois lettres ASCII (sinon erreur de ligne) ; `transaction_id` non vide,
   `[0-9A-Z]` seulement après `trim` et majuscules (sinon erreur de ligne : 38-11b s'en sert de clé de doublon et
   y accole `:FEE`). `BalanceImpact` = `Credit` | `Debit` | `Memo` (valeurs PayPal, sans casse ; autre → erreur de
   ligne). `PaypalCsv { rows: Vec<PaypalRow>, date_order: DateOrder }` avec `DateOrder` = `DayFirst` | `MonthFirst` |
   `Dotted` | `Iso` (le format décidé du fichier, que le compte rendu peut afficher). Le lecteur **ne filtre rien** : statut,
   `Memo`, doublons sont l'affaire de 38-11b.
4. **AC-4 — Dates.** Formats admis : `JJ/MM/AAAA`, `MM/JJ/AAAA`, `JJ.MM.AAAA`, `AAAA-MM-JJ` ; jour et mois sur un
   ou deux chiffres (`5/3/2025` admis), année sur quatre chiffres (sinon erreur de ligne `DATE`). Pour la barre
   oblique, l'ordre se décide **sur tout le fichier** : un premier nombre > 12 sur une ligne → jour en tête ; un
   second > 12 → mois en tête ; les deux sur le même fichier → `DateOrderConflict` ; aucun des deux →
   `DateOrderAmbiguous` (refus, jamais de supposition : un fichier d'un exercice entier, de plusieurs centaines de
   lignes, a un jour > 12 avec une quasi-certitude ; le refus dit alors d'exporter un exercice plus long, ou de
   changer le format de date dans PayPal). Deux formats différents dans un même fichier (`/` et `.`) → erreur de
   ligne `DATE`. L'heure et le fuseau sont gardés tels quels, **sans conversion** : la date est celle de l'export
   (limite écrite au manuel par 38-11d : une ligne du 31 décembre tard le soir peut tomber à l'autre exercice selon
   le fuseau choisi dans PayPal). Année hors `ANNEE_MIN..=ANNEE_MAX` (`lib.rs:35-41`, `annee_recevable`) → erreur de
   ligne.
5. **AC-5 — Montants et cohérence.** `Gross`, `Fee`, `Net`, `Balance` : forme **fermée**, une expression régulière
   (`regex` est déjà en dépendance) `^-?(\d+|\d{1,3}(,\d{3})+)(\.\d{1,2})?$` après `trim` ; la virgule n'est admise
   que comme séparateur de milliers (trois chiffres derrière), puis retirée, et la valeur lue par
   `Decimal::from_str_exact`. ⚠️ **Pas `parse_amount(raw, '.')`** (`csv/parser.rs:816`, privé) : il retire toute
   virgule sans la contrôler, si bien que `1,50` — un fichier d'un compte PayPal à virgule décimale — se lirait
   `150`, et que `gross`, `fee` et `net` restant cohérents entre eux, **aucun contrôle ci-dessous ne le verrait** :
   une erreur de facteur 100 muette. Toute autre forme (`1,50`, `1.500,00`, `1 500.00`, apostrophe, signe `+`) →
   erreur de ligne `AMOUNT` ; `parse_amount` n'est pas touché (AC-6). Contrôles par ligne, chacun une erreur
   nommée (`PaypalLineError { line, field, code, value }`, valeur tronquée à `MAX_CSV_LINE_ERROR_VALUE_CHARS`,
   `error.rs:55`) : `net ≠ gross + fee` → `NET_MISMATCH` ; `Credit` avec `gross < 0` ou `Debit` avec `gross > 0` →
   `SIGN_MISMATCH` (`Memo` : aucun contrôle de signe). **Aucun contrôle sur le signe des frais** : PayPal rend
   des frais **positifs** sur certains remboursements (brut négatif, frais rendus positifs, net = brut + frais) ;
   un `FEE_POSITIVE` aurait refusé un fichier réel. Les erreurs s'agrègent (plafond `MAX_CSV_LINE_ERRORS`,
   `error.rs:49`) dans `PaypalCsvError::Lines(Vec<PaypalLineError>)` ; une seule erreur refuse le fichier entier
   (pas d'import partiel, comme le CSV bancaire). Conséquence pour 38-11b : une ligne de frais positifs donne une
   transaction de frais **positive** (`:FEE`, `amount = fee`) ; 38-11c2 l'envoie « à examiner » ou à son classement
   des frais (sens du montant à y lire, jamais supposé négatif).
6. **AC-6 — Pureté et publication.** Aucune dépendance neuve ; aucune référence à un autre crate du workspace ;
   `cargo publish --dry-run -p kesh-import` (job `Backend (Rust)` de la CI) reste vert ; `csv/parser.rs` n'est pas
   modifié (`parse_amount` reste privé).
7. **AC-7 — Fixture fictive.** `crates/kesh-import/tests/fixtures/paypal-balance-affecting-fictif.csv` (et une ligne
   au catalogue de `crates/kesh-import/tests/fixtures/README.md`, qui dit tous ses fichiers synthétiques) : BOM UTF-8,
   virgule, guillemets, **41 colonnes** dans l'ordre PayPal — `Date`, `Time`, `TimeZone`, `Name`, `Type`, `Status`,
   `Currency`, `Gross`, `Fee`, `Net`, `From Email Address`, `To Email Address`, `Transaction ID`, `Shipping Address`,
   `Address Status`, `Item Title`, `Item ID`, `Shipping and Handling Amount`, `Insurance Amount`, `Sales Tax`,
   `Option 1 Name`, `Option 1 Value`, `Option 2 Name`, `Option 2 Value`, `Reference Txn ID`, `Invoice Number`,
   `Custom Number`, `Quantity`, `Receipt ID`, `Balance`, `Address Line 1`, `Address Line 2/District/Neighborhood`,
   `Town/City`, `State/Province/Region/County/Territory/Prefecture/Republic`, `Zip/Postal Code`, `Country`,
   `Contact Phone Number`, `Subject`, `Note`, `Country Code`, `Balance Impact` (le fichier 2024 de Beat, supprimé, ne
   se relit plus : les **vingt** noms que ses notes relèvent — dont ceux d'AC-2 — et leur ordre font foi, les
   vingt et un autres sont des remplissages dont le nom ne sert à aucun test ; l'intégrateur le dit au Change Log
   s'il en corrige un). **Aucune donnée réelle** : noms inventés, e-mails `@example.invalid`, identifiants `0TEST…`
   de 17 caractères `[0-9A-Z]`. **Toutes les dates en 2020** (`JJ/MM/AAAA`, dont au moins un jour > 12 par
   fichier) : c'est l'exercice du seed `with-company` des E2E (ouvert au 01.01.2020), si bien que l'E2E de 38-11d
   importe la fixture sans créer d'exercice. Trois devises (EUR, USD, CHF) et au moins : un encaissement *Website
   Payment* avec frais, un *Subscription Payment*, un achat *General Payment* (débit, frais négatifs), une paire
   *User Initiated Currency Conversion* (débit EUR, crédit USD, même `Reference Txn ID`), un *General Withdrawal*, une
   paire *Payment Review Hold* / *Payment Review Release*, un *Payment Refund* **à frais positifs**, une ligne
   `Pending`, une ligne `Denied`, une ligne `Memo`, un montant à milliers (`1,234.56`), une cellule `Note` sur deux
   lignes entre guillemets. Elle sert aussi à 38-11b, 38-11c2 et à l'E2E de 38-11d.

## Tâches

- [ ] T1 — `paypal.rs` : types, en-tête par noms, lecture des lignes (AC-1 à AC-3).
- [ ] T2 — Dates à ordre décidé sur le fichier (AC-4).
- [ ] T3 — Montants par forme fermée, contrôles de cohérence, agrégation (AC-5).
- [ ] T4 — Fixture fictive et tests (AC-7) ; `cargo publish --dry-run -p kesh-import` (AC-6).

## Stratégie de tests

Tests unitaires dans `paypal.rs` et un test d'intégration `crates/kesh-import/tests/paypal_csv.rs` sur la fixture :
nombre de lignes et valeurs de chacune (brut, frais, net, sens, identifiant lié) ; colonnes dans un autre ordre ;
colonne requise manquante (toutes nommées) ; doublon de colonne ; BOM retiré (nom de la première colonne exact) ; ISO-8859-1 ;
ligne à nombre de cellules faux ; note sur deux lignes (numéro de ligne juste après elle) ; BOM absent ; dates `JJ/MM` et `MM/JJ` décidées,
conflit, ambiguïté ; `NET_MISMATCH`, `SIGN_MISMATCH`, remboursement à frais positifs **accepté**, montants `1,234.56` lu et `1,50`, `1.500,00`
refusés (`AMOUNT`), devise `EU`, identifiant avec `:` ; plafond d'erreurs. Gate ciblé `binary(paypal_csv)` et les tests de `kesh-import` ; gate complet au push.

## Modules touchés (1)

`kesh-import` seul.

## Note de version

*Ajouté* — Kesh lit le CSV « Balance affecting transactions » de PayPal (colonnes en anglais) ; un fichier d'une autre
langue ou d'un autre rapport est refusé en nommant les colonnes qui manquent.

## Notes de développement

- Reprise de l'ancienne fiche 38-7 (client PayPal, `deferred`) : seule l'idée du classement par type survit, en 38-11c2.
- Format relevé sur le fichier 2024 de Beat (notes sans donnée personnelle,
  `kesh-gate-logs/epic-38-kiss/paypal-2024-comprendre.md`) : 41 colonnes, UTF-8 avec BOM ; statuts `Completed`,
  `Pending`, `Denied`.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Sonnet 5.5) : 0 CRITICAL, 2 HIGH, 4 MEDIUM, 3 LOW, tous remédiés. HIGH : `FEE_POSITIVE`
  refusait les remboursements à frais positifs ; `parse_amount` lit `1,50` comme `150`, erreur de facteur 100 que
  les contrôles de cohérence ne voient pas (montants à forme fermée). MEDIUM : BOM non retranché des octets passés à
  `decode_bytes` ; types d'erreur non définis (`DateOrder`, `PaypalCsvError`) ; 21 colonnes de la fixture jamais
  nommées ; dates de la fixture sans exercice d'accueil en E2E (2020). LOW : numéro de ligne d'une cellule multiligne,
  `Time`/`TimeZone` requises sans usage, `MAX_CSV_LINE_ERRORS` cité à `lib.rs` (défini `error.rs:49`). Rapport :
  `kesh-gate-logs/epic-38-kiss/validation/38-11a-lecteur-csv-paypal-P1.md`.
- 2026-11-20 — validation P4 (Claude Sonnet 5.5, contexte frais) : renvois « 38-11c » nus désignés 38-11c1 ou 38-11c2 après le découpage ; aucun autre changement.
