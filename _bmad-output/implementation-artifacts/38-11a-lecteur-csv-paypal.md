# Story 38-11a : lecteur du CSV PayPal « Balance affecting transactions » (`kesh-import`)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`, numéros de ligne relevés après la fusion de `origin/main`
(v0.19.0 et suivants, commit de fusion `43ceb541`). Epic 38, **phase 1 KISS** (décision de Beat du 2026-11-20,
§ *Phase 1 KISS* du fichier d'epic). **Dépend de** : rien. **Suivie par** : 38-11b (import), 38-11c (classement).*

## Story

En tant que **Comptable**, je veux que Kesh lise le CSV « Balance affecting transactions » qu'exporte PayPal pour un
exercice, ligne par ligne, avec le brut, les frais, le net, le type, le sens, le nom, l'e-mail et les identifiants de
transaction, afin que l'import (38-11b) n'ait plus qu'à ranger ces lignes dans les comptes bancaires de chaque devise.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-11a-1` | Lire le CSV PayPal par les **noms** de colonnes (anglais), quel que soit leur ordre ; les colonnes requises manquantes sont toutes nommées dans le refus. | AC-1, AC-2 |
| `FR-38-11a-2` | Rendre chaque ligne avec date, nom, type, statut, devise, brut, frais, net, e-mails, identifiant, identifiant lié, sens, solde, titre, objet, note. | AC-3 |
| `FR-38-11a-3` | Refuser une ligne incohérente (net ≠ brut + frais, signe contraire au sens, montant ou date illisible), en nommant la ligne et le champ. | AC-4, AC-5 |
| `NFR-38-11a-1` | Fonction pure, sans I/O, dans `kesh-import`, qui reste publiable sans dépendance interne (`cargo publish --dry-run -p kesh-import`). | AC-6 |
| `NFR-38-11a-2` | Aucune donnée réelle dans le dépôt : la fixture est fictive. | AC-7 |

## Critères d'acceptation

1. **AC-1 — Entrée et décodage.** `kesh_import::paypal::parse_paypal_csv(bytes: &[u8]) -> Result<PaypalCsv,
   PaypalCsvError>` (module neuf `crates/kesh-import/src/paypal.rs`, réexporté par `lib.rs:47-55`). Encodage par
   `detect_encoding` puis `decode_bytes` (`crates/kesh-import/src/csv/encoding.rs:48`, `:107`) : BOM UTF-8 retiré
   (le CSV PayPal en porte un), virgule, guillemets doubles, lecture par le crate `csv` déjà en dépendance
   (`Cargo.toml`). Fichier vide ou sans ligne de données → `PaypalCsvError::Empty`. Taille : la borne est celle de la
   route (38-11b), non du lecteur.
2. **AC-2 — En-tête par noms.** La première ligne est l'en-tête ; les noms se comparent après `trim` et sans casse.
   **Requises** : `Date`, `Time`, `TimeZone`, `Name`, `Type`, `Status`, `Currency`, `Gross`, `Fee`, `Net`,
   `From Email Address`, `To Email Address`, `Transaction ID`, `Reference Txn ID`, `Balance Impact`. **Facultatives**
   (absentes = vides) : `Item Title`, `Subject`, `Note`, `Balance`, `Invoice Number`. Les 41 colonnes du fichier
   réel ne gênent pas : les autres sont ignorées. Requises absentes → `PaypalCsvError::MissingColumns(Vec<String>)`,
   **toutes** listées, dans l'ordre ci-dessus (un CSV d'une autre langue ou d'un autre rapport s'arrête là, avec un
   refus qui dit pourquoi). Deux colonnes de même nom → `DuplicateColumn(nom)`.
3. **AC-3 — Ligne rendue.** `PaypalRow { line: usize, date: NaiveDate, time: Option<String>, time_zone:
   Option<String>, name: Option<String>, kind: String, status: String, currency: String, gross: Decimal, fee:
   Decimal, net: Decimal, from_email: Option<String>, to_email: Option<String>, transaction_id: String,
   reference_txn_id: Option<String>, balance_impact: BalanceImpact, balance: Option<Decimal>, item_title:
   Option<String>, subject: Option<String>, note: Option<String>, invoice_number: Option<String> }` ; cellule vide
   → `None` ; `currency` en majuscules, trois lettres ASCII (sinon erreur de ligne) ; `transaction_id` non vide,
   `[0-9A-Z]` seulement après `trim` et majuscules (sinon erreur de ligne : 38-11b s'en sert de clé de doublon et
   y accole `:FEE`). `BalanceImpact` = `Credit` | `Debit` | `Memo` (valeurs PayPal, sans casse ; autre → erreur de
   ligne). `PaypalCsv { rows: Vec<PaypalRow>, date_order: DateOrder }`. Le lecteur **ne filtre rien** : statut,
   `Memo`, doublons sont l'affaire de 38-11b.
