# Story 38-11b : réglages du compte PayPal et import du CSV, ventilé par devise (migration de l'epic)

Status: review

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
solde attende son rapprochement (38-11c2).

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
     rien d'écrit ; en CHF, admis, et **le contrat de l'instantané de la 34-2 (AC-10, C-e341-38) tenu** : les gardes 2
     à 4, appelées avant l'écriture d'ouverture, ne font que des lectures verrouillantes (revue P1, M1 ; test de
     l'écriture croisée et test de création avec ouverture).
   - **La devise du compte du plan lié à un compte PayPal ne change plus** (revue P1, M3), dans aucun sens (retirée,
     posée, remplacée), même sans ligne d'écriture : `PUT /api/v1/accounts/{id}` → 409
     `ACCOUNT_CURRENCY_PROVIDER_LINKED` (`details: { bankAccountId }`, code neuf, quatre locales), jugé après
     `ACCOUNT_CURRENCY_LOCKED` et avant `ACCOUNT_CURRENCY_IN_USE`.
   - Un compte PayPal n'est jamais principal (CHECK de l'AC-1) ; sans IBAN, il n'entre pas à la QR-facture, qui
     refuse sur `iban.is_none()` (doc de `bank_account.rs:125-142`). **Lots pain.001** (revue P1, M2) : la création
     d'un lot (`payment_batches::create_batch`, lecture du compte de l'étape (0)) refuse un compte PayPal, **IBAN ou
     non**, en 409 `BANK_ACCOUNT_IS_PROVIDER`, après « clos » et avant l'absence d'IBAN — testé pour les deux, aucun
     lot ; un compte devenu PayPal après la création d'un lot est refusé à la confirmation (`pay_in_tx`).
   - **Refus serveur** (nouveau code `BANK_ACCOUNT_IS_PROVIDER`, 409, `details: { bankAccountId }`) : aperçu et
     création d'un import CAMT ou CSV (`routes/bank_imports.rs` `preview` `:1029`, `create` `:1216`), aperçu et
     création d'un relevé saisi à la main (`routes/bank_statements_manual.rs` `:132`, `:221`), règlement d'une facture
     client sur un compte PayPal (`crates/kesh-db/src/repositories/invoice_settlements_write.rs:202-219`, `provider`
     lu dans la même instruction que `account_kind`, refus après « clos » comme la carte,
     `SettlementAccountIsCreditCard` de la 34-11a2 pour précédent : un `DbError` neuf, rendu 409) et paiement d'une
     facture fournisseur (`supplier_invoices::pay_in_tx_as`, `crates/kesh-db/src/repositories/supplier_invoices.rs:2018`,
     compte bancaire choisi : ce chemin ne refuse pas la carte aujourd'hui, le refus PayPal y est neuf) — refus dont
     héritent la confirmation d'un lot pain.001 et l'**acceptation d'une proposition « facture fournisseur »**
     (`accept_one_supplier_invoice`, qui appelle `pay_in_tx_as`) : là, le refus sort en `FailedProposal`
     `BANK_ACCOUNT_IS_PROVIDER` (`details: { bankAccountId }`, bras nommé de
     `supplier_payment_error_to_failed_proposal`, revue P1, M2), et non en `DATABASE_ERROR`. Le CSV PayPal passe par sa
     route (AC-4) ; les rattachements à une écriture existante (38-11c1) restent admis.
   - ⚠️ **Limites écrites (décision de l'orchestrateur du 2026-11-20, validation P2)** — à reprendre au manuel par
     38-11d : (a) **phase 1 : une facture fournisseur ne se paie pas depuis un compte PayPal** (KISS : les achats PayPal
     passent par les règles, 38-11c2) ; *remède* : régler la facture par le choix « compte interne » (un compte du plan),
     ou passer l'écriture à la main (en devise pour un solde en devise, 37-3b), puis **rattacher** la transaction PayPal à
     cette écriture (38-11c1 AC-1) ; (b) **angle mort assumé, comme la carte (34-11a2, C-e341-41)** — **inventaire
     clos** des écrivains « compte bancaire choisi par l'appelant » (refait à la remédiation de la revue P1, M2) :
     *refusent* le règlement client, le paiement fournisseur et ce qui l'appelle (confirmation d'un lot,
     `accept_one_supplier_invoice`), la création d'un lot pain.001, les imports CAMT, CSV et manuels ; *ne refusent
     pas* un compte PayPal **CHF** — l'acceptation d'une proposition « facture » (`accept_one_invoice`,
     `routes/reconciliation.rs:2580`), les règlements groupés (`routes/reconciliation_group.rs`, qui appelle
     `grouped_settlements.rs:784`, `:2325`), le rattachement de règlement de la 29-8 (`link_in_tx`,
     `reconciliation_link`), **la lecture des propositions de la réconciliation générique** (`get_proposals`,
     `routes/reconciliation.rs:1091`, qui saute le hors-CHF mais propose règles et factures pour une transaction PayPal
     **CHF**), **l'acceptation d'une proposition « règle »** (`accept_one_rule`, refus hors CHF seulement) et
     **l'éclatement d'une proposition** (`accept_one_split`), la **saisie manuelle** et l'**éclatement** d'une transaction
     (`post_manual`, `post_split`, `reconciliation_manual`, `reconciliation_split`). *Tranché* (revue P1) : angle mort
     écrit, non refus — en CHF, ces chemins passent une écriture juste sur le compte du plan du solde (la transaction
     contre sa contrepartie), et le rapprochement PayPal propre (38-11c2) n'existe pas encore ; l'écran les masque
     (`withoutProviders`, 38-11d AC-1), l'API non ; *remède* : ne pas accepter la proposition, la passer en rejet ou la
     traiter par le rapprochement PayPal (38-11c2). Le rattachement à une écriture existante (`link_entry`, 38-11c1)
     est admis, non un angle mort. Un point de traçage est à ouvrir pour la phase 2
     (`kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`, point 1).
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

- [x] T0 — Reprendre `main` ; relire les migrations de `main` et des Epics 47 et 49 ; date de la migration ; preuve
  MariaDB de la forme `DROP CONSTRAINT` + `ADD CONSTRAINT` du même nom.
- [x] T1 — Migration, squash, audit des migrations, test de migration (AC-1).
- [x] T2 — Entités et dépôts `bank_accounts`, `bank_transactions`, `reconciliation_rules` (champs neufs) ;
  constructions existantes (AC-8).
- [x] T3 — Réglages PayPal aux routes des comptes bancaires, `PATCH` refusé, levée de C-37-9, refus aux autres chemins
  (AC-2, AC-3).
- [x] T4 — `with_account_locks` ; route d'import, ventilation, paires neutres, doublons, compte rendu ; registre des
  routes et phrase du manuel d'administration (AC-4 à AC-6).
- [x] T5 — Sauvegarde, export CSV (AC-7) ; i18n (AC-8).

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
ni règlement de facture, ni paiement de facture fournisseur, ni lot de paiement pain.001 (limite de la phase 1 : payer
par un compte interne ou une écriture à la main, puis rattacher la transaction) ; la devise de son compte du plan ne
change plus.
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

## Dev Agent Record

*Développée le 2026-11-20 (Claude Opus 5.5), worktree `kesh-wt38`, branche `epic/38-paypal`, commits `efb5fb0c` (code,
migration, tests, i18n, docs) et `c7790f59` (un `allow` clippy du test de schéma). Base : `ce221ae8` (38-11a `done`).
Journaux : `kesh-gate-logs/s38-11b/`. Commits suivants : `0da7b6d0` (trois tests ajustés après le premier gate complet).*

### T0 — reprise de `main`, date, forme MariaDB

- `git fetch origin main` : `origin/main` n'a aucun commit absent de la branche ; dernière migration de `main`
  `20261118000001_grouped_settlements`. Epic 49 (`20261118000002`, branche `origin/epic/49-…`) et Epic 47
  (`20261119000001`) relues, hors de cette branche : `20261120000001` reste après elles.
- **Preuve MariaDB 10.11.19** (`T0.log`, base jetable `kesh_s3811b_t0` sur `kesh-mariadb-dev2`) : dans **un seul**
  `ALTER TABLE`, `DROP CONSTRAINT IF EXISTS x, ADD CONSTRAINT IF NOT EXISTS x` **retire la contrainte sans la reposer**
  (l'`IF NOT EXISTS` est jugé sur l'état d'avant : `COUNT = 0` après le premier passage). Forme retenue :
  `DROP CONSTRAINT IF EXISTS x, ADD CONSTRAINT x` (sans `IF NOT EXISTS`) dans le même `ALTER` : posée au premier
  passage, présente au rejeu, refuse un compte `bank` sans IBAN (`ERROR 4025`). La migration entière, rejouée à la main
  sur une base migrée, est sans effet (contraintes et index inchangés) → verdict `yes`. Le SQL de retrait écrit en tête
  de la migration a été joué tel quel, puis la migration réappliquée (`T0.log`).
- Clé étrangère : `ADD CONSTRAINT fk… FOREIGN KEY IF NOT EXISTS (…)` (la forme `ADD CONSTRAINT IF NOT EXISTS fk…
  FOREIGN KEY` est une erreur 1064, constatée).

### Ce qui est fait

- **Migration** `20261120000001_paypal_import.sql` (AC-1), conforme à la fiche, avec deux constats de développement :
  (1) la forme de la repose ci-dessus ; (2) **CHECK `chk_bank_accounts_provider` rendue null-sûre** (`provider IS NOT
  NULL AND provider_account IS NOT NULL AND …`) : la rédaction de la fiche laissait passer un libellé sans prestataire
  et un prestataire sans libellé (un CHECK qui vaut `NULL` est satisfait) — trouvé par le test de schéma, corrigé avant
  toute application hors base jetable (P8). MariaDB retire l'index implicite `fk_bank_transactions_bank_account` au
  profit du neuf `idx_bank_transactions_account_txid` (squash) ; le SQL de retrait le repose. Squash régénéré
  (`scripts/regen-test-schema.sh`, client `mariadb` enveloppé en `--skip-ssl` dans `kesh-gate-logs/s38-11b/bin/` : le
  client de la station exige TLS ; d'où aussi l'en-tête du dump, `40101`/`sandbox`, qui diffère de l'ancien).
  `migrations.sha384` : ligne ajoutée. **P5** : ligne `yes` ; recomptés depuis la source : `ls …/*.sql | wc -l` = 96,
  `grep -c '^| \`20'` = 96, `yes` 12 + `tracked-by-sqlx` 84 + `no` 0 = 96. **P6** : `grep -rn
  "migrations.len()\|apply_migrations_up_to" crates/` (77 lignes) — tous les sites résolvent par version
  (`migrations_before`) sauf `migrations_upgrade_path.rs::upgrade_path_preserves_data`, positionnel assumé : `total`
  95 → 96 et `total - 61` → `total - 62`, frontière 34 inchangée (et les quatre mentions de 61/95 du fichier).
  **P7** : DDL seul. **P8** : aucune migration appliquée touchée. Le semis de
  `draft_migration_preserves_a_populated_{016,rc1}_base` appelle `pay_in_tx`, qui lit désormais `provider` sur un
  schéma d'avant la migration : la colonne y est **prêtée** (ajoutée vide avant le semis, retirée avec les autres
  colonnes postérieures), seul moyen de garder le semis par le code de production.
- **Entités et dépôts** (T2) : `BankAccountProvider` aplati dans `BankAccount` (`#[sqlx(flatten)]`, `#[serde(flatten)]`,
  JSON `provider` + `paypal`) ; `BankTransaction`/`NewBankTransaction` (`provider_type`, `counterparty_email`) ;
  `ReconciliationRule`, `NewReconciliationRule`, `UpdateReconciliationRule` (trois filtres, `INSERT` et `UPDATE`
  dynamique complétés ; l'API de la phase 1 passe `None`). **Source unique** des colonnes de `bank_transactions`
  (`bank_imports::TX_COLUMNS` et `reconciliation::BANK_TX_COLUMNS` étaient deux copies). Écrivains SQL complétés :
  `INSERT` du `POST`, `update_for_company` (`BankAccountPutFields` gagne trois champs), `INSERT INTO bank_transactions`
  de `create_with_transactions`, `INSERT`/`UPDATE` de `reconciliation_rules`. **Laissés tels quels, délibérément** :
  `bank_accounts::create` et `upsert_primary_in_tx` (ils créent un compte ordinaire, les trois colonnes prennent leur
  `NULL` ; aucun ne peut créer un compte PayPal). Constructions inventoriées par le compilateur (E0063) : **52
  initialiseurs** complétés (25 + 17 + 10 aux trois passes). Dépôt : `list_provider_balances`,
  `count_transactions_locked`, `existing_transaction_ids`, `accounts::fee_account_facts_in_share_mode` (lecteurs ;
  aucun écrivain neuf, l'inventaire `bank_account_closed_inventory` est resté vert).
- **Routes des comptes bancaires** (T3, AC-2, AC-3) : corps `paypal` (`POST` ; `PUT` en double option : absent =
  inchangé, `null` = retiré) ; garde 1 (libellé) dans `validate_bank_input`, fonction commune au `POST` et au `PUT` ;
  gardes 2 à 4 dans **une** fonction, `check_provider_guards_in_tx`, appelée par les deux sous la sentinelle, après
  la relecture du compte lié, avant tout exclusif sur `bank_accounts` ; garde 5 dans `refuse_provider_change_in_use`,
  en pré-vol (pool) puis sous le `FOR UPDATE` de la ligne. Levée de C-37-9 : paramètre `provider` de
  `validate_journal_account_id` et `locked_ledger_type`, vrai au `POST`/`PUT` sur l'état résultant, toujours faux au
  `PATCH` et à l'assistant. `PATCH` d'un compte PayPal → `400 BANK_ACCOUNT_FIELD_NOT_ALLOWED` (`journalAccountId`), en
  pré-vol et sous verrou. IBAN facultatif, QR-IBAN et principal refusés, carte + `paypal` refusée
  (`CREDIT_CARD_FIELD_NOT_ALLOWED`, `field: paypal`). Traces : les trois champs aux `details`.
  Refus `409 BANK_ACCOUNT_IS_PROVIDER` (`DbError::BankAccountIsProvider`) : imports CAMT/CSV (aperçu, création ; helper
  `refuse_provider_account`), relevé manuel (aperçu, création, par `check_statement`), règlement client (après « clos »
  et la carte, même instruction), paiement fournisseur (après « clos », même instruction).
- **Import** (T4, AC-4 à AC-6) : `crates/kesh-api/src/routes/paypal_imports.rs` (fonctions pures `ventilate`,
  `transactions_of_row`, `dedupe_rows`, constantes `PAYPAL_FEE_SUFFIX`, `PAYPAL_NEUTRAL_PAIR_TYPES`) ; route
  `POST /api/v1/paypal-imports` (même limite de corps que l'import bancaire) ; `with_account_locks` dans
  `kesh-reconciliation/src/mutex.rs` (le nom du verrou et le `GET_LOCK` extraits en deux helpers partagés avec
  `with_account_lock`, sans changer son comportement). Registre `LIB_ROUTES` (`Traced, SansEcritureAuJournal`) :
  157 → 158 routes, 149 → 150 tracées, registre complet 161 → 162, sans écriture au journal 128 → 129 ; manuel
  d'administration (`.tex` seul) : « 150 des 158 », « 150 + 4 + 4 = 158 ». Action d'audit `paypal_import.created`
  (`audit_labels`, quatre locales).
- **AC-7** : trois exports CSV (colonnes **en fin de ligne**), aller-retour d'une sauvegarde et sauvegarde 0.19.0 sans
  les huit colonnes (`admin_full_import_e2e.rs`) ; `ABSENCES_ADMISES_EPIC_37` et `TABLES_TO_TRUNCATE` non touchés.
- **AC-8** : huit codes neufs (`errors.rs`) et treize clés aux quatre locales (dix messages d'erreur, deux de forme du
  libellé, une action d'audit) ; messages `PROVIDER_*` sans nom de prestataire. `docs/api-external.md` : section « Le
  compte PayPal et l'import de son relevé », table des ressources, huit lignes à la table des erreurs.

### Écarts à la fiche (décidés, à relire en revue)

1. **Ordre au `PUT`** : le pré-vol de la garde 5 (`PROVIDER_ACCOUNT_IN_USE`) passe **avant** les refus de forme, et
   non seulement avant `ACCOUNT_IN_FOREIGN_CURRENCY` : sans cela, retirer `paypal` d'un compte mouvementé sans IBAN
   rendait « IBAN invalide » au lieu du 409 qui dit la vraie raison. Au `POST`, l'ordre de la fiche est tenu (forme →
   compte lié → unicité → frais) ; un `404` de compte lié inconnu (pré-vol existant) passe avant la garde 2.
2. **Garde d'unicité sérialisée en fait** : la lecture se fait sous la sentinelle `companies` que prennent le `POST` et
   le `PUT` ; la fiche la disait « non sérialisée ». Le refus d'ambiguïté de l'import (AC-4) reste, testé par une
   ligne écrite en base.
3. **Unités du compte rendu** : `inserted` et `duplicates` comptent des **transactions** (une ligne à frais en compte
   deux) ; une ligne est ignorée entière si son identifiant a paru plus haut dans le fichier ou si sa transaction
   brute **ou** ses frais sont déjà sur le compte.
4. **Période et solde de clôture** d'une entrée : ceux des lignes **insérées** de la devise ; le solde est celui de la
   ligne de date la plus tardive (à date égale, la dernière du fichier). Une devise tout en doublons n'a pas d'entrée
   (fiche) — son empreinte n'est donc pas gardée.
5. **Trace `paypal_import.created`** : `entity_type = bank_account`, `entity_id` = le plus petit identifiant des soldes
   de ce compte PayPal (aucune entité « compte PayPal » n'existe en phase 1).
6. **Libellé** : refusé aussi s'il porte un caractère caché ou un saut de ligne (prédicat commun
   `is_hidden_in_line`), au-delà de la fiche ; libellé de détail plafonné à 500 caractères (borne de la saisie
   manuelle), nom à 255, type à 64, adresse à 254.
7. **Solde d'ouverture d'un compte PayPal EUR** : le test constate `ACCOUNT_REQUIRES_CURRENCY` sur un exercice
   multi-devises ; sur un exercice mono-devise, le refus central est `MULTI_CURRENCY_DISABLED` (rien d'écrit non plus).

### Tests (recomptés de `ce221ae8` à `efb5fb0c`, `grep -cE '#\[(sqlx::)?test'` aux deux bornes)

23 tests neufs : `paypal_import_e2e.rs` 8 (HTTP), `paypal_imports.rs` 9 (unitaires de la ventilation),
`paypal_import_schema.rs` 2 (contraintes), `admin_full_import_e2e.rs` 2 (AC-7), `bank_account.rs` 2 (JSON, borne) ; un
test étendu (`serialize_bank_accounts_csv_porte_la_description`). **Sondes négatives** (`mutations.log`, chaque
mutation annulée par `git checkout`) : refus PayPal du règlement retiré → 1 test rouge ; doublons en base ignorés → 2 ;
levée de C-37-9 retirée sous verrou → 7 ; paires neutres importées → 3 ; garde 5 retirée → 1. La CHECK null-sûre a été
trouvée par le test de schéma lui-même (rouge sur la rédaction de la fiche).

### Gates

Au dernier commit de code `0da7b6d0` (base `kesh_s3811b` remise à zéro avant chaque gate : DROP/CREATE, migrations,
seed) :

- `scripts/test-fast.sh --ci` (fmt, clippy `-D warnings`, nextest) par `heavy.sh` : **6197 exécutés, 6197 passés,
  14 ignorés** (`gate-backend.log`). Deux runs précédents, rouges, ont été corrigés : un `too_many_arguments` du test
  de schéma (clippy n'avait tourné qu'avant sa création), puis 5 tests (liste de clés de la réponse
  `bank_account_opening_balance_e2e`, inventaire `account_columns_inventory` qui exigeait le classement de
  `bank_accounts.fee_account_id`, trois tests de `lettering_documents_backfill` dont le montage paie une facture sur un
  schéma d'avant la migration — colonne `provider` prêtée comme les autres colonnes postérieures).
- `scripts/gate-ci.sh` sur une copie `git archive` de `0da7b6d0` (`gate-ci-src/`, conteneur `kesh-mariadb-dev2` par les
  enveloppes, mem-guard actif) : **6216 passés, 0 échoué, 18 ignorés** (`gate-ci.log`) ; cible de compilation
  supprimée ensuite.
- Frontend : aucun fichier du frontend touché ; `npm run test:unit` lancé parce que des tests Vitest lisent les
  catalogues `.ftl` : **278 fichiers, 4612 tests passés** (`front-vitest.log`, après `npm ci` dans le worktree).
  `npm run check`, `lint-i18n-ownership`, `build` et l'**E2E Playwright non lancés** (la story ne touche aucun code du
  frontend ; aucun écran — 38-11d).
- Migration et T0 : `T0.log` (forme MariaDB, rejeu, SQL de retrait joué puis migration réappliquée).

### Remédiation de la revue P1

*Le 2026-11-20, rapport `kesh-gate-logs/s38-11b/revue-p1-rapport.md` (0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW). Prémisses
relues au code avant correction : les trois MEDIUM et L1, L2 se vérifient ; pour L2, `end_to_end_id` était déjà
tronqué à 255 (`NAME_MAX_CHARS`), seuls `transaction_id` (frais `:FEE`) et `reference` (`TXN `) débordaient.*

- **M1** — `bank_accounts::list_provider_balances_locked` (neuve, `kesh-db`) : comptes bancaires `LOCK IN SHARE MODE`,
  puis devises des comptes du plan par identifiant, sous le même verrou (deux instructions : la portée d'un verrou de
  lecture sur une sous-requête scalaire n'est pas garantie). `check_provider_guards_in_tx` l'emploie : au `POST`, toutes
  les lectures avant l'écriture d'ouverture sont verrouillantes ; doc-comments de `create_once` et de la garde mis à
  jour.
- **M2** — (1) `payment_batches::create_batch` lit `provider` à l'étape (0) et refuse `BankAccountIsProvider` après
  « clos », avant l'IBAN ; (2) bras nommé `BANK_ACCOUNT_IS_PROVIDER` dans `supplier_payment_error_to_failed_proposal` ;
  (3) inventaire clos et décision (angle mort écrit pour `accept_one_invoice`, `accept_one_rule`, `accept_one_split`,
  règlements groupés, `link_in_tx`, saisie manuelle et éclatement) recopiés à l'AC-3, limite (b) corrigée ;
  `docs/api-external.md` (paragraphe du compte PayPal, tables des erreurs).
- **M3** — garde 3 bis de `accounts::guard_currency_update` : tout changement de devise d'un compte lié à un compte
  bancaire de prestataire (`provider_bank_account_linked_in_tx`, `LOCK IN SHARE MODE`) → `DbError::AccountCurrencyProviderLinked`,
  409 `ACCOUNT_CURRENCY_PROVIDER_LINKED` (clé `error-account-currency-provider-linked` aux quatre locales,
  `docs/api-external.md`).
- **L1** — `recheck_targets_locked` (route d'import), en tête de la transaction d'écriture, sous les verrous nommés :
  relit les soldes par `list_provider_balances_locked` ; devise sans solde → 422 `PROVIDER_CURRENCY_ACCOUNT_MISSING`,
  deux → 409 `PROVIDER_CURRENCY_DUPLICATE`, un autre compte que celui verrouillé → 409 `OPTIMISTIC_LOCK_CONFLICT`.
- **L2** — `PAYPAL_TRANSACTION_ID_MAX_CHARS` (255 − `:FEE`, assertion de compilation pour `TXN `) et
  `check_transaction_id_lengths`, appelée après la lecture : 422 `PAYPAL_CSV_INVALID` (`PAYPAL_CSV_LINES`, erreur de ligne
  `transaction_id`) au lieu d'une erreur de base.
- **L3 à L7** : dette, `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` (points 5 à 9 ; point 1 annoté).

**Tests** (recomptés de `44d5069e` à `ac73d4ea`, `grep -cE '#\[(sqlx::)?test'` aux deux bornes) : 6 neufs —
`paypal_import_e2e.rs` 8 → 10 (`la_devise_du_compte_du_plan_d_un_solde_paypal_ne_change_pas`,
`import_relit_les_soldes_sous_ses_verrous`), `bank_account_opening_balance_e2e.rs` 20 → 22
(`solde_paypal_chf_cree_avec_son_ouverture`, `ecriture_croisee_pendant_la_creation_d_un_solde_paypal_est_vue`),
`reconciliation_e2e.rs` 136 → 137 (`accepting_a_supplier_invoice_on_a_provider_account_is_refused_by_name`),
`paypal_imports.rs` 9 → 10 (`identifiant_de_transaction_borne_a_la_colonne`) ; 2 étendus
(`les_autres_chemins_refusent_un_compte_paypal` : lot depuis un compte PayPal sans IBAN — le code attendu passe de
`PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN` à `BANK_ACCOUNT_IS_PROVIDER` — et avec IBAN ; `import_refus_de_tete_roles_et_cles` :
identifiant de 252 caractères → 422). **Sondes négatives** (`kesh-gate-logs/s38-11b/mutations-p1.log`, script
`mutations-p1.py`, fichier restauré après chaque) : les six mutations (M1, M2a lot, M2b bras, M3, L1, L2) font chacune
rougir leur test.

**Gates** au dernier commit de code `ac73d4ea` :

- gate ciblé pendant la remédiation (`cible-3.log` : fmt, clippy workspace `-D warnings`, 222 tests des binaires
  touchés passés ; `cible-4.log` : `paypal_import_e2e` 10/10) ;
- `scripts/test-fast.sh --ci` par `heavy.sh`, base `kesh_s3811b` remise à zéro avant (DROP/CREATE, migrations, seed) :
  **6203 exécutés, 6203 passés, 14 ignorés** (`gate-backend-p1.log`) ;
- `scripts/gate-ci.sh` sur une copie `git archive` de `ac73d4ea` : **6222 passés, 0 échoué, 18 ignorés**
  (`gate-ci-p1.log`), cible de compilation supprimée ensuite ;
- Vitest (les catalogues `.ftl` ont une clé neuve) : **278 fichiers, 4612 tests passés** (`front-vitest-p1.log`) ;
  `npm run check`, `lint-i18n-ownership`, `build` et l'E2E Playwright **non lancés** (aucun fichier du frontend
  touché).

### File List

Hors dépôt : `kesh-gate-logs/s38-11b/` (T0, gates, mutations, prompt de revue).

- `_bmad-output/implementation-artifacts/38-11b-compte-paypal-et-import.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `crates/kesh-api/src/audit_labels.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/exports/csv_tables.rs`
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/src/routes/bank_accounts.rs`
- `crates/kesh-api/src/routes/bank_import_fingerprint.rs`
- `crates/kesh-api/src/routes/bank_imports.rs`
- `crates/kesh-api/src/routes/bank_statements_manual.rs`
- `crates/kesh-api/src/routes/invoice_pdf_service.rs`
- `crates/kesh-api/src/routes/mod.rs`
- `crates/kesh-api/src/routes/onboarding.rs`
- `crates/kesh-api/src/routes/paypal_imports.rs`
- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/src/routes/reconciliation_rules.rs`
- `crates/kesh-api/tests/admin_full_import_e2e.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`
- `crates/kesh-api/tests/bank_accounts_closed_e2e.rs`
- `crates/kesh-api/tests/bank_imports_e2e.rs`
- `crates/kesh-api/tests/dates_bornes_e2e.rs`
- `crates/kesh-api/tests/jour_suisse_nuit_e2e.rs`
- `crates/kesh-api/tests/paypal_import_e2e.rs`
- `crates/kesh-api/tests/project_reassignment_e2e.rs`
- `crates/kesh-api/tests/reconciliation_e2e.rs`
- `crates/kesh-api/tests/reconciliation_group_e2e.rs`
- `crates/kesh-api/tests/reconciliation_link_e2e.rs`
- `crates/kesh-api/tests/reconciliation_link_entry_e2e.rs`
- `crates/kesh-api/tests/reconciliation_manual_e2e.rs`
- `crates/kesh-api/tests/reconciliation_rules_e2e.rs`
- `crates/kesh-api/tests/reconciliation_split_e2e.rs`
- `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`
- `crates/kesh-db/migrations.sha384`
- `crates/kesh-db/migrations/20261120000001_paypal_import.sql`
- `crates/kesh-db/src/backup.rs`
- `crates/kesh-db/src/entities/bank_account.rs`
- `crates/kesh-db/src/entities/bank_transaction.rs`
- `crates/kesh-db/src/entities/mod.rs`
- `crates/kesh-db/src/entities/reconciliation_rule.rs`
- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/src/repositories/accounts.rs`
- `crates/kesh-db/src/repositories/bank_accounts.rs`
- `crates/kesh-db/src/repositories/bank_imports.rs`
- `crates/kesh-db/src/repositories/bank_transactions.rs`
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-db/src/repositories/payment_batches.rs`
- `crates/kesh-db/src/repositories/reconciliation.rs`
- `crates/kesh-db/src/repositories/reconciliation_link.rs`
- `crates/kesh-db/src/repositories/reconciliation_rules.rs`
- `crates/kesh-db/src/repositories/supplier_invoices.rs`
- `crates/kesh-db/test-schema/0001_schema_squash.sql`
- `crates/kesh-db/tests/account_currency_repository.rs`
- `crates/kesh-db/tests/bank_accounts_repository.rs`
- `crates/kesh-db/tests/bank_imports_repository.rs`
- `crates/kesh-db/tests/bank_transactions_repository.rs`
- `crates/kesh-db/tests/credit_card_repository.rs`
- `crates/kesh-db/tests/migrations_upgrade_path.rs`
- `crates/kesh-db/tests/paypal_import_schema.rs`
- `crates/kesh-db/tests/reconciliation_repository.rs`
- `crates/kesh-db/tests/reconciliation_rules_repository.rs`
- `crates/kesh-i18n/locales/de-CH/messages.ftl`
- `crates/kesh-i18n/locales/en-CH/messages.ftl`
- `crates/kesh-i18n/locales/fr-CH/messages.ftl`
- `crates/kesh-i18n/locales/it-CH/messages.ftl`
- `crates/kesh-reconciliation/src/lib.rs`
- `crates/kesh-reconciliation/src/manual.rs`
- `crates/kesh-reconciliation/src/matching.rs`
- `crates/kesh-reconciliation/src/mutex.rs`
- `crates/kesh-reconciliation/src/rules.rs`
- `crates/kesh-reconciliation/src/split.rs`
- `crates/kesh-db/tests/account_columns_inventory.rs`
- `crates/kesh-db/tests/lettering_documents_backfill.rs`
- `crates/kesh-api/tests/bank_account_opening_balance_e2e.rs`
- `_bmad-output/implementation-artifacts/38-11d-ecrans-manuel-e2e-paypal.md` (revue P1 : phrase de l'angle mort)
- `docs/api-external.md`
- `docs/manual/fr/admin-manual.tex`
- `docs/migrations-idempotence-audit.md`

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
- 2026-11-20 — validation P4 (Claude Sonnet 5.5, contexte frais) : renvois « 38-11c » nus désignés 38-11c1 ou 38-11c2 après le découpage ; aucun autre changement.
- 2026-11-20 — développée (Claude Opus 5.5) : T0 à T5, statut `review` ; Dev Agent Record ci-dessus (écarts 1 à 7 à relire en revue P1, prompt `kesh-gate-logs/s38-11b/revue-p1-prompt.md`).
- 2026-11-20 — revue de code P1 (passe complète, une lentille ; rapport `kesh-gate-logs/s38-11b/revue-p1-rapport.md`) :
  0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW. **Revue P1 remédiée** (commits `f31fdbf2` code, `ac73d4ea` tests) : M1 (lectures
  verrouillantes avant l'écriture d'ouverture, `list_provider_balances_locked`), M2 (lot pain.001 refusé dès la
  création, `FailedProposal` `BANK_ACCOUNT_IS_PROVIDER` à l'acceptation d'une facture fournisseur, inventaire clos et
  angle mort tranché à l'AC-3), M3 (`ACCOUNT_CURRENCY_PROVIDER_LINKED`, code neuf, quatre locales), L1 (relecture sous
  les verrous nommés) et L2 (identifiant borné, 422) corrigés ; L3 à L7 à la dette (`issues-a-ouvrir.md`, points 5 à 9).
  Symptômes grepés (code, fiche, `docs/api-external.md`, fiche 38-11d — phrase de l'angle mort reprise au manuel).
  Gates complets verts (Dev Agent Record). Prochaine passe : P2 complète, Opus
  (`kesh-gate-logs/s38-11b/revue-p2-prompt.md`).
