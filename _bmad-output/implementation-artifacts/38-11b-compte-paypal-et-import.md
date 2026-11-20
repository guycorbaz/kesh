# Story 38-11b : réglages du compte PayPal et import du CSV, ventilé par devise (migration de l'epic)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`, numéros de ligne relevés après la fusion de `origin/main`
(commit de fusion `43ceb541`). Epic 38, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 38-11a
(lecteur), l'Epic 37 publié en 0.19.0 (devise d'un compte, cours, lignes en devise). **Suivie par** : 38-11c
(rapprochement), 38-11d (écrans). Reprend, réduits, l'AC de l'ancienne 38-2 (compte prestataire, levée de C-37-9
promise par D8) et l'idée de doublon par identifiant de l'ancienne 38-3b.*

## Story

En tant qu'**Administrateur**, je veux faire d'un compte bancaire Kesh le solde, dans une devise, d'un de mes deux
comptes PayPal — sans IBAN, lié à un compte du plan **déjà en devise**, avec son compte de frais —, puis, en tant que
**Comptable**, importer d'un geste le CSV PayPal d'un exercice, ventilé par devise, sans doublon, afin que chaque
mouvement du solde attende son rapprochement (38-11c).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-11b-1` | Un compte bancaire porte, facultativement, des réglages PayPal : le **compte PayPal** (libellé, p. ex. « PayPal www ») et le **compte de frais**. | AC-1, AC-2 |
| `FR-38-11b-2` | Un compte bancaire PayPal se crée sans IBAN et se lie à un compte du plan en devise (levée de C-37-9 pour ce seul cas, engagement D8 de l'Epic 37). | AC-1, AC-3 |
| `FR-38-11b-3` | Un compte PayPal a **au plus un** compte bancaire par devise. | AC-2 |
| `FR-38-11b-4` | Import **à la demande** (route et bouton) du CSV d'un compte PayPal : chaque devise va au compte bancaire de cette devise ; une devise sans compte refuse tout l'import en la nommant. | AC-4, AC-5 |
| `FR-38-11b-5` | Lignes `Status ≠ Completed` et lignes `Memo` écartées ; paires de retenue neutres écartées ; doublons (même identifiant de transaction sur le même compte) ignorés ; tout est compté au compte rendu. | AC-5, AC-6 |
| `FR-38-11b-6` | Une ligne avec frais donne deux transactions : le brut et les frais. | AC-5 |
| `NFR-38-11b-1` | **Une** migration, non breaking, DDL seul. | AC-1 |
| `NFR-38-11b-2` | Sauvegarde, réimport d'une sauvegarde 0.19.0, export CSV et remise à zéro restent justes. | AC-7 |
| `NFR-38-11b-3` | Aucune modification du chemin d'import CAMT/CSV existant au-delà des champs neufs passés à `None`. | AC-5, AC-8 |

## Critères d'acceptation

1. **AC-1 — Migration `20261120000001_paypal_import.sql`** (après `20261118000001_grouped_settlements` de `main`,
   `20261118000002` de l'Epic 49 et `20261119000001` de l'Epic 47 ; si une migration de `main` la dépasse à la
   reprise de `main`, la renuméroter **avant toute application hors base jetable**, P8). **Non breaking** au sens du
   ⚠️ de P1 : `kesh_version_min_required` reste à sa valeur (0.17.6), aucune version Cargo relevée. En tête, le même
   argument que `20261118000001_grouped_settlements.sql` : un binaire antérieur refuse de démarrer sur la base
   migrée et refuse toute sauvegarde qui porte les colonnes neuves ; aucun chemin ne lui fait lire les valeurs
   neuves. Trois `ALTER TABLE` :
   - **`bank_accounts`** : `provider VARCHAR(16) NULL`, `provider_account VARCHAR(120) NULL`, `fee_account_id BIGINT
     NULL` + FK `fk_bank_accounts_fee_account` → `accounts(id)` `ON DELETE RESTRICT` ; CHECK
     `chk_bank_accounts_provider` : `provider IS NULL AND provider_account IS NULL AND fee_account_id IS NULL` **ou**
     (`BINARY provider = 'paypal'` et `account_kind = 'bank'` et `is_primary = FALSE` et
     `CHAR_LENGTH(TRIM(provider_account)) > 0`) ; la contrainte d'IBAN
     (`chk_bank_accounts_bank_has_iban`, `migrations/20261110000001_bank_accounts_without_iban.sql:48-49`) est
     **retirée puis reposée sous le même nom** : `CHECK (account_kind <> 'bank' OR iban IS NOT NULL OR provider IS NOT
     NULL)` — inchangée pour tout compte sans réglage PayPal (`DROP CONSTRAINT` n'est pas dans la liste de P3) ;
     index unique `uq_bank_accounts_provider_currency` **impossible** (la devise est celle du compte du plan, autre
     table) : l'unicité par devise est une garde d'application (AC-2).
   - **`bank_transactions`** : `provider_type VARCHAR(64) NULL` (le `Type` PayPal), `counterparty_email
     VARCHAR(254) NULL`, index `idx_bank_transactions_account_txid (bank_account_id, transaction_id)` (recherche de
     doublon, AC-6).
   - **`reconciliation_rules`** (servi par 38-11c) : `MODIFY COLUMN match_type ENUM(<les quatre valeurs
     d'aujourd'hui, dans leur ordre>, 'provider_type', 'counterparty_email_exact') NOT NULL` — valeurs **ajoutées en
     fin** — avec le commentaire `-- breaking-skip-bump:` (P4) : aucun binaire antérieur ne peut lire une ligne qui
     porte une valeur neuve (refus de démarrage par `sqlx`, refus des sauvegardes à colonnes inconnues) ; une passe
     de revue confirme la justification ; `direction VARCHAR(8) NULL` + CHECK `chk_reconciliation_rules_direction`
     (`direction IS NULL OR BINARY direction IN ('credit', 'debit')`). L'index unique
     `uq_reconciliation_rules_match_active` (`20260513000001_reconciliation_rules.sql:65-66`) **n'est pas touché**
     (limite KISS : une seule règle active par type et valeur, quel que soit le sens — écrite au manuel par 38-11d).
   Aucun `UPDATE` (P7 sans objet, `every_data_backfill_migration_is_triaged` vert). **P5** : ligne à
   `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx`), en-tête `## Table d'audit (N migrations)` et
   ligne `Total` recomptés (`ls crates/kesh-db/migrations/*.sql | wc -l` : 96 attendu, 95 au relevé ;
   `grep -c '^| \`20' docs/migrations-idempotence-audit.md`), partitions recomptées depuis le tableau. **P6** : `grep
   -rn "migrations.len()\|apply_migrations_up_to" crates/` exécuté, chaque site inspecté, au Change Log. **P8** :
   aucune migration appliquée touchée. Squash de test régénéré (`scripts/regen-test-schema.sh`,
   `test_schema_guard.rs` vert ; ordre des valeurs ENUM vérifié à l'octet dans le squash). Un test de migration
   vérifie : compte `bank` sans IBAN refusé sans `provider`, admis avec ; `provider = 'PayPal'` refusé (octet) ;
   compte PayPal principal refusé. **Gate complet** même en cours de boucle (exception `kesh-db`).
2. **AC-2 — Réglages PayPal d'un compte bancaire** (routes existantes `POST /api/v1/bank-accounts` et `PUT
   /api/v1/bank-accounts/{id}`, `crates/kesh-api/src/lib.rs:1053` ; Administrateur, comme aujourd'hui). Le corps gagne
   un objet facultatif `paypal: { account: string, feeAccountId: number | null }` ; absent au `PUT` = inchangé,
   `null` = retiré. Gardes, dans cet ordre, chacune un refus nommé (quatre locales) :
   - `account` rogné, 1 à 120 caractères (400 `VALIDATION_ERROR`) ; numéro de carte refusé comme dans les autres champs
     (`refuse_card_number`) ;
   - compte lié **obligatoire** pour un compte PayPal (400 `PAYPAL_LEDGER_REQUIRED`) : sa devise **est** celle du
     solde ;
   - **un compte bancaire par devise et par compte PayPal** : un autre compte bancaire non archivé de même
     `provider_account` lié à un compte du plan de même devise (CHF compris) → 409 `PAYPAL_CURRENCY_DUPLICATE`
     (`details: { account, currency, bankAccountId }`), jugé sous le verrou partagé du compte du plan ;
   - compte de frais : actif, imputable, de type **charge** ; CHF ou de la **même devise** que le compte lié (une
     écriture ne porte qu'une devise étrangère, `FX_MULTIPLE_CURRENCIES` de 37-3a) → 400 `PAYPAL_FEE_ACCOUNT_INVALID`
     (`details: { accountId, reason }`, `reason` ∈ `not_found`, `not_postable`, `not_expense`, `currency`) ; absent
     admis (les frais vont alors « à examiner », 38-11c) ;
   - retirer les réglages, changer `account` ou le compte lié d'un compte PayPal qui porte déjà une transaction → 409
     `PAYPAL_ACCOUNT_IN_USE` ; changer le compte de frais reste admis.
   L'entité `BankAccount` (`crates/kesh-db/src/entities/bank_account.rs`) et les lectures du dépôt
   (`repositories/bank_accounts.rs`) portent les trois colonnes ; la réponse gagne `paypal: { account, feeAccountId }
   | null`. Trace d'audit des créations et modifications : les champs neufs entrent au `details` existant.
3. **AC-3 — IBAN facultatif et compte du plan en devise, pour le seul compte PayPal.**
   - `validate_bank_input` (`crates/kesh-api/src/routes/bank_accounts.rs:548`, exigence d'IBAN `:563-577`) : avec
     `paypal`, l'IBAN est facultatif ; s'il est donné, il est contrôlé comme aujourd'hui ; le QR-IBAN est refusé (400
     `BANK_ACCOUNT_FIELD_NOT_ALLOWED`, `field: qrIban`).
   - Levée de C-37-9 : les deux refus `ACCOUNT_IN_FOREIGN_CURRENCY` d'un lien bancaire —
     `locked_ledger_type` (`bank_accounts.rs:435-458`, refus `:444-452`) et le pré-vol (`:680-691`) — **ne
     s'appliquent pas** quand le corps porte `paypal` ; ils restent pour tout autre compte bancaire (tests existants
     de 37-1 verts, inchangés). Le verrou partagé sur la devise du compte du plan (`accounts::type_in_share_mode`,
     `repositories/accounts.rs:633`) reste la lecture qui juge.
   - Un compte PayPal n'est jamais principal (CHECK de l'AC-1) ; sans IBAN, il n'entre ni aux lots pain.001 ni à la
     QR-facture, qui refusent sur `iban.is_none()` (doc de `bank_account.rs:125-142`) — vérifié par un test qui
     tente un lot pain.001 depuis un compte PayPal (refus existant).
4. **AC-4 — Route d'import** `POST /api/v1/paypal-imports` (multipart : `file`, `account`), Comptable et plus, clés
   d'API `read-write` admises (comme `POST /api/v1/bank-imports`, `lib.rs:946`), inscrite au registre des routes
   auditées (`crates/kesh-api/tests/audit_route_registry.rs`) et à `LIB_ROUTES` si le dépôt la compte. Taille
   maximale : celle de l'import bancaire. Ordre des refus, rien d'écrit à aucun : 400 `VALIDATION_ERROR` (champ
   manquant) → 404 `PAYPAL_ACCOUNT_NOT_FOUND` (aucun compte bancaire non archivé de ce `provider_account`) → 422
   `PAYPAL_CSV_INVALID` (erreur de 38-11a, `details` : colonnes manquantes ou erreurs de ligne, forme de
   `CsvLineErrorPayload`, `bank_imports.rs:702`) → 422 `PAYPAL_CURRENCY_ACCOUNT_MISSING` (`details: { currencies:
   [...] }` : toutes les devises du fichier, après filtres de l'AC-5, sans compte bancaire de ce compte PayPal ; un
   nouveau solde n'est jamais créé d'office) → 409 `BANK_ACCOUNT_CLOSED` (un des comptes visés clos) → 422
   `BANK_IMPORT_DUPLICATE_FILE` (même fichier déjà importé : empreinte de l'AC-6).
5. **AC-5 — Ventilation.** Lignes **écartées** avant tout, comptées par motif : `status` ≠ `Completed` (sans casse)
   → `notCompleted` (listées : identifiant, type, statut) ; `balance_impact = Memo` → `memo` ; **paires neutres** :
   deux lignes `Completed` du fichier, même devise, même `Reference Txn ID` non vide, brut opposé, frais nuls, de
   types de la liste fermée `PAYPAL_NEUTRAL_PAIR_TYPES` (`Payment Review Hold`, `Payment Review Release`, `Hold on
   Balance for Dispute Investigation`, `Reversal of General Account Hold`, `Account Hold for Open Authorization`,
   `Reversal of Account Hold`) → `neutralPairs` (une retenue sans sa levée dans le fichier reste importée). Puis, par
   devise, **une** entrée `bank_imports` (`create_with_transactions`, `repositories/bank_imports.rs:72`) sur le
   compte bancaire de la devise : `source_format = 'CSV'` (valeur existante : aucun comportement n'en dépend, aucune
   valeur neuve), `filename` = nom du fichier, période = première et dernière date de la devise, `closing_balance` =
   `Balance` de la dernière ligne de la devise si présent, `opening_balance` = `NULL`. Chaque ligne donne
   (`NewBankTransaction`, `entities/bank_transaction.rs:113`, gagne `provider_type` et `counterparty_email`, posés à
   `None` par les chemins CAMT, CSV et manuel) :
   - **brut** (si `gross ≠ 0`) : `amount = gross`, `currency`, `booking_date = date`, `value_date = None`,
     `transaction_id` = identifiant, `reference = "TXN <id>"`, `end_to_end_id` = `Reference Txn ID`,
     `counterparty_name` = `Name`, `counterparty_email` = `From Email Address` pour un crédit, `To Email Address`
     pour un débit, `provider_type` = `Type`, `details` = `Type`, `Item Title`, `Subject`, `Note` non vides joints par
     « · » (tronqué comme les autres imports) ;
   - **frais** (si `fee ≠ 0`) : mêmes champs, `amount = fee`, `transaction_id = "<id>:FEE"` (constante
     `PAYPAL_FEE_SUFFIX` ; un identifiant PayPal n'a jamais de `:`, 38-11a AC-3), `details` = « Frais PayPal · »
     suivi du `Type`.
   La somme des transactions d'une devise égale la somme de ses `Net` (test). L'import passe sous le verrou de
   **chaque** compte visé, pris par identifiant croissant (`with_account_lock`,
   `crates/kesh-reconciliation/src/mutex.rs:75`, comme `create_under_account_lock`, `bank_imports.rs:1278`) ; une
   seule transaction de base : tout ou rien. Trace d'audit : celle de l'import CSV existant, par devise (relevée au
   code au développement), plus `paypal_import.created` (`details` : compte PayPal, nom du fichier, compte rendu de
   l'AC-6).
6. **AC-6 — Doublons et compte rendu.** Empreinte du fichier : `file_hash` = SHA-256 de `fichier ‖ ":" ‖ devise`
   (une par `bank_imports` ; la contrainte de longueur `chk_bank_imports_hash_len` tient) ; refus de l'AC-4 si
   `find_by_company_and_hash` (`repositories/bank_imports.rs:308`) en trouve une. **Doublon de transaction** : une
   transaction du même compte bancaire porte déjà ce `transaction_id` (brut ou `:FEE`) → la ligne est **ignorée**
   (un relevé qui chevauche le précédent s'importe ; la fenêtre de doublons date-montant-référence des imports
   bancaires, `find_in_dedup_window`, `repositories/bank_transactions.rs:83`, **ne s'applique pas** ici : deux ventes
   du même montant le même jour sont deux transactions). Une devise dont toutes les lignes sont des doublons ne crée
   pas d'entrée `bank_imports`. Réponse `201` : `{ account, currencies: [{ currency, bankAccountId, importId |
   null, inserted, duplicates }], skipped: { notCompleted: [...], memo, neutralPairs } }`.
7. **AC-7 — Sauvegarde, export, remise à zéro.** `.keshbackup` : les colonnes passent seules (lecture de
   `information_schema`) ; aller-retour d'un compte PayPal et de ses transactions testé (`admin_full_import_e2e.rs`).
   Liste close des absences admises (`ABSENCES_ADMISES_EPIC_37`, `crates/kesh-db/src/backup.rs:170`) : les cinq
   colonnes neuves de `bank_accounts` et `bank_transactions`, et `reconciliation_rules.direction`, avec la version de
   la migration (≥ `PREMIERE_MIGRATION_EPIC_37`, `:238`) — une sauvegarde 0.19.0 s'importe, comptes sans réglage
   PayPal (test). Export CSV : colonnes ajoutées à `serialize_bank_accounts_csv` (`exports/csv_tables.rs:680`),
   `serialize_bank_transactions_csv` (`:776`) et `serialize_reconciliation_rules_csv` (`:1119`) ;
   `chaque_colonne_du_schema_est_exportee_ou_ecartee` (`:2658`) vert. `TABLES_TO_TRUNCATE` (`backup.rs:46`) inchangé
   (FK neuve `bank_accounts → accounts` déjà dans l'ordre enfants → parents : vérifié par
   `backup_inventory_matches_schema`).
8. **AC-8 — Non-régression et i18n.** Tests des imports CAMT, CSV et manuels inchangés en assertions ; les
   constructions de `NewBankTransaction` et `BankAccount` gagnent leurs champs (l'échec de compilation les
   inventorie, nombre au Change Log). Codes neufs (`PAYPAL_LEDGER_REQUIRED`, `PAYPAL_CURRENCY_DUPLICATE`,
   `PAYPAL_FEE_ACCOUNT_INVALID`, `PAYPAL_ACCOUNT_IN_USE`, `PAYPAL_ACCOUNT_NOT_FOUND`, `PAYPAL_CSV_INVALID`,
   `PAYPAL_CURRENCY_ACCOUNT_MISSING`) : `errors.rs` et messages aux quatre locales (`crates/kesh-i18n/locales/*`).

## Tâches

- [ ] T0 — Reprendre `main` ; relire les migrations de `main` et des Epics 47 et 49 ; date de la migration.
- [ ] T1 — Migration, squash, audit des migrations, test de migration (AC-1).
- [ ] T2 — Entités et dépôt `bank_accounts`, `bank_transactions` (champs neufs) ; constructions existantes (AC-8).
- [ ] T3 — Réglages PayPal aux routes des comptes bancaires, levée de C-37-9 (AC-2, AC-3).
- [ ] T4 — Route d'import, ventilation, paires neutres, doublons, compte rendu (AC-4 à AC-6).
- [ ] T5 — Sauvegarde, absences admises, export CSV (AC-7) ; i18n (AC-8).

## Stratégie de tests

- **Unitaires** (fonction pure de ventilation, sans base, dans `kesh-api` ou `kesh-core` selon le patron de
  `bank_imports`) : filtres, paires neutres (paire complète écartée, retenue seule gardée), brut et frais, somme =
  net, détails.
- **HTTP** (`crates/kesh-api/tests/paypal_import_e2e.rs`, `#[sqlx::test]` sur le squash) avec la fixture de 38-11a :
  société multi-devises, comptes EUR, USD, CHF d'un compte PayPal ; import → trois entrées, nombres attendus ;
  réimport → 422 `BANK_IMPORT_DUPLICATE_FILE` ; fichier chevauchant (copie avec deux lignes de plus) → deux
  insérées, le reste en doublons ; devise sans compte → 422 nommant la devise, rien d'écrit ; compte clos → 409 ;
  clé d'API `read` → 403. Réglages : chaque refus de l'AC-2 ; compte EUR lié à un compte du plan EUR avec `paypal`
  (201) et sans (400 `ACCOUNT_IN_FOREIGN_CURRENCY`, inchangé).
- **Migration** : test de l'AC-1 ; `test_schema_guard.rs`.
- Gate **complet** (exception `kesh-db`).

## Modules touchés (4)

`kesh-db` (migration, entités, dépôts, sauvegarde), `kesh-api` (routes `bank_accounts`, `paypal_imports` neuve,
erreurs, exports), `kesh-import` (lecture seule de 38-11a), `kesh-i18n`.

## Note de version

*Ajouté* — Un compte bancaire peut être le solde, dans une devise, d'un compte PayPal : sans IBAN, lié à un compte du
plan en devise, avec son compte de frais. Le CSV PayPal d'un exercice s'importe d'un geste, ventilé par devise ; une
ligne avec frais donne deux transactions ; les doublons sont ignorés.
*Migration* — `20261120000001_paypal_import`, non breaking : la sauvegarde d'avant la mise à jour reste exigée (base,
`.keshbackup`, dossier des documents) ; le retour à la version précédente passe par elle.

## Notes de développement

- Ancienne 38-2 (`deferred`) : table `payment_provider_accounts`, connexions, migration breaking — remplacées par
  trois colonnes de `bank_accounts` ; la phase 2 pourra les déplacer.
- ⚠️ La base de Beat ne reçoit cette migration qu'à une version publiée, ou en connaissance de cause : une migration
  appliquée ne se modifie plus (P8), et son horodatage doit rester après celles de `main` (risque écrit au fichier
  d'epic).

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
