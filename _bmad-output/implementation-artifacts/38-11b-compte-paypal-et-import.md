# Story 38-11b : réglages du compte PayPal et import du CSV, ventilé par devise (migration de l'epic)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`, numéros de ligne relevés après la fusion de `origin/main`
(commit de fusion `43ceb541`), revérifiés à la validation P1 sur `origin/main` `fd04099f` (arbre `crates/` identique).
Epic 38, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 38-11a (lecteur), l'Epic 37 publié en
0.19.0 (devise d'un compte, cours, lignes en devise). **Suivie par** : 38-11c1 (rattachement en devise, filtres de règle), 38-11c2 (revue et acceptation), 38-11d (écrans).
Reprend, réduits, l'AC de l'ancienne 38-2 (compte prestataire, levée de C-37-9 promise par D8) et l'idée de doublon
par identifiant de l'ancienne 38-3b.*

## Story

En tant que **Comptable** (les routes des comptes bancaires sont des routes Comptable+, `comptable_routes`,
`crates/kesh-api/src/lib.rs:579`, `:1053-1060`), je veux faire d'un compte bancaire Kesh le solde, dans une devise,
d'un de mes deux comptes PayPal — sans IBAN, lié à un compte du plan **déjà en devise**, avec son compte de frais —,
puis importer d'un geste le CSV PayPal d'un exercice, ventilé par devise, sans doublon, afin que chaque mouvement du
solde attende son rapprochement (38-11c).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-11b-1` | Un compte bancaire porte, facultativement, des réglages de prestataire : le **prestataire** (`paypal` seul servi ; `wise` et `stripe` réservés au schéma pour les Epics 39 et 40), le **compte du prestataire** (libellé, p. ex. « PayPal www ») et le **compte de frais**. | AC-1, AC-2 |
| `FR-38-11b-2` | Un compte bancaire PayPal se crée sans IBAN et se lie à un compte du plan en devise (levée de C-37-9 pour ce seul cas, engagement D8 de l'Epic 37). | AC-1, AC-3 |
| `FR-38-11b-3` | Un compte PayPal a **au plus un** compte bancaire par devise. | AC-2, AC-4 |
| `FR-38-11b-4` | Import **à la demande** (route et bouton) du CSV d'un compte PayPal : chaque devise va au compte bancaire de cette devise ; une devise sans compte refuse tout l'import en la nommant. | AC-4, AC-5 |
| `FR-38-11b-5` | Lignes `Status ≠ Completed` et lignes `Memo` écartées ; paires de retenue neutres écartées ; doublons (même identifiant de transaction sur le même compte) ignorés ; tout est compté au compte rendu. | AC-5, AC-6 |
| `FR-38-11b-6` | Une ligne avec frais donne deux transactions : le brut et les frais, **chacune de son signe** (frais négatifs d'ordinaire, positifs sur certains remboursements, 38-11a AC-5). | AC-5 |
| `FR-38-11b-7` | Un compte PayPal est refusé **côté serveur** aux imports bancaires génériques (CAMT, CSV, saisie manuelle) et aux règlements de facture (client et fournisseur). Le refus du paiement d'une facture fournisseur est une **limite de la phase 1** (remède à l'AC-3). | AC-3 |
| `NFR-38-11b-1` | **Une** migration, non breaking, DDL seul, sans `MODIFY COLUMN`. | AC-1 |
| `NFR-38-11b-2` | Sauvegarde, réimport d'une sauvegarde 0.19.0, export CSV et remise à zéro restent justes. | AC-7 |
| `NFR-38-11b-3` | Aucune modification du chemin d'import CAMT/CSV existant au-delà des champs neufs passés à `None` et du refus d'un compte PayPal. | AC-3, AC-8 |

## Critères d'acceptation

1. **AC-1 — Migration `20261120000001_paypal_import.sql`** (après `20261118000001_grouped_settlements` de `main`,
   `20261118000002_invoice_line_types` de l'Epic 49 et `20261119000001_account_mirror` de l'Epic 47 ; si une migration
   de `main` la dépasse à la reprise de `main`, la renuméroter **avant toute application hors base jetable**, P8).
   **Non breaking** au sens du ⚠️ de P1 : `kesh_version_min_required` reste à sa valeur (0.17.6), aucune version Cargo
   relevée. En tête, le même argument que `20261118000001_grouped_settlements.sql` : un binaire antérieur refuse de
   démarrer sur la base migrée et refuse toute sauvegarde qui porte les colonnes neuves ; aucun chemin ne lui fait
   lire les valeurs neuves. **Aucun `MODIFY COLUMN`, `CHANGE COLUMN`, `DROP COLUMN` ni `RENAME`** (P3 sans objet ;
   décision de l'orchestrateur du 2026-11-20 : pas d'ajout de valeurs à l'`ENUM` `match_type`). Trois `ALTER TABLE`,
   un par table, en formes `IF NOT EXISTS` là où MariaDB les admet (précédent `20261118000002`) :
   - **`bank_accounts`** : `provider VARCHAR(16) NULL`, `provider_account VARCHAR(120) NULL`, `fee_account_id BIGINT
     NULL` + FK `fk_bank_accounts_fee_account` → `accounts(id)` `ON DELETE RESTRICT` ; CHECK
     `chk_bank_accounts_provider` : `provider IS NULL AND provider_account IS NULL AND fee_account_id IS NULL` **ou**
     (`BINARY provider IN ('paypal', 'wise', 'stripe')` et `account_kind = 'bank'` et `is_primary = FALSE` et
     `CHAR_LENGTH(TRIM(provider_account)) > 0`) — `wise` et `stripe` **réservés** pour les Epics 39 et 40, qui n'auront
     ainsi pas de migration ; **aucun chemin de l'API ne les écrit en phase 1** (le corps ne porte qu'un objet
     `paypal`, AC-2 : rien d'autre à refuser). La contrainte d'IBAN (`chk_bank_accounts_bank_has_iban`,
     `migrations/20261110000001_bank_accounts_without_iban.sql:48-49`) est **retirée puis reposée sous le même nom** :
     `CHECK (account_kind <> 'bank' OR iban IS NOT NULL OR provider IS NOT NULL)` — inchangée pour tout compte sans
     prestataire (`DROP CONSTRAINT` n'est pas dans la liste de P3) ; la forme (retrait et pose du même nom dans un
     seul `ALTER TABLE`, ou deux) est **prouvée sur MariaDB 10.11 à T0**, au Change Log. L'unicité par devise n'est pas
     un index (la devise est celle du compte du plan, autre table) : garde d'application (AC-2) et refus d'ambiguïté à
     l'import (AC-4).
   - **`bank_transactions`** : `provider_type VARCHAR(64) NULL` (le `Type` PayPal), `counterparty_email
     VARCHAR(254) NULL`, index `idx_bank_transactions_account_txid (bank_account_id, transaction_id)` (recherche de
     doublon, AC-6).
   - **`reconciliation_rules`** (servi par 38-11c1) : trois **filtres facultatifs**, combinés au critère existant
     (`match_type`, `match_value`, inchangés et toujours obligatoires) — `provider_type VARCHAR(64) NULL`,
     `counterparty_email VARCHAR(254) NULL`, `direction VARCHAR(8) NULL` ; CHECK `chk_reconciliation_rules_filters` :
     `(provider_type IS NULL OR CHAR_LENGTH(TRIM(provider_type)) > 0) AND (counterparty_email IS NULL OR
     CHAR_LENGTH(TRIM(counterparty_email)) > 0) AND (direction IS NULL OR BINARY direction IN ('credit', 'debit'))`.
     L'index unique `uq_reconciliation_rules_match_active` (`20260513000001_reconciliation_rules.sql:65-66`, sur
     `company_id, match_type, active_uniq`) **n'est pas touché** : **limite KISS écrite** — deux règles actives de même
     critère et même valeur ne peuvent différer par leurs seuls filtres ou leur sens (409
     `RECONCILIATION_RULE_DUPLICATE` existant, testé en 38-11c1 ; écrite au manuel par 38-11d).
   Aucun `UPDATE` (P7 sans objet, `every_data_backfill_migration_is_triaged` vert). **P5** : ligne à
   `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx` ou `yes` selon les formes retenues), en-tête
   `## Table d'audit (N migrations)` et ligne `Total` **recomptés depuis la source** (`ls
   crates/kesh-db/migrations/*.sql | wc -l` ; 95 au relevé du 2026-11-20 sur la branche, davantage si les Epics 47 et
   49 sont dans `main` à la reprise ; `grep -c '^| \`20' docs/migrations-idempotence-audit.md`), partitions
   recomptées depuis le tableau. **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` exécuté,
   chaque site inspecté, au Change Log. **P8** : aucune migration appliquée touchée. Squash de test régénéré
   (`scripts/regen-test-schema.sh`, `test_schema_guard.rs` vert). Un test de migration vérifie : compte `bank` sans
   IBAN refusé sans `provider`, admis avec ; `provider = 'PayPal'` refusé (octet) ; `provider = 'wise'` admis par le
   schéma ; compte PayPal principal refusé ; filtre de règle vide (`'  '`) et `direction = 'Credit'` refusés.
   **Gate complet** même en cours de boucle (exception `kesh-db`).
2. **AC-2 — Réglages PayPal d'un compte bancaire** (routes existantes `POST /api/v1/bank-accounts` et `PUT
   /api/v1/bank-accounts/{id}`, `lib.rs:1053-1060`, Comptable+ comme aujourd'hui). Le corps gagne un objet facultatif
   `paypal: { account: string, feeAccountId: number | null }` ; absent au `PUT` = inchangé, `null` = retiré ; il pose
   `provider = 'paypal'`. Le `PUT` d'un compte PayPal admet `iban: null` (AC-3). Gardes, dans cet ordre, chacune un
   refus nommé (quatre locales), **une seule fonction** appelée par le `POST` et le `PUT`, jugée sur l'**état
   résultant** (corps, ou réglages existants si `paypal` est absent au `PUT`) :
   - `account` rogné, 1 à 120 caractères (400 `VALIDATION_ERROR`) ; numéro de carte refusé comme dans les autres champs
     (`refuse_card_number`, `routes/bank_accounts.rs:387`) ;
   - compte lié **obligatoire** pour un compte PayPal (400 `PROVIDER_LEDGER_REQUIRED`) : sa devise **est** celle du
     solde ;
   - **un compte bancaire par devise et par compte PayPal** : un autre compte bancaire non archivé de même
     `provider_account` (comparaison de la collation, sans casse) lié à un compte du plan de même devise (CHF compris)
     → 409 `PROVIDER_CURRENCY_DUPLICATE` (`details: { account, currency, bankAccountId }`). Cette garde n'est **pas
     sérialisée** (deux créations simultanées sur deux comptes du plan différents ne prennent pas le même verrou) :
     l'import refuse un doublon par devise (AC-4), seul filet ; limite écrite ;
   - compte de frais : actif, imputable, de type **charge** ; CHF ou de la **même devise** que le compte lié (une
     écriture ne porte qu'une devise étrangère, `FX_MULTIPLE_CURRENCIES` de 37-3a) → 400 `PROVIDER_FEE_ACCOUNT_INVALID`
     (`details: { accountId, reason }`, `reason` ∈ `not_found`, `not_postable`, `not_expense`, `currency`) ; absent
     admis (les frais vont alors « à examiner », 38-11c2) ;
   - retirer les réglages, changer `account` ou le compte lié d'un compte PayPal qui porte déjà une transaction → 409
     `PROVIDER_ACCOUNT_IN_USE` ; changer le compte de frais reste admis.
   **`PATCH /api/v1/bank-accounts/{id}`** (`patch_bank_account_journal_link`, `:2056`, lien du compte du plan, Comptable+)
   sur un compte PayPal → 400 `BANK_ACCOUNT_FIELD_NOT_ALLOWED` (`field: journalAccountId`, code existant) : on ne
   délie ni ne change le compte du plan d'un compte PayPal par ce chemin (le `PUT` porte les gardes) ; pour tout autre
   compte, le `PATCH` est inchangé (refus `ACCOUNT_IN_FOREIGN_CURRENCY` compris, `:2116`).
   L'entité `BankAccount` (`crates/kesh-db/src/entities/bank_account.rs`) et les lectures du dépôt
   (`repositories/bank_accounts.rs`) portent les trois colonnes ; la réponse gagne `provider: 'paypal' | 'wise' | 'stripe' | null` (la colonne, lue telle quelle : l'écran en tire
   `isProvider`, 38-11d AC-1, valable pour les Epics 39 et 40 sans retouche) et `paypal: { account, feeAccountId }
   | null` (`null` aussi pour un `provider` réservé, inatteignable en phase 1). Trace d'audit des créations et
   modifications : les champs neufs entrent au `details` existant.
3. **AC-3 — IBAN facultatif, compte du plan en devise, refus aux autres chemins, pour le seul compte PayPal.**
   - `validate_bank_input` (`routes/bank_accounts.rs:548`, exigence d'IBAN `:563-577`) : compte PayPal (état
     résultant) → IBAN facultatif (`null` admis à la création et au `PUT`) ; s'il est donné, contrôlé comme aujourd'hui ;
     le QR-IBAN est refusé (400 `BANK_ACCOUNT_FIELD_NOT_ALLOWED`, `field: qrIban`).
   - Levée de C-37-9 : les deux refus `ACCOUNT_IN_FOREIGN_CURRENCY` d'un lien bancaire — `locked_ledger_type`
     (`:435-458`, refus `:444-452`) et le pré-vol (`:680-691`) — **ne s'appliquent pas** quand le compte est PayPal à
     l'issue du `POST` ou du `PUT` (état résultant : un `PUT` sans `paypal` sur un compte PayPal lié à un compte EUR
     reste admis) ; ils restent pour tout autre compte bancaire et pour le `PATCH` (tests existants de 37-1 verts,
     inchangés). **Mécanisme** : `validate_journal_account_id` (`:650`, quatre appelants : `POST` `:1013`, `PUT`
     `:1446`, `PATCH` `:2073`, assistant `routes/onboarding.rs:672`) et `locked_ledger_type` (`:435`, appelée par le `POST`
     et le `PUT` `:429` et par le `PATCH` `:2116`) gagnent un paramètre « compte PayPal à l'issue » ; le `POST` et le
     `PUT` le passent vrai, le `PATCH` et l'assistant **toujours faux** (le `PATCH` d'un compte PayPal est de toute façon
     refusé, ci-dessus). Un `PUT` qui **retire** `paypal` (`null`) d'un compte lié à un compte EUR est jugé sur l'état
     résultant (plus PayPal) : 400 `ACCOUNT_IN_FOREIGN_CURRENCY`, test (s'il porte déjà une transaction, le 409
     `PROVIDER_ACCOUNT_IN_USE` de l'AC-2 passe d'abord). Le verrou partagé sur la devise du compte du plan (`accounts::type_in_share_mode`,
     `repositories/accounts.rs:633`) reste la lecture qui juge.
   - Solde d'ouverture (`openingBalance` du `POST`, `enter_opening_balance` `:1265`) d'un compte PayPal en devise :
     refusé par le contrôle central existant des lignes en devise (`ACCOUNT_REQUIRES_CURRENCY`), constaté par un test,
     rien d'écrit ; en CHF, inchangé.
   - Un compte PayPal n'est jamais principal (CHECK de l'AC-1) ; sans IBAN, il n'entre ni aux lots pain.001 ni à la
     QR-facture, qui refusent sur `iban.is_none()` (doc de `bank_account.rs:125-142`) — vérifié par un test qui tente
     un lot pain.001 depuis un compte PayPal sans IBAN (refus existant).
   - **Refus serveur** (nouveau code `BANK_ACCOUNT_IS_PROVIDER`, 409, `details: { bankAccountId }`) : aperçu et
     création d'un import CAMT ou CSV (`routes/bank_imports.rs` `preview` `:1029`, `create` `:1216`), aperçu et
     création d'un relevé saisi à la main (`routes/bank_statements_manual.rs` `:132`, `:221`), règlement d'une facture
     client sur un compte PayPal (`crates/kesh-db/src/repositories/invoice_settlements_write.rs:202-219`, `provider`
     lu dans la même instruction que `account_kind`, refus après « clos » comme la carte,
     `SettlementAccountIsCreditCard` de la 34-11a2 pour précédent : un `DbError` neuf, rendu 409) et paiement d'une
     facture fournisseur (`supplier_invoices::pay_in_tx_as`, `crates/kesh-db/src/repositories/supplier_invoices.rs:2018`,
     compte bancaire choisi : ce chemin ne refuse pas la carte aujourd'hui, le refus PayPal y est neuf). Le CSV PayPal
     passe par sa route (AC-4) ; les rattachements à une écriture existante (38-11c) restent admis.
   - ⚠️ **Limites écrites (décision de l'orchestrateur du 2026-11-20, validation P2)** — à reprendre au manuel par
     38-11d : (a) **phase 1 : une facture fournisseur ne se paie pas depuis un compte PayPal** (KISS : les achats PayPal
     passent par les règles, 38-11c) ; *remède* : régler la facture par le choix « compte interne » (un compte du plan),
     ou passer l'écriture à la main (en devise pour un solde en devise, 37-3b), puis **rattacher** la transaction PayPal à
     cette écriture (38-11c1 AC-1) ; (b) **angle mort assumé, comme la carte (34-11a2, C-e341-41)** : le refus ne couvre
     que les deux écrivains ci-dessus ; l'acceptation d'une proposition « facture » ou « facture fournisseur » par la
     réconciliation générique (`accept_one_invoice`, `routes/reconciliation.rs:3058`, `accept_one_supplier_invoice`,
     `:3580`), les règlements groupés (`grouped_settlements.rs:784`, `:2325`), le rattachement de règlement de la 29-8
     (`link_in_tx`), **la lecture des propositions de la réconciliation générique** (`get_proposals`,
     `routes/reconciliation.rs:1091`, qui saute le hors-CHF mais propose règles et factures pour une transaction PayPal
     **CHF**) et **l'acceptation d'une proposition « règle »** (`accept_one_rule`, `:4191`, refus hors CHF seulement,
     `:4385-4395`) ne refusent pas un compte PayPal CHF — l'écran les masque (`withoutProviders`, 38-11d AC-1), l'API
     non ; *remède* : ne pas accepter la proposition, la passer en
     rejet ou la traiter par le rapprochement PayPal (38-11c2). Un point de traçage est à ouvrir pour la phase 2 (revue P2,
     rapport).
4. **AC-4 — Route d'import** `POST /api/v1/paypal-imports` (multipart : `file`, `account`), Comptable+, clés d'API
   `read-write` admises (comme `POST /api/v1/bank-imports`, `lib.rs:946`). Inscrite à `LIB_ROUTES`
   (`crates/kesh-api/tests/audit_route_registry.rs:440`) en `Traced, SansEcritureAuJournal` (statut `Traced` : c'est la
   colonne que comptent `traced + exempt + no_matter`, la colonne de rejeu est à part) ; le compte de
   `the_registry_partition_is_what_the_story_declares` (157 au relevé) relevé de un avec sa raison, et la phrase « N
   des M routes à verbe mutant » et la somme « a + b + c = M » du manuel d'administration
   (`docs/manual/fr/admin-manual.tex`, lues par `le_manuel_dit_le_compte_du_registre`) mises à jour — **`.tex` seul**,
   PDF à la release. Taille maximale : celle de l'import bancaire. Ordre des refus, rien d'écrit à aucun : 400
   `VALIDATION_ERROR` (champ manquant) → 404 `PROVIDER_ACCOUNT_NOT_FOUND` (aucun compte bancaire non archivé de ce
   `provider_account` et `provider = 'paypal'`) → 422 `PAYPAL_CSV_INVALID` (erreur de 38-11a, `details` : colonnes
   manquantes ou erreurs de ligne, forme de `CsvLineErrorPayload`, `bank_imports.rs:702`) → 422
   `PROVIDER_CURRENCY_ACCOUNT_MISSING` (`details: { currencies: [...] }` : toutes les devises du fichier, après filtres
   de l'AC-5, sans compte bancaire de ce compte PayPal ; un nouveau solde n'est jamais créé d'office) → 409
   `PROVIDER_CURRENCY_DUPLICATE` (deux comptes bancaires non archivés de ce compte PayPal pour une même devise : la
   course de l'AC-2) → 409 `BANK_ACCOUNT_CLOSED` (un des comptes visés clos) → 422 `BANK_IMPORT_DUPLICATE_FILE` (même
   fichier déjà importé : empreinte de l'AC-6).
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
   - **frais** (si `fee ≠ 0`, **quel que soit son signe** : négatif d'ordinaire, positif sur un remboursement) : mêmes
     champs, `amount = fee`, `transaction_id = "<id>:FEE"` (constante `PAYPAL_FEE_SUFFIX` ; un identifiant PayPal n'a
     jamais de `:`, 38-11a AC-3), `details` = « Frais PayPal · » suivi du `Type`.
   Deux lignes du fichier de même identifiant : la seconde est un doublon (AC-6). La somme des transactions d'une
   devise égale la somme de ses `Net` (test, frais positifs compris). L'import passe sous le verrou de **chaque**
   compte visé, pris par identifiant croissant : helper **neuf** `with_account_locks` dans
   `crates/kesh-reconciliation/src/mutex.rs` (à côté de `with_account_lock`, `:75` : `GET_LOCK` successifs sur la même
   connexion, ids triés et dédoublonnés, libérés en ordre inverse, `ReconciliationError::AccountLocked` au premier
   délai dépassé, tous ceux déjà pris rendus), **partagé avec 38-11c2** (et les Epics 39, 40). **Patron de l'import existant**
   (`create_under_account_lock`, `routes/bank_imports.rs:1532`) : les verrous se tiennent sur une connexion **dédiée**
   (`lock_tx`, `MANUAL_STATEMENT_LOCK_TIMEOUT_SECS`) et l'écriture se fait dans une **seconde** transaction de base,
   une seule pour toutes les devises (tout ou rien) ; un verrou non obtenu → 409 `RECONCILIATION_ACCOUNT_LOCKED`
   (`AppError::ReconciliationAccountLocked`), rien d'écrit (test : verrou tenu sur le second compte → 409, rien écrit,
   le premier verrou rendu). La recherche des doublons (AC-6) se fait **dans** cette transaction, sous les verrous.
   (38-11c2, dont l'acceptation écrit sous les verrous dans la transaction même, appelle le helper sur celle-ci, comme
   `accept_once` appelle `with_account_lock`.)
   Trace d'audit : `bank_import.created` par devise, comme l'import CSV (`routes/bank_imports.rs:1945`, `:1976`), plus
   `paypal_import.created` (`details` : compte PayPal, nom du fichier, compte rendu de l'AC-6), sous l'acteur
   (`NewAuditLogEntry::for_actor`).
6. **AC-6 — Doublons et compte rendu.** Empreinte du fichier : `file_hash` = SHA-256 de `fichier ‖ ":" ‖ devise`
   (une par `bank_imports` ; la contrainte de longueur `chk_bank_imports_hash_len` tient ; l'index n'est plus unique
   depuis `20260507000001`) ; refus de l'AC-4 si `find_by_company_and_hash` (`repositories/bank_imports.rs:308`) en
   trouve une. **Doublon de transaction** : une transaction du même compte bancaire porte déjà ce `transaction_id`
   (brut ou `:FEE`), ou une ligne précédente du même fichier → la ligne est **ignorée** (un relevé qui chevauche le
   précédent s'importe ; la fenêtre de doublons date-montant-référence des imports bancaires,
   `find_in_dedup_window`, `repositories/bank_transactions.rs:83`, **ne s'applique pas** ici : deux ventes du même
   montant le même jour sont deux transactions). Une devise dont toutes les lignes sont des doublons ne crée pas
   d'entrée `bank_imports`. Réponse `201` : `{ account, currencies: [{ currency, bankAccountId, importId | null,
   inserted, duplicates }], skipped: { notCompleted: [...], memo, neutralPairs } }`.
7. **AC-7 — Sauvegarde, export, remise à zéro.** `.keshbackup` : les colonnes passent seules (lecture de
   `information_schema`) ; aller-retour d'un compte PayPal, de ses transactions et d'une règle à filtres testé
   (`admin_full_import_e2e.rs`). Une sauvegarde 0.19.0 s'importe (colonnes neuves `NULL` : la règle générique
   `missing_required_columns` ne refuse qu'une colonne obligatoire) — test ; `ABSENCES_ADMISES_EPIC_37`
   (`crates/kesh-db/src/backup.rs:170`) **n'est pas touché** (liste propre à l'Epic 37 ; précédent `20261118000001`,
   dont les colonnes n'y sont pas). Export CSV : colonnes ajoutées à `serialize_bank_accounts_csv`
   (`exports/csv_tables.rs:680`), `serialize_bank_transactions_csv` (`:776`) et `serialize_reconciliation_rules_csv`
   (`:1119`, les trois filtres) ; `chaque_colonne_du_schema_est_exportee_ou_ecartee` (`:2658`) vert.
   `TABLES_TO_TRUNCATE` (`backup.rs:46`) inchangé (FK neuve `bank_accounts → accounts` déjà dans l'ordre enfants →
   parents : vérifié par `backup_inventory_matches_schema`).
8. **AC-8 — Non-régression et i18n.** Tests des imports CAMT, CSV et manuels inchangés en assertions ; les
   constructions de `NewBankTransaction`, `BankAccount` et `ReconciliationRule` gagnent leurs champs (l'échec de
   compilation les inventorie, nombre au Change Log). ⚠️ **Les écrivains SQL, eux, ne rougissent pas** : une liste de
   colonnes qui oublie un champ le laisse `NULL` en silence. Sites à compléter, relevés sur `origin/main` :
   `INSERT INTO bank_accounts` du `POST` (`routes/bank_accounts.rs:1165`) et du dépôt (`repositories/bank_accounts.rs:65`,
   `:475`), la mise à jour du `PUT` (`update_for_company`), `INSERT INTO bank_transactions` de `create_with_transactions`
   (`repositories/bank_imports.rs:151`), `INSERT INTO reconciliation_rules` (`repositories/reconciliation_rules.rs:218`)
   et son `UPDATE` ; les listes `COLUMNS` (`bank_transactions.rs:21`, `reconciliation_rules.rs:27`, `bank_accounts`) ; le
   test d'aller-retour de l'AC-7 et un test d'import qui relit `provider_type` et `counterparty_email` les tiennent.
   Inventaire `crates/kesh-api/tests/bank_account_closed_inventory.rs` : toute fonction **neuve** qui écrit
   `bank_accounts` ou `bank_transactions` (p. ex. un écrivain dédié aux réglages PayPal) se classe au `REGISTRE` avec sa
   raison, faute de quoi le test rougit ; `create_with_transactions` y est déjà. Codes neufs (`PROVIDER_LEDGER_REQUIRED`,
   `PROVIDER_CURRENCY_DUPLICATE`, `PROVIDER_FEE_ACCOUNT_INVALID`, `PROVIDER_ACCOUNT_IN_USE`, `PROVIDER_ACCOUNT_NOT_FOUND`,
   `PAYPAL_CSV_INVALID`, `PROVIDER_CURRENCY_ACCOUNT_MISSING`, `BANK_ACCOUNT_IS_PROVIDER`) : `errors.rs` et messages aux
   quatre locales (`crates/kesh-i18n/locales/*`) ; les codes `PROVIDER_*` sont **génériques** (Q-39-10 de l'Epic 39 :
   Wise et Stripe les reprennent tels quels, messages sans nom de prestataire). L'entité `ReconciliationRule` porte les
   trois filtres (lus, écrits par 38-11c1).

## Tâches

- [ ] T0 — Reprendre `main` ; relire les migrations de `main` et des Epics 47 et 49 ; date de la migration ; preuve
  MariaDB de la forme `DROP CONSTRAINT` + `ADD CONSTRAINT` du même nom.
- [ ] T1 — Migration, squash, audit des migrations, test de migration (AC-1).
- [ ] T2 — Entités et dépôts `bank_accounts`, `bank_transactions`, `reconciliation_rules` (champs neufs) ;
  constructions existantes (AC-8).
- [ ] T3 — Réglages PayPal aux routes des comptes bancaires, `PATCH` refusé, levée de C-37-9, refus aux autres chemins
  (AC-2, AC-3).
- [ ] T4 — `with_account_locks` ; route d'import, ventilation, paires neutres, doublons, compte rendu ; registre des
  routes et phrase du manuel d'administration (AC-4 à AC-6).
- [ ] T5 — Sauvegarde, export CSV (AC-7) ; i18n (AC-8).

## Stratégie de tests

- **Unitaires** (fonction pure de ventilation, sans base, dans `kesh-api` ou `kesh-core` selon le patron de
  `bank_imports`) : filtres, paires neutres (paire complète écartée, retenue seule gardée), brut et frais (frais
  négatifs et positifs), somme = net, doublon dans le fichier, détails.
- **Verrous** : `with_account_locks` (deux comptes, ordre, délai sur le second rend le premier).
- **HTTP** (`crates/kesh-api/tests/paypal_import_e2e.rs`, `#[sqlx::test]` sur le squash) avec la fixture de 38-11a :
  société multi-devises, comptes EUR, USD, CHF d'un compte PayPal ; import → trois entrées, nombres attendus ;
  réimport → 422 `BANK_IMPORT_DUPLICATE_FILE` ; fichier chevauchant (copie avec deux lignes de plus) → deux
  insérées, le reste en doublons ; devise sans compte → 422 nommant la devise, rien d'écrit ; deux comptes EUR
  (insérés en base) → 409 `PROVIDER_CURRENCY_DUPLICATE` ; compte clos → 409 ; clé d'API `read` → 403. Réglages : chaque
  refus de l'AC-2 ; compte EUR lié à un compte du plan EUR avec `paypal` (201) et sans (400
  `ACCOUNT_IN_FOREIGN_CURRENCY`, inchangé) ; `PUT` sans `paypal` d'un compte PayPal EUR (200) et avec `iban: null`
  (200) ; `PATCH` d'un compte PayPal → 400 ; import CAMT, relevé manuel, règlement de facture et paiement fournisseur
  sur un compte PayPal → 409 `BANK_ACCOUNT_IS_PROVIDER`.
- **Migration** : test de l'AC-1 ; `test_schema_guard.rs`.
- Gate **complet** (exception `kesh-db`).

## Modules touchés (5)

`kesh-db` (migration, entités, dépôts, sauvegarde), `kesh-api` (routes `bank_accounts`, `bank_imports`,
`bank_statements_manual`, `paypal_imports` neuve, erreurs, exports), `kesh-reconciliation` (`with_account_locks`),
`kesh-import` (lecture seule de 38-11a), `kesh-i18n`. Plus une phrase du manuel d'administration (`.tex`).

## Note de version

*Ajouté* — Un compte bancaire peut être le solde, dans une devise, d'un compte PayPal : sans IBAN, lié à un compte du
plan en devise, avec son compte de frais. Le CSV PayPal d'un exercice s'importe d'un geste, ventilé par devise ; une
ligne avec frais donne deux transactions ; les doublons sont ignorés. Un compte PayPal ne reçoit ni relevé bancaire
ni règlement de facture, ni paiement de facture fournisseur (limite de la phase 1 : payer par un compte interne ou une
écriture à la main, puis rattacher la transaction).
*Migration* — `20261120000001_paypal_import`, non breaking : la sauvegarde d'avant la mise à jour reste exigée (base,
`.keshbackup`, dossier des documents) ; le retour à la version précédente passe par elle.

## Notes de développement

- Ancienne 38-2 (`deferred`) : table `payment_provider_accounts`, connexions, migration breaking — remplacées par
  trois colonnes de `bank_accounts` ; la phase 2 pourra les déplacer. Le domaine `provider` admet dès maintenant
  `wise` et `stripe` (décision de l'orchestrateur du 2026-11-20), pour que les Epics 39 et 40 n'aient pas de
  migration ; leur epic ouvrira l'API (objets `wise`, `stripe` du corps, ou un `provider` générique).
- Choix KISS : les filtres de règle sont des colonnes `NULL`, non des valeurs neuves de l'`ENUM` `match_type` (P3) ;
  le prix est un critère principal obligatoire et l'index unique inchangé.
- ⚠️ La base de Beat ne reçoit cette migration qu'à une version publiée, ou en connaissance de cause : une migration
  appliquée ne se modifie plus (P8), et son horodatage doit rester après celles de `main` (risque écrit au fichier
  d'epic).

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 2 HIGH, 6 MEDIUM, 4 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-38-kiss/validation/38-11b-compte-paypal-et-import-P1.md`). Principaux : ENUM
  `match_type` remplacé par trois filtres `NULL` (décision de l'orchestrateur), domaine `provider` élargi à `wise` et
  `stripe` ; rôle des routes corrigé (Comptable+, non Administrateur) ; levée de C-37-9 jugée sur l'état résultant
  (un `PUT` sans `paypal` était refusé) ; `PATCH` d'un compte PayPal refusé ; refus serveur aux imports génériques et
  aux règlements ; registre `LIB_ROUTES` et phrase du manuel d'administration ; course de l'unicité par devise
  refusée à l'import ; `with_account_locks` partagé ; frais positifs ; `ABSENCES_ADMISES_EPIC_37` laissé tel quel ;
  ligne citée `create_under_account_lock` (`:1278` → `:1532`) retirée.
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 2 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-38-kiss/validation/38-11b-compte-paypal-et-import-P2.md`). Principaux : refus du
  paiement d'une facture fournisseur **maintenu** (décision de l'orchestrateur) et écrit comme limite avec son remède,
  angle mort des acceptations de la réconciliation générique nommé (précédent carte 34-11a2), chemins de
  `invoice_settlements_write.rs` et `supplier_invoices.rs` complétés ; mécanisme de la levée de C-37-9
  (`validate_journal_account_id`, `locked_ledger_type`, quatre appelants, `PATCH` et assistant toujours faux, retrait de
  `paypal` par `PUT`) ; verrous sur le patron de `create_under_account_lock` (connexion dédiée, seconde transaction,
  409 `RECONCILIATION_ACCOUNT_LOCKED`) ; écrivains SQL à colonnes explicites (silencieux) et inventaire
  `bank_account_closed_inventory` ; statut `Traced` du registre précisé.
- 2026-11-20 — validation P3 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW, remédiés (rapport
  `kesh-gate-logs/epic-38-kiss/validation/38-11-P3.md`). Codes génériques `PROVIDER_*` (`PROVIDER_ACCOUNT_NOT_FOUND`,
  `PROVIDER_CURRENCY_ACCOUNT_MISSING`, `PROVIDER_CURRENCY_DUPLICATE`, `PROVIDER_LEDGER_REQUIRED`,
  `PROVIDER_FEE_ACCOUNT_INVALID`, `PROVIDER_ACCOUNT_IN_USE` ; `PAYPAL_CSV_INVALID` reste propre au format), repris tels
  quels par 39-11b et 40-7b ; angle mort (b) de l'AC-3 complété de `get_proposals` et `accept_one_rule` ; réponse des
  comptes bancaires avec `provider` (l'écran en tire `withoutProviders`, générique pour les Epics 39 et 40) ; renvois à
  38-11c1 / 38-11c2 après le découpage.