4. **AC-4 — Dates.** Formats admis : `JJ/MM/AAAA`, `MM/JJ/AAAA`, `JJ.MM.AAAA`, `AAAA-MM-JJ`. Pour la barre oblique,
   l'ordre se décide **sur tout le fichier** : un premier nombre > 12 sur une ligne → jour en tête ; un second > 12
   → mois en tête ; les deux sur le même fichier → `DateOrderConflict` ; aucun des deux → `DateOrderAmbiguous`
   (refus, jamais de supposition). L'heure et le fuseau sont gardés tels quels, **sans conversion** : la date est
   celle de l'export (limite écrite au manuel par 38-11d : une ligne du 31 décembre tard le soir peut tomber à
   l'autre exercice selon le fuseau choisi dans PayPal). Année hors `ANNEE_MIN..=ANNEE_MAX` (`lib.rs:35-41`) →
   erreur de ligne.
5. **AC-5 — Montants et cohérence.** `Gross`, `Fee`, `Net`, `Balance` lus par `parse_amount(raw, '.')`
   (`crates/kesh-import/src/csv/parser.rs:816`, passé `pub(crate)` — même règle que le CSV bancaire : virgule des
   milliers admise, `1.500,00` refusé comme incohérent), deux décimales au plus (sinon erreur). Contrôles par ligne,
   chacun une erreur nommée (`PaypalLineError { line, field, code, value }`, valeur tronquée à
   `MAX_CSV_LINE_ERROR_VALUE_CHARS`) : `net ≠ gross + fee` → `NET_MISMATCH` ; `Credit` avec `gross < 0` ou `Debit`
   avec `gross > 0` → `SIGN_MISMATCH` ; `fee > 0` → `FEE_POSITIVE`. Les erreurs s'agrègent (plafond
   `MAX_CSV_LINE_ERRORS`, `lib.rs:51-54`) dans `PaypalCsvError::Lines(Vec<PaypalLineError>)` ; une seule erreur
   refuse le fichier entier (pas d'import partiel, comme le CSV bancaire).
6. **AC-6 — Pureté et publication.** Aucune dépendance neuve ; aucune référence à un autre crate du workspace ;
   `cargo publish --dry-run -p kesh-import` (job `Backend (Rust)` de la CI) reste vert ; `parse_amount` garde son
   comportement (ses tests de `parser.rs` inchangés).
7. **AC-7 — Fixture fictive.** `crates/kesh-import/tests/fixtures/paypal-balance-affecting-fictif.csv` : BOM, 41
   colonnes dans l'ordre PayPal, **aucune donnée réelle** (noms inventés, e-mails `@example.invalid`, identifiants
   `0TEST…` de 17 caractères), deux devises (EUR, USD) et CHF, et au moins : un encaissement *Website Payment* avec
   frais, un *Subscription Payment*, un achat *General Payment* (débit), une paire *User Initiated Currency
   Conversion* (débit EUR, crédit USD, même `Reference Txn ID`), un *General Withdrawal*, une paire *Payment Review
   Hold* / *Payment Review Release*, un *Payment Refund*, une ligne `Pending`, une ligne `Memo`. Elle sert aussi à
   38-11b, 38-11c et à l'E2E de 38-11d.

## Tâches

- [ ] T1 — `paypal.rs` : types, en-tête par noms, lecture des lignes (AC-1 à AC-3).
- [ ] T2 — Dates à ordre décidé sur le fichier (AC-4).
- [ ] T3 — `parse_amount` en `pub(crate)`, contrôles de cohérence, agrégation (AC-5).
- [ ] T4 — Fixture fictive et tests (AC-7) ; `cargo publish --dry-run -p kesh-import` (AC-6).

## Stratégie de tests

Tests unitaires dans `paypal.rs` et un test d'intégration `crates/kesh-import/tests/paypal_csv.rs` sur la fixture :
nombre de lignes et valeurs de chacune (brut, frais, net, sens, identifiant lié) ; colonnes dans un autre ordre ;
colonne requise manquante (toutes nommées) ; doublon de colonne ; BOM absent ; dates `JJ/MM` et `MM/JJ` décidées,
conflit, ambiguïté ; `NET_MISMATCH`, `SIGN_MISMATCH`, `FEE_POSITIVE`, montant `1.500,00`, devise `EU`, identifiant
avec `:` ; plafond d'erreurs. Gate ciblé `binary(paypal_csv)` et les tests de `kesh-import` ; gate complet au push.

## Modules touchés (1)

`kesh-import` seul.

## Note de version

*Ajouté* — Kesh lit le CSV « Balance affecting transactions » de PayPal (colonnes en anglais) ; un fichier d'une autre
langue ou d'un autre rapport est refusé en nommant les colonnes qui manquent.

## Notes de développement

- Reprise de l'ancienne fiche 38-7 (client PayPal, `deferred`) : seule l'idée du classement par type survit, en 38-11c.
- Format relevé sur le fichier 2024 de Beat (notes sans donnée personnelle,
  `kesh-gate-logs/epic-38-kiss/paypal-2024-comprendre.md`) : 41 colonnes, UTF-8 avec BOM ; statuts `Completed`,
  `Pending`, `Denied`.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
