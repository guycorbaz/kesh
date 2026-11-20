# Story 38-2 : compte bancaire prestataire — compte `bank` et réglages de prestataire (D8, B16)

Status: deferred

*(2026-11-20 — reportée en phase 2 par la décision de Beat (phase 1 KISS, § « Phase 1 KISS » du fichier d'epic) ; validée le 2026-11-15, gardée telle quelle comme base de la phase 2. Ce qu'en reprend la phase 1 est dit dans les fiches 38-11a à 38-11d.)*

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`, numéros de ligne relevés au commit `8b3d548d`. Epic 38 :
décisions E1, E11, E18, E19, E24 ; contrats **C-38-3** (et C-38-2 pour le refus de suppression d'une connexion,
FR-38-1-4), C-38-0, C-38-12 ; décisions de Beat B1, B3, B16 (maintenu par B22), B23 ; engagement **D8** de l'Epic 37
(levée de C-37-9). **Dépend de** : 38-1 (connexions), l'Epic 37 (devise d'un compte et ses refus, 37-1 AC-4 ; liste
close des absences admises, 37-1 AC-6). **Suivie par** : 38-3a (colonnes des transactions et soldes rapportés d'un
compte prestataire, qui étendent le refus `PROVIDER_ACCOUNT_IN_USE`), 38-3b (téléchargement), 38-6a (compte de frais), 38-9a (écrans).*

⛔ **Arbitrage O1 (2026-11-15), qui prime sur le reste de cette fiche : la migration de cette story est l'UNIQUE
migration breaking de l'epic** (la fiche d'origine la disait non breaking et laissait la décision à Beat, Q-38-2-a :
close). Elle porte, dans **un seul fichier** : la table `payment_provider_accounts`, le remplacement de la contrainte
d'IBAN, `reconciliation_rules.match_type` en `VARCHAR(40)` (O2), et `kesh_version_min_required` relevé en dernière
instruction (P2, P2-bis : **dix** crates, `kesh-providers` naissant en 38-3b ; numéro de release à Guy, Q1 de l'epic —
bloquant pour cette fiche comme il l'était pour l'ancienne 38-3). Détail à l'AC-1.

⚠️ **Écart à l'epic, relevé au code (résolu par O1).** L'epic
(E11, inventaire) tient qu'un compte `bank` **sans IBAN** est admis depuis 34-11a1. **C'est faux au code** :
`chk_bank_accounts_bank_has_iban CHECK (account_kind <> 'bank' OR iban IS NOT NULL)`
(`crates/kesh-db/migrations/20261110000001_bank_accounts_without_iban.sql:48-49`) l'interdit au schéma, et
`validate_bank_input` exige l'IBAN d'un compte `bank` (`crates/kesh-api/src/routes/bank_accounts.rs:557-570`) ; seule
la carte de crédit en est dispensée. Un solde PayPal n'a pas d'IBAN. Le code avait prévu ce cas pour **un type neuf**
(`crates/kesh-db/src/entities/bank_account.rs:137-142` : « un futur type sans IBAN devra ajouter `CHECK (is_primary =
FALSE OR iban IS NOT NULL)` »), que B16/B22 écartent. Cette fiche **propose** (AC-1) de retirer la contrainte et de la
remplacer par `chk_bank_accounts_primary_has_iban` ; c'est un `DROP CONSTRAINT` (C-38-0), **dans la migration
breaking de cette story** (O1), avec l'analyse de compatibilité de l'AC-1. Un `CHECK` ne pouvant lire une autre table,
« IBAN obligatoire sauf compte prestataire » ne s'écrit pas au schéma : l'exigence d'IBAN d'un compte `bank` ordinaire
reste portée par `validate_bank_input` (garde d'application, testée à l'AC-4), le schéma ne gardant que le principal.

## Story

En tant qu'**Administrateur** d'une société qui tient un compte PayPal par solde de devise, je veux faire de chaque
solde un **compte bancaire Kesh ordinaire** — créé à neuf sans IBAN, ou un compte bancaire existant — **lié au compte
du plan déjà en devise** qui porte son historique (numéro, écritures et soldes gardés), avec ses réglages de
prestataire (connexion, devise du solde, compte de frais de cette devise), afin que les téléchargements et les
propositions de l'epic s'y appuient sans que ce compte devienne jamais compte principal, compte de lot de paiement ni
compte de l'assistant.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-2-1` | Pour un solde de devise, créer un compte bancaire `bank` sans IBAN avec ses réglages, ou poser les réglages sur un compte bancaire existant (qui ne porte encore aucune transaction, P2). | AC-3, AC-4 |
| `FR-38-2-2` | Lier ce compte à un compte du plan **déjà en devise**, en gardant numéro, historique et soldes (D8, levée de C-37-9 pour ce seul compte). | AC-3, AC-4, AC-6 |
| `FR-38-2-3` | Désigner le compte de frais de la devise (charge, actif, même devise) ; le regroupement est son compte parent. | AC-3, AC-5 |
| `FR-38-2-4` | Une seule devise par connexion (`PROVIDER_ACCOUNT_DUPLICATE_CURRENCY`) ; retrait des réglages refusé si le compte est utilisé (`PROVIDER_ACCOUNT_IN_USE`). | AC-3, AC-5 |
| `FR-38-2-5` | Un compte prestataire n'est jamais principal et n'entre ni aux lots pain.001 ni à l'assistant. | AC-7 |
| `FR-38-2-6` *(ajout)* | Le compte du plan d'un compte prestataire — en devise (O8) ou en CHF (P2) — n'est lié à aucun autre compte bancaire (`PROVIDER_ACCOUNT_LEDGER_SHARED`) : le contrôle de solde de 38-3b/38-10 compare un compte du plan à un solde du prestataire. | AC-3, AC-4 |
| `FR-38-2-7` *(ajout)* | La liste des comptes bancaires dit, pour chacun, s'il est compte prestataire (connexion, devise, compte de frais). | AC-8 |
| `FR-38-2-8` *(ajout, FR-38-1-4)* | La suppression d'une connexion qui porte un compte prestataire est refusée (`PROVIDER_CONNECTION_IN_USE`). | AC-9 |
| `NFR-38-2-1` | Gardes au serveur, sous le verrou de devise du compte du plan, dans la transaction du lien. | AC-3, AC-6 |
| `NFR-38-2-2` | Comptes désignés en colonnes FK `ON DELETE RESTRICT`, archivage refusé tant qu'un réglage les désigne (B23). | AC-1, AC-5 |
| `NFR-38-2-3` | **Migration breaking (O1)** : `kesh_version_min_required` relevé en dernière instruction, crates bumpés dans le même commit, retour par la sauvegarde d'avant la mise à jour ; `account_kind` toujours `'bank'`. | AC-1 |

## Critères d'acceptation

1. **AC-1 — Migration `<date>000002_payment_provider_accounts.sql`** (après celle de 38-1, date postérieure à
   `20261114000003` ; **breaking (O1)** ; DDL, plus l'`UPDATE` de version en dernière instruction ; en tête du fichier
   `-- BREAKING (P1, P3 : MODIFY COLUMN) : relève kesh_version_min_required à <X.Y.Z>`, patron de
   `20261110000001`). Ordre des instructions : (1) `CREATE TABLE payment_provider_accounts` ; (2) le remplacement de la
   contrainte d'IBAN ; (3) `ALTER TABLE reconciliation_rules MODIFY COLUMN match_type VARCHAR(40) CHARACTER SET ascii
   COLLATE ascii_bin NOT NULL, ADD CONSTRAINT chk_reconciliation_rules_match_type_format CHECK (BINARY match_type
   REGEXP '^[a-z][a-z_]{1,39}$')` — **exactement** cette définition (la colonne d'origine, `ENUM(…) NOT NULL` sans
   `DEFAULT` ni `COMMENT`, `migrations/20260513000001_reconciliation_rules.sql:40`, ne perd rien) ; format seul, sans
   énumération (O2) ; les quatre valeurs existantes (`counterparty_contains`, `counterparty_exact`,
   `reference_contains`, `iban_exact`) satisfont le format et sont conservées, aucune ligne réécrite ; l'index
   `uq_reconciliation_rules_match_active (company_id, match_type, active_uniq)` (`:65-66`) se **reconstruit à
   l'`ALTER`** : aucun `DROP INDEX` n'est requis, le nom et les colonnes restent (38-4 le remplace ensuite sous le même
   nom) ; la comparaison passe de `utf8mb4_unicode_ci` à `ascii_bin`, sans effet sur des valeurs en minuscules ASCII.
   **Test d'unicité après l'`ALTER`** (montée par version, P6) : deux règles actives de même société, type et valeur
   restent refusées (`RECONCILIATION_RULE_DUPLICATE`, `is_duplicate_rule_constraint` sur le nom), une valeur hors
   format (`'Iban_exact'`, `'x'`) est refusée par le `CHECK` ; (4) **en dernière instruction** `UPDATE _kesh_version SET
   kesh_version_min_required = '<X.Y.Z>' WHERE id = 1;` (P2). **Hors périmètre de cette story** : le décodage
   tolérant `Other(String)` de `ReconciliationMatchType` et de `BankImportSourceFormat` (E4) — tant qu'aucune valeur
   neuve n'est écrite, l'enum reste tel quel et la désérialisation de l'API refuse l'inconnu ; il vient avec la
   première écriture d'une valeur neuve : **38-3a** pour `PROVIDER_API`, **38-4** pour les critères (confirmé par
   l'orchestrateur, liste inter-fiches, point 1 ; 38-2-P1-3 close). Cette migration ne change que le DDL (et la
   version).
   - **Version figée `0.17.6`, à trier** : `grep -rn '0\.17\.6' crates docs CHANGELOG.md` (relevé au commit : `post_restore.rs`
     ×2, `migrations_fresh_install.rs` ×2, `migrations_upgrade_path.rs` ×16, `admin_full_import_e2e.rs` ×11, et des
     commentaires d'historique dans `routes/` et `tests/`) ; chaque occurrence triée — **assertion de la version
     minimale courante** (`migrations_fresh_install.rs:249`, `admin_full_import_e2e.rs:4671`, `:4818`,
     `migrations_upgrade_path.rs` qui lit le seuil après `MIGRATOR.run()` complet) → `<X.Y.Z>` ; **jalon historique**
     (seuil lu à la version `20261110000001`, sauvegarde 0.17.6 importée, commentaires datés) → inchangé ; le tri est
     cité au Change Log.
   - **P6, relevé au commit** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` —
     `migrations_upgrade_path.rs` ×27, `invoice_lines_revenue_account_backfill.rs` ×13, `lettering_documents_backfill.rs`
     ×6, `accounts_role_backfill.rs` ×5, `closing_accounts_backfill.rs` ×5, `pre_upgrade_backup/tests.rs` ×3, quatre
     autres ×2 ; chaque site inspecté, cité au Change Log.
   - **Note de version** (section de la fiche, assemblée au `CHANGELOG.md` à la release ; aucune ligne n'y est écrite
     en cours d'epic) : en tête, l'avertissement de la 0.17.6
     (`CHANGELOG.md:13`) repris — sauvegarde complète avant la mise à jour, **migration breaking**, version minimale
     relevée à `<X.Y.Z>`, la version précédente refuse de démarrer sur la base migrée et d'importer une sauvegarde
     `<X.Y.Z>` : **retour uniquement par la sauvegarde d'avant la mise à jour** ; puis le compte `bank` sans IBAN d'un
     prestataire. **Manuel d'administration** : sous-section *Passer à la `<X.Y.Z>`* écrite par 38-9a (AC-11 de
     38-9a, patron de *Passer à la 0.17.6*) — 38-2 lui transmet le texte de sa Note de version ; 38-2 n'écrit au manuel que la
     phrase de l'AC-10. **P2-bis** : les **dix** crates passent à `<X.Y.Z>` dans le même commit ; gate
   **runtime** complet (`admin_backup_e2e`, `admin_full_import_e2e`, `migrations_fresh_install`) avant `done` ;
   `<X.Y.Z>` est à Guy (Q1 de l'epic). **P3/P4** : pas de `breaking-skip-bump`. **P7** : l'`UPDATE _kesh_version`
   écrit une table système jamais restaurée → une entrée **`Durable`** à `EXEMPT_MIGRATIONS`
   (`crates/kesh-db/src/post_restore.rs:425`, patron de `20261110000001`, `:645`) et le décompte de
   `every_exemption_declares_a_coherent_basis` (24 au relevé, `post_restore.rs:1252` : +1, cumulé) ; les autres instructions sont du DDL. **Retour** :
   un binaire antérieur refuse de démarrer sur la base migrée et d'importer un `.keshbackup` de cette version ; **le
   retour à la version précédente passe uniquement par la sauvegarde d'avant la mise à jour** (`backup/pre-upgrade/`,
   #847, et celle que les notes exigent) — à écrire en tête des notes de version et au manuel d'administration
   (38-9a) ; recette de montée `scripts/recette-montee.sh` depuis l'image publiée précédente, jouée ici et à la
   release (`docs/testing.md`). Test de montée au patron de `bank_accounts_without_iban` (`migrations_upgrade_path.rs`),
   index résolus **par version** (P6) : règles existantes inchangées après la montée, unicité encore tenue, un compte
   `bank` sans IBAN non principal accepté et un principal sans IBAN refusé.
   - **`payment_provider_accounts`** (table **applicative**, E11, C-38-3) : `bank_account_id BIGINT NOT NULL PRIMARY
     KEY` (FK `bank_accounts`, `RESTRICT`), `company_id BIGINT NOT NULL` (FK `companies`, `RESTRICT` ; égal à celui du
     compte bancaire et de la connexion, contrôlé par le dépôt), `connection_id BIGINT NOT NULL` (FK
     `payment_provider_connections`, `RESTRICT` — c'est elle qui fait refuser la suppression d'une connexion, AC-9),
     `currency CHAR(3) CHARACTER SET ascii COLLATE ascii_bin NOT NULL` (même définition et même `CHECK BINARY currency
     REGEXP '^[A-Z]{3}$'` qu'`accounts.currency` de 37-1 AC-1 ; **CHF admis** : le solde CHF d'un compte PayPal est un
     compte prestataire comme les autres), `fee_account_id BIGINT NULL` (FK `accounts`, `RESTRICT` ; `NULL` = à
     désigner, la proposition le refuse alors en 38-6a, `FEE_ACCOUNT_INVALID`), `version INT NOT NULL DEFAULT 1`,
     `created_at`, `updated_at` `DATETIME(3) NOT NULL` ; `UNIQUE uq_ppa_connection_currency (connection_id,
     currency)` (B1 : un compte par solde et par devise). Colonnes `NULL` de comptes liés d'autres prestataires (p. ex.
     compte intermédiaire de Stripe) : **ajoutées par leur epic**, en FK `RESTRICT` (B23) — aucune ici.
   - **Contrainte de `bank_accounts` (Q-38-2-a, close par O1)** : `ALTER TABLE bank_accounts DROP CONSTRAINT
     chk_bank_accounts_bank_has_iban, ADD CONSTRAINT chk_bank_accounts_primary_has_iban CHECK (is_primary = FALSE OR
     iban IS NOT NULL)`. *Analyse de compatibilité* : la colonne `iban` est déjà `NULL` et lue `Option<String>` depuis
     la 0.17.6 (`entities/bank_account.rs:126-143`) ; les sites qui exigent un IBAN refusent **sur `iban.is_none()`,
     jamais sur le type** (C-e341-18 : lot pain.001, `repositories/payment_batches.rs:170-185, 884-886` ; import CAMT ;
     QR-facture, `routes/invoice_pdf_service.rs:342-353`, qui lit le **principal**, désormais gardé par la contrainte
     neuve) ; un binaire antérieur qui lirait un compte `bank` sans IBAN le refuserait en `400` à la modification
     (`validate_bank_input`), sans corruption — et, la version minimale étant relevée par ce même fichier, un binaire antérieur ne démarre de
     toute façon pas sur la base migrée. Les lignes existantes satisfont la contrainte neuve (un `bank` a un
     IBAN, une carte n'est jamais principale). Le schéma **ne garantit plus** qu'un compte `bank` non principal porte
     un IBAN : l'invariant passe à `validate_bank_input` et au dépôt (`create`/`PUT`), à tester (le `POST` et le `PUT`
     d'un compte ordinaire sans IBAN restent refusés).
   - **Sites à reprendre** (le changement d'invariant, greppé : `grep -rn "bank_has_iban" crates docs`) :
     `entities/bank_account.rs:126-143` (doc de `iban` : le schéma ne garantit plus `Some` pour `Bank` ; garantit `Some`
     pour le principal), `repositories/bank_accounts.rs:657-663` (`BankAccountPutFields`), `repositories/payment_batches.rs:157-162`
     (commentaire : « l'IBAN d'un compte ne devient jamais nul » **devient faux** — un `PUT` d'un compte prestataire posé
     en `attach` avec IBAN peut l'omettre et le passer à `NULL`, AC-4 ; reformulé : « hors compte prestataire, qui est
     refusé sous verrou, AC-7 »), `routes/invoice_pdf_service.rs:343-346`
     (contrainte citée), tests `crates/kesh-db/tests/bank_accounts_repository.rs:2990-2996`
     (`sans_iban_les_invariants_du_schema` : le cas `bank` sans IBAN attend désormais un **succès** hors principal, et
     un refus `chk_bank_accounts_primary_has_iban` en principal) et `crates/kesh-db/tests/migrations_upgrade_path.rs:2500-2550`
     (montée de 34-11a1 : après `MIGRATOR.run()` complet, le refus attendu par `chk_bank_accounts_bank_has_iban`
     n'existe plus — l'assertion porte désormais sur l'état à la version `20261110000001`, résolue **par version**
     (P6), ou devient le refus de la contrainte neuve en principal), et le **squash** `crates/kesh-db/test-schema/
     0001_schema_squash.sql` (`:176` pour la contrainte, `:1167` pour `match_type`, `:1180` pour l'index ; garde
     `test_schema_guard.rs`).
   - **P5** : ligne d'audit d'idempotence (`tracked-by-sqlx` ; `CREATE TABLE`, `ALTER TABLE … DROP/ADD CONSTRAINT`
     et `MODIFY COLUMN` sans `IF EXISTS` : re-exécution hors sqlx → erreur 1050 ; l'`UPDATE` de version est
     idempotent) ; total (**deux** sites) et partition recomptés à la source. **P6** : `grep -rn "migrations.len()\|
     apply_migrations_up_to" crates/` exécuté, sites inspectés et cités (le test de 34-11a1 ci-dessus en est un).
     **P8** : sans objet (fichier neuf).
   - **Liste close des absences admises** (37-1 AC-6, E18) : `payment_provider_accounts` en `Absence::Table` ; test
     d'import d'une sauvegarde d'avant l'Epic 38 étendu (aucun compte prestataire). ⚠️ Le `DROP CONSTRAINT` ne change
     rien à l'import d'une sauvegarde antérieure (ses lignes satisfont les deux contraintes).
   - **Inscriptions** : `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/backup.rs:46`) **avant**
     `payment_provider_connections` (placée par 38-1), donc avant `bank_accounts` (:104) et `accounts` (:105) — enfant
     des trois ; absente de `RESET_PRESERVED_TABLES` ; `reset_partition_is_guarded_by_the_schema` reste vert (rejoint
     `companies`). Export CSV : `TABLES_EXPORTEES` (`crates/kesh-api/src/exports/global.rs:116`), fichier
     `payment_provider_accounts.csv` (sérialiseur écrit à la main au patron de `serialize_bank_accounts_csv`,
     `csv_tables.rs:640`) : `bank_account_id`, `connection_id`, `provider` et `connection_label` (lus par jointure : la
     table des connexions est hors export, 38-1 AC-2), `currency`, `fee_account_id`, `fee_account_number` ; cas de test
     de l'en-tête contre le schéma. `account_kind` reste `'bank'` partout (aucune valeur neuve, B16).
   - **Sites de dénombrement** (même grep que 38-1) : `admin_full_export_e2e.rs:~281` (`data_count == 44` : +1,
     cumulé avec 38-1 et l'Epic 37) ; `onboarding_audit_e2e.rs:2046` (`EMPTY_BEFORE_RESET`, liste fermée : la table y
     entre) ; le décompte de `TABLES_EXPORTEES` que `exports_global_e2e.rs` contrôle fichier par fichier
     (`payment_provider_accounts.csv` : 0 ligne).

2. **AC-2 — Dépôt `kesh-db::repositories::payment_provider_accounts`** : `list_by_company`, `list_by_connection`,
   `find_by_bank_account_in_tx` (verrou au choix de l'appelant), `insert_in_tx`, `update_fee_account_in_tx` (version),
   `delete_in_tx` (version), `is_provider_account_in_tx(bank_account_id)` (lecture `LOCK IN SHARE MODE`, utilisée par
   les gardes de l'AC-6 et de l'AC-7) ; entité `PaymentProviderAccount`. **Ordre des verrous** (à écrire en tête du
   module et vérifié contre celui des comptes bancaires, `repositories/bank_accounts.rs:546-553`, et contre les
   verrous de devise de 37-1, C-37-5) : sentinelle `companies` `FOR UPDATE` (`acquire_company_sentinel_lock`, :1725)
   → `payment_provider_connections` (ligne, `FOR UPDATE`) → comptes de créance des réglages en partagé (comme la
   création d'un compte bancaire, `routes/bank_accounts.rs:700-715, 1075-1085`) → `accounts` visés (compte lié, compte de frais)
   `LOCK IN SHARE MODE`, **par `id` croissant** → `bank_accounts` (ligne, `FOR UPDATE`) → `payment_provider_accounts`.
   Toute route de cette story est enveloppée de `retry_app_on_deadlock` (`crates/kesh-api/src/retry.rs:40`), comme
   `bank_accounts::patch` (`repositories/bank_accounts.rs:546-553` en décrit les cycles).

3. **AC-3 — `POST /api/v1/provider-connections/{id}/accounts`** (Administrateur, session seule : `admin_routes`, comme
   les routes de 38-1 ; onboarding terminé, `assert_onboarding_complete` comme `create_bank_account`,
   `routes/bank_accounts.rs:969`). Deux formes, champ `mode` :
   - **`"create"`** — `{ mode, currency, bankName, description?, journalAccountId, feeAccountId? }` : crée un compte
     bancaire `account_kind = 'bank'`, **sans IBAN ni QR-IBAN**, `is_primary = FALSE`, `bankName` et `description`
     contrôlés par les fonctions existantes (`refuse_too_long_bank_name`, `normalize_description`, refus d'un numéro de
     carte, `routes/bank_accounts.rs:495-526, 539-555`), puis sa ligne `payment_provider_accounts`, **dans la même
     transaction** (audits `bank_account.created` et `payment_provider_account.created`). `iban`, `qrIban`, `isPrimary`,
     `cardLast4`, `accountKind`, `openingBalance` dans le corps → `400 VALIDATION_ERROR`, `details.field` (le corps est
     `deny_unknown_fields` ; un solde d'ouverture en devise n'est pas l'objet de cette route : le compte du plan porte
     déjà son historique, D8).
   - **`"attach"`** — `{ mode, bankAccountId, currency, journalAccountId?, feeAccountId? }` : pose les réglages sur un
     compte bancaire **existant** de la société (`bank`, non archivé, non clos — sinon `409 BANK_ACCOUNT_CLOSED` ou
     `404`), **sans transaction** (ci-dessous), en gardant son id, son nom et son IBAN éventuel ; `journalAccountId`
     présent remplace le lien dans la même transaction, absent garde le lien courant. ⛔ **Compte qui porte au moins
     une transaction bancaire** (`count_transactions_for_bank_account`, `repositories/bank_accounts.rs:1544`, lu sous
     le verrou de la ligne) → `409 PROVIDER_ACCOUNT_HAS_TRANSACTIONS`, `details.transactionCount`, rien n'est posé :
     ses transactions de fichier (CAMT, CSV, relevé saisi) n'ont pas de `provider_tx_id`, et le premier passage (38-3b,
     depuis `fetch_from`) retéléchargerait les mêmes mouvements sans que E6 les reconnaisse — doublons muets, deux
     propositions pour un mouvement ; et ces transactions sont en CHF (aucun import n'en enregistre d'autre depuis
     27-2, `routes/reconciliation.rs:1097-1100`) quand la devise du solde ne l'est pas. Le compte du plan, lui, garde son
     historique (D8) : un compte neuf (`create`) s'y lie. Carte de crédit → `400
     PROVIDER_ACCOUNT_KIND_INVALID` (une carte n'est pas un solde de prestataire). Compte principal → `409
     PROVIDER_ACCOUNT_CANNOT_BE_PRIMARY` (rien n'est posé ; retirer d'abord le statut principal). Compte déjà porteur de
     réglages → `409 PROVIDER_ACCOUNT_DUPLICATE_CURRENCY` s'il s'agit de la même connexion et devise, sinon `409
     PROVIDER_ACCOUNT_ALREADY_ATTACHED` (`details.connectionId`).
   - **Gardes communes**, au serveur, **dans la transaction**, sous les verrous de l'AC-2, dans cet ordre (une seule
     raison, la plus bloquante) : connexion de la société (`404`) et prestataire connu (`409 PROVIDER_UNKNOWN`) →
     `currency` (normalisée comme 37-1 AC-4 : `trim`, majuscules ASCII, liste ISO 4217 embarquée → `400
     CURRENCY_UNKNOWN`) → société multi-devises si `currency ≠ CHF` (`409 MULTI_CURRENCY_DISABLED`, 37-1) → devise déjà
     tenue par la connexion → `409 PROVIDER_ACCOUNT_DUPLICATE_CURRENCY` (`details.bankAccountId` du compte qui la porte ;
     contrainte `uq_ppa_connection_currency` en filet, erreur 1062 mappée par son nom) → **compte du plan lié**
     (`journalAccountId` ; exigé en `create`, et en `attach` si le compte bancaire n'en a pas — `400
     PROVIDER_ACCOUNT_LEDGER_REQUIRED`) : de la société, actif, imputable, **de bilan** (actif seulement : un solde de
     prestataire est un avoir — `400` avec le refus de type existant, `ledger_type_refusal`) et **dans la devise du
     solde** — CHF : sans devise (`accounts.currency IS NULL`) ; autre : `accounts.currency = currency` — sinon `400
     PROVIDER_ACCOUNT_CURRENCY_MISMATCH`, `details` `{ accountId, accountCurrency, expectedCurrency }` → compte du plan
     déjà lié à un **autre** compte bancaire (quel que soit son état), **en devise ou en CHF** → `409
     PROVIDER_ACCOUNT_LEDGER_SHARED` (`details.bankAccountId` ; FR-38-2-6 — le contrôle de solde compare un compte du
     plan à un solde du prestataire en CHF comme en devise ; O8 ne nomme que la devise, où C-37-9 ferme déjà le sens
     inverse : en CHF, rien ne le ferme sans cette garde) → compte de créance des réglages → refus
     existant `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` (`repositories/bank_accounts.rs:1699`) → **compte de frais**
     (`feeAccountId`, facultatif) : AC-5.
   - **Levée de C-37-9 (D8)** : le refus `ACCOUNT_IN_FOREIGN_CURRENCY` que 37-1 AC-4 pose sur le lien d'un compte
     bancaire à un compte en devise **ne s'applique pas** quand la ligne `payment_provider_accounts` de **même devise**
     est posée dans la même transaction (cette route) ; le compte du plan **garde** son numéro, ses lignes et ses soldes
     (aucune écriture). Le `account_kind` reste `'bank'`.
   - Réponse `201` : le compte bancaire (forme de la liste, AC-8) avec son bloc `provider`.

4. **AC-4 — Routes existantes des comptes bancaires sur un compte prestataire.**
   - **`PATCH /api/v1/bank-accounts/{id}`** (lien, `patch_bank_account_journal_link`, `routes/bank_accounts.rs:2032`) et
     **`PUT /api/v1/bank-accounts/{id}`** (`update_bank_account`, :1383) : pour un compte **porteur de réglages**
     (`is_provider_account_in_tx`, lu sous le verrou de la ligne), le refus `ACCOUNT_IN_FOREIGN_CURRENCY` de 37-1 est
     **remplacé** par la garde **entière** du compte du plan lié de l'AC-3, appelée par la **même fonction** (une seule
     définition) : actif seulement — `BankAccountKind::Bank` admet aussi un passif (`entities/bank_account.rs:50`),
     qu'un compte prestataire refuse —, imputable, `PROVIDER_ACCOUNT_CURRENCY_MISMATCH` si la devise du compte visé
     n'est pas celle des réglages, `PROVIDER_ACCOUNT_LEDGER_SHARED`, compte de créance ; **délier** (`journalAccountId:
     null`) un compte prestataire → `409 PROVIDER_ACCOUNT_LEDGER_REQUIRED` s'il porte au moins une transaction
     (`count_transactions_for_bank_account`, lue sous le verrou de la ligne ; arbitrage O8 : « délier n'est admis que
     s'il ne porte aucune transaction téléchargée » — les soldes rapportés n'y comptent pas, O8 ne les nomme pas) ;
     sans transaction, le lien est retiré et le compte reste « lien à désigner » (comme `fee_account_id NULL`), sans
     proposition possible d'ici là. Pour un compte **sans** réglages, rien ne change (37-1), **hormis** le sens inverse
     de FR-38-2-6 : le lier (`POST`, `PUT`, `PATCH`) au compte du plan d'un compte prestataire → `409
     PROVIDER_ACCOUNT_LEDGER_SHARED` (en devise, C-37-9 refuse déjà ; en CHF, cette garde est la seule), lu par la
     lecture des comptes bancaires liés à ce compte du plan (`bank_accounts` sur `journal_account_id`, `LOCK IN SHARE
     MODE`, patron `first_active_bank_account_linked_to`, `repositories/bank_accounts.rs:1670-1676`, **sans** son filtre
     `archived = FALSE`) jointe à `payment_provider_accounts` ; test pour chacun des trois chemins.
   - **`PUT`** d'un compte prestataire : `iban` **facultatif** (absent ou `null` → `NULL` ; présent → contrôlé comme
     aujourd'hui) — c'est la seule dérogation à `validate_bank_input` (`:557-570`), décidée sur
     `is_provider_account` **lu sous verrou** et non sur le corps ; `qrIban` présent → `400
     PROVIDER_ACCOUNT_FIELD_NOT_ALLOWED` (`details.field`) ; `isPrimary: true` → `409
     PROVIDER_ACCOUNT_CANNOT_BE_PRIMARY`. Un compte prestataire **avec** IBAN (posé en `attach`) le garde tant que le
     `PUT` le renvoie (remplacement complet, règle d'aujourd'hui).
   - **`POST /api/v1/bank-accounts`** ne crée jamais de compte prestataire (il n'a pas de champ pour cela) et continue
     d'exiger l'IBAN d'un `bank` : la seule voie d'un compte `bank` sans IBAN est l'AC-3.
   - **`DELETE /api/v1/bank-accounts/{id}`** (archivage) et `…/close` : admis sur un compte prestataire aux conditions
     d'aujourd'hui (archivage refusé s'il a des transactions : gardes « fonds » d'`archive_once`,
     `routes/bank_accounts.rs:1645-1660`, la fonction du dépôt `archive_for_company`, `repositories/bank_accounts.rs:806`,
     ne faisant que l'`UPDATE`) ; la ligne de réglages **reste** (l'historique
     du lien est gardé) ; 38-3b ne télécharge pas pour un compte archivé ou clos (point de contact). Limite écrite au
     manuel (38-9a) : la devise reste tenue par la connexion tant que la ligne existe (`uq_ppa_connection_currency`).

5. **AC-5 — Compte de frais, modification et retrait des réglages (FR-38-2-3, FR-38-2-4, NFR-38-2-2).**
   - **Garde du compte de frais** (`feeAccountId`, à la création, en `attach` et au `PATCH`) : de la société, **actif**,
     **imputable**, de type **`Expense`**, dans la **devise du solde** (CHF : sans devise) — sinon `400
     FEE_ACCOUNT_INVALID`, `details` `{ accountId, reason }`, `reason` ∈ `not_found`, `archived`, `not_postable`,
     `not_expense`, `currency_mismatch` ; plus **`missing`** (`details.accountId` nul : aucun compte de frais désigné,
     `fee_account_id NULL`), **posée par 38-6a** à la proposition — le code et la liste des raisons sont fixés ici une
     fois pour les deux stories. Le **regroupement** des frais est son compte parent (`accounts.parent_id`) :
     aucun réglage de plus (B3). Deux connexions peuvent désigner le même compte de frais ? **Oui** (rien ne
     l'interdit ; B3 recommande un sous-compte par prestataire et par devise, ce que l'écran 38-9a suggère sans
     l'imposer).
   - **`PATCH /api/v1/provider-connections/{id}/accounts/{bankAccountId}`**, `{ version, feeAccountId }` : seul
     `feeAccountId` se modifie (`null` admis : à désigner) ; `currency` **immuable** (présente → `400
     VALIDATION_ERROR`) ; version périmée → `409 OPTIMISTIC_LOCK_CONFLICT` ; no-op sans audit. Audit
     `payment_provider_account.updated`.
   - **`DELETE /api/v1/provider-connections/{id}/accounts/{bankAccountId}`**, `{ version }` : retire la ligne de
     réglages — le compte bancaire **reste**, ordinaire. Refus `409 PROVIDER_ACCOUNT_IN_USE`, `details.reasons` (liste
     **complète**, triée) ∈ `foreign_currency_link` (compte lié en devise : il deviendrait un compte ordinaire lié à un
     compte en devise, contraire à C-37-9), `transactions` (au moins une `bank_transactions` du compte,
     `count_transactions_for_bank_account`, `repositories/bank_accounts.rs:1544`), `no_iban` (le compte deviendrait un
     `bank` sans IBAN hors prestataire — **sauf s'il est archivé** : un compte archivé est figé, hors des listes, aucun
     `PUT` ni `PATCH` ne l'atteint, `repositories/bank_accounts.rs:570, 730` ; sans cette exception, un compte créé
     par erreur (`create`, donc sans IBAN) ne perdrait jamais ses réglages, sa devise resterait prise par
     `uq_ppa_connection_currency` et la connexion ne se supprimerait plus. Chemin écrit et testé : délier s'il est en
     devise (AC-4, sans transaction), archiver, retirer les réglages, supprimer la connexion), et **`balance_seen`** — au moins une ligne `provider_balance_snapshots`
     (`opening` ou `reported`, instantanés compris) de la connexion dans la devise du compte —, **posée par 38-3a**
     (AC-4), qui crée la table ; le code et ses quatre messages listent les quatre raisons dès cette story. Audit
     `payment_provider_account.deleted`.
   - **Archivage d'un compte du plan désigné** (`archive_guarded`, `crates/kesh-db/src/repositories/accounts.rs:859`) :
     compte de frais d'une ligne de réglages, ou compte lié d'un compte prestataire → `409
     ACCOUNT_USED_BY_PROVIDER_ACCOUNT`, `details` `{ bankAccountIds, role }` (`role` ∈ `fee`, `ledger`), **sans
     confirmation possible** (à la différence d'`ACCOUNT_DESIGNATED_IN_SETTINGS`, `:893-899`, qui se confirme) ; la FK
     `RESTRICT` ne suffit pas, l'archivage n'étant qu'un `UPDATE active = FALSE`. Changement de devise d'un compte ainsi
     désigné → `409 ACCOUNT_CURRENCY_IN_USE` de 37-1 AC-4, site `payment_provider_accounts` en `details` (la liste
     **close** des sites de 37-1 gagne une entrée : `payment_provider_accounts.fee_account_id`, vingtième `REFERENCES
     accounts` — `grep -rn "REFERENCES accounts" crates/kesh-db/migrations/*.sql` en rend **19** au commit de relevé —,
     et le lien d'un compte prestataire, qui **exige** une devise au lieu de l'interdire). Retypage d'un compte de frais
     hors `Expense` → même refus (`details.reason = "fee_account_type"`).

6. **AC-6 — Concurrence (NFR-38-2-1).** Tests de course, au patron des tests de sérialisation du dépôt des comptes
   bancaires (attente sur le texte `LOCK IN SHARE MODE`, `repositories/bank_accounts.rs:1665-1669`) : (a) lien d'un
   compte prestataire EUR contre changement de devise du compte du plan (37-1) — l'un attend l'autre, jamais un compte
   prestataire EUR lié à un compte USD ; (b) deux `POST …/accounts` de même devise sur la même connexion — un `201`, un
   `409 PROVIDER_ACCOUNT_DUPLICATE_CURRENCY` ; (c) `DELETE` des réglages contre arrivée d'une transaction (import CSV
   sur le compte) — le retrait voit la transaction ou l'import attend ; ⚠️ 38-3a refuse cet import sur un compte
   prestataire (AC-6 de 38-3a) : la course (c) y change de sens (l'import voit les réglages et refuse, ou passe avant
   leur retrait) et 38-3b la rejoue contre l'insertion d'un passage ; (d) archivage du compte de frais contre `PATCH`
   qui le désigne ; (e) `attach` contre import d'une transaction sur le même compte — l'`attach` voit la transaction
   (`PROVIDER_ACCOUNT_HAS_TRANSACTIONS`) ou l'import attend.

7. **AC-7 — Exclusions (FR-38-2-5).**
   - **Compte principal** : jamais (AC-3, AC-4), et `chk_bank_accounts_primary_has_iban` en filet pour un compte sans
     IBAN ; la QR-facture lit le principal (`routes/invoice_pdf_service.rs:332-353`), inchangée.
   - **Lots de paiement pain.001** : `create_batch` (`repositories/payment_batches.rs:158-185`) refuse un compte
     prestataire **même avec IBAN** → nouvelle variante `DbError::PaymentBatchProviderAccount { bank_account_id }` →
     `409 PAYMENT_BATCH_ACCOUNT_IS_PROVIDER`, lu par `is_provider_account_in_tx` **après la revérification
     verrouillante** de la ligne (`recheck_open_unarchived_locked_in_tx`, `LOCK IN SHARE MODE`, appelée après
     l'`INSERT` du lot, `payment_batches.rs:236-248`) — ⚠️ la lecture de la ligne à `:165` est **non verrouillante**
     (commentaire `:150-156`) : y lire les réglages laisserait passer une pose de réglages commitée entre-temps ; le
     refus précoce à `:165` peut s'y ajouter, la décision reste sous verrou (`attach` tient la ligne `FOR UPDATE`, AC-2) ;
     sans IBAN, le refus existant `PaymentBatchAccountHasNoIban` (:184) tombe le premier — ordre écrit, test des deux.
     **Génération du `pain.001`** d'un lot créé avant la pose des réglages (`payment_batches.rs:875-886`) : même refus,
     lu au même endroit que l'IBAN ; sans lui, un `PUT` qui retire l'IBAN du compte devenu prestataire (AC-4) ne
     rendrait que `PaymentBatchAccountHasNoIban`. L'écran de création d'un lot (38-9a) n'offre pas ces comptes.
   - **Assistant d'installation** (`routes/onboarding.rs:612-700`, `upsert_primary_in_tx`) : il ne crée ou ne modifie que
     le compte **principal**, qu'un compte prestataire ne peut pas être ; les routes de cette story exigent l'onboarding
     terminé. Test : `set_bank_account` après la pose de réglages sur un autre compte ne touche pas ce dernier.
   - **Import CAMT** : sélection par IBAN ; un compte PayPal sans IBAN n'y est jamais choisi (rien à changer) ; un
     compte prestataire **avec** IBAN : l'import CAMT reste admis en 38-2 — **point de contact 38-3a** (AC-6), qui refuse les
     imports de fichier sur un compte prestataire (arbitrage O9 : mélange de `provider_tx_id` et de clés de doublon
     par fenêtre) ; 38-2 ne pose pas ce refus.

8. **AC-8 — Lecture (FR-38-2-7).** `GET /api/v1/bank-accounts` (`list_bank_accounts`, :883, tous rôles) : chaque
   élément gagne `provider` — `null` pour un compte ordinaire, sinon `{ connectionId, connectionLabel, provider,
   currency, feeAccountId, feeAccountNumber, version }` (une jointure de plus dans `list_by_company_with_balances`,
   `repositories/bank_accounts.rs:1834`, sans N+1) ; `#[serde(skip_serializing_if = "Option::is_none")]` pour que le
   JSON d'un compte ordinaire reste identique (patron C-37-9 de 37-1 AC-4). `GET
   /api/v1/provider-connections/{id}/accounts` (Administrateur) : les comptes de la connexion, même forme. Type TS
   mis à jour (`frontend/src/lib/features/bank-accounts/…` : le champ seul, sans écran — 38-9a).

9. **AC-9 — Suppression d'une connexion portant un compte (FR-38-1-4, C-38-2).** Le mappage posé en 38-1 (AC-5) est
   exercé ici : `DELETE /api/v1/provider-connections/{id}` d'une connexion portant une ligne `payment_provider_accounts`
   → `409 PROVIDER_CONNECTION_IN_USE`, `details.bankAccountIds` (lus sous le verrou de la connexion, `FOR UPDATE`,
   avant la suppression — la FK `RESTRICT` reste le filet, son erreur 1451 mappée sur le même code) ; après retrait des
   réglages (AC-5), la suppression passe.

10. **AC-10 — Registres, audit, i18n, MCP, manuel.**
    - **Routes** : `POST|GET /api/v1/provider-connections/{id}/accounts`, `PATCH|DELETE
      /api/v1/provider-connections/{id}/accounts/{bankAccountId}` dans `admin_routes` (avant ses `route_layer`,
      `lib.rs:513-520`) ; `LIB_ROUTES` **+3** mutantes (`post` create/attach, `patch`, `delete`), toutes `Traced,
      SansEcritureAuJournal` ; décomptes de `the_registry_partition_is_what_the_story_declares` en delta (+3, +3
      tracées), cumulés avec 38-1.
    - **Audit** : type d'entité `payment_provider_account` ; actions `payment_provider_account.created`, `.updated`,
      `.deleted` ; détail `{ bankAccountId, connectionId, currency, feeAccountId }` avant/après ; libellés aux quatre
      locales ; `audit_label_registry` vert.
    - **Codes neufs** (`AppError`, messages aux quatre locales, `crates/kesh-i18n/locales/*/messages.ftl`) :
      `PROVIDER_ACCOUNT_CURRENCY_MISMATCH`, `PROVIDER_ACCOUNT_DUPLICATE_CURRENCY`, `PROVIDER_ACCOUNT_ALREADY_ATTACHED`,
      `PROVIDER_ACCOUNT_LEDGER_REQUIRED`, `PROVIDER_ACCOUNT_LEDGER_SHARED`, `PROVIDER_ACCOUNT_KIND_INVALID`,
      `PROVIDER_ACCOUNT_FIELD_NOT_ALLOWED`, `PROVIDER_ACCOUNT_CANNOT_BE_PRIMARY`, `PROVIDER_ACCOUNT_IN_USE`,
      `PROVIDER_ACCOUNT_HAS_TRANSACTIONS` (P2),
      `FEE_ACCOUNT_INVALID` (repris par 38-6a, C-38-8), `ACCOUNT_USED_BY_PROVIDER_ACCOUNT`,
      `PAYMENT_BATCH_ACCOUNT_IS_PROVIDER`. Les cinq de C-38-3 y sont sous leur nom exact.
    - **MCP** : aucun outil ne liste les comptes bancaires (`crates/kesh-api/src/mcp/outils/`, relevé : `balance_agee`,
      `contacts`, `ecritures`, `exercices`, `factures`, `factures_fournisseur`, `grand_livre`, `kesh_version`, `pieces`,
      `plan_comptable`, `postes_ouverts`, `rapports`) ; E23 : rien à ajouter.
    - **Manuel** : la section des comptes prestataires est écrite en 38-9a avec les écrans ; 38-2 ajoute au manuel
      d'administration la phrase sur l'invariant changé (« un compte bancaire sans IBAN est un compte de prestataire ;
      le compte principal a toujours un IBAN ») si une section décrit `chk_bank_accounts_bank_has_iban` ou l'IBAN
      obligatoire (grep du manuel et du PDF aplati : à faire, résultat au Change Log).

## Tâches / sous-tâches

- [ ] **T0 — Numéro de release `<X.Y.Z>` obtenu de Guy** (Q1 de l'epic), écrit au Change Log, **avant toute ligne de
      code** (la migration le fige, P2) ; l'arbitrage Q-38-2-a est acquis (O1).
- [ ] **T1 — Migration** (AC-1) : fichier, squash, audit d'idempotence et compteurs, P6 (dont
      `migrations_upgrade_path.rs:2500-2550`), inscriptions (`TABLES_TO_TRUNCATE`, `TABLES_EXPORTEES` et sérialiseur,
      liste close R1 et son test), sites de l'invariant d'IBAN, `match_type` en `VARCHAR(40)`, `UPDATE
      _kesh_version`, dix crates à `<X.Y.Z>` dans le même commit, `EXEMPT_MIGRATIONS` (+1), sites de dénombrement
      (`data_count`, `EMPTY_BEFORE_RESET`), recette de montée, gate runtime (`admin_backup_e2e`,
      `admin_full_import_e2e`, `migrations_fresh_install`).
- [ ] **T2 — Dépôt** (AC-2) : entité, fonctions, ordre des verrous écrit et testé.
- [ ] **T3 — Routes prestataire** (AC-3, AC-5, AC-9) : `create`/`attach`, `PATCH`, `DELETE`, `GET`, gardes dans
      l'ordre, enveloppe de rejeu, codes et messages.
- [ ] **T4 — Routes des comptes bancaires** (AC-4, AC-8) : dérogation d'IBAN sur verrou, levée de C-37-9 pour le seul
      compte prestataire, garde entière du compte lié (fonction de l'AC-3), `LEDGER_SHARED` dans les deux sens, refus
      de délier, bloc `provider` de la liste, type TS. `attach` d'un compte à transactions refusé (T3).
- [ ] **T5 — Comptes du plan** (AC-5) : archivage et changement de devise d'un compte désigné, retypage du compte de
      frais ; inventaire des sites de 37-1 complété.
- [ ] **T6 — Exclusions** (AC-7) : lot pain.001, tests de l'assistant et du principal.
- [ ] **T7 — Concurrence** (AC-6).
- [ ] **T8 — Registres** (AC-10) : `LIB_ROUTES` +3, audit, i18n quatre locales, manuel.
- [ ] **T9 — Gates** : gate complet (migration, dépôts : ciblage interdit), base remise à zéro avant ; E2E complet au
      dernier commit de code (aucun écran neuf, mais les comptes bancaires sont lus par l'accueil, les paramètres et
      les lots : le JSON inchangé d'un compte ordinaire se vérifie là).

## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release.)*

- **Migration breaking** : sauvegarde complète avant la mise à jour ; version minimale relevée à `<X.Y.Z>` ; la version
  précédente refuse de démarrer sur la base migrée et d'importer une sauvegarde `<X.Y.Z>` : **retour uniquement par la
  sauvegarde d'avant la mise à jour**.
- **Ajouté** : le compte bancaire d'un prestataire de paiement (PayPal), sans IBAN, lié à un compte comptable.

**Compteurs** (audit des migrations — total aux deux sites et partition —, `Cargo.lock`, nombre de crates, sites du
grep P6, `data_count`, registres) : recomptés depuis la source après chaque reprise de `main`.

## Notes de développement

- **D8 tient en une phrase** : *un compte bancaire porteur de réglages de prestataire peut être lié à un compte du
  plan en devise de même devise ; tout autre compte bancaire reste soumis à C-37-9.* Toute garde de cette story se
  décide sur la ligne `payment_provider_accounts` **lue sous verrou**, jamais sur un champ du corps.
- **Pourquoi pas `account_kind = 'provider'`** : B16, maintenu par B22 — `BankAccountKind::from_str` refuse l'inconnu
  (`entities/bank_account.rs:61-69`), si bien qu'une valeur neuve serait une migration breaking de plus ; la seule
  breaking de l'epic est celle de cette story (O1). Le prix est le remplacement de `chk_bank_accounts_bank_has_iban`
  (en tête).
- **Soldes** : `currentBalance` de la liste reste calculé comme aujourd'hui (en devise principale) ; l'affichage en
  devise d'un compte en devise est l'affaire de 37-4. Le contrôle « solde du prestataire = solde comptable » est 38-3b
  (complétude) et 38-10 (relevé).
- **Archivage d'un compte prestataire** : la ligne de réglages reste et garde sa devise prise ; recréer un compte de
  même devise pour la même connexion demande le retrait des réglages, refusé s'il a des transactions ou un solde vu
  (`no_iban` ne vaut pas pour un compte archivé, AC-5). Limite écrite (38-9a) ; une issue si Beat la juge gênante.
- **Références** : epic E11, C-38-3, D8 (§ *Dépendance à l'Epic 37*) ; 37-1 AC-4 (refus `ACCOUNT_IN_FOREIGN_CURRENCY`,
  `ACCOUNT_CURRENCY_IN_USE`, liste close des sites) et AC-6 ; `migrations/20261110000001_bank_accounts_without_iban.sql:39-55`
  ; `entities/bank_account.rs:21-69, 126-143` ; `routes/bank_accounts.rs:205, 480-526, 539-637, 641-685, 700-715, 883, 964-1000,
  1383, 1600, 2032` ; `repositories/bank_accounts.rs:546-553, 657-663, 806, 1544, 1665-1669, 1699, 1725, 1834` ;
  `repositories/accounts.rs:859-899` ; `repositories/payment_batches.rs:158-185, 875-886` ;
  `routes/invoice_pdf_service.rs:332-353` ; `routes/onboarding.rs:612-700` ; `backup.rs:46, 99-105, 137` ;
  `exports/global.rs:116` ; `csv_tables.rs:640`.

## Questions ouvertes

- *Q-38-2-a **close** par l'arbitrage O1 (2026-11-15) : contrainte remplacée par `chk_bank_accounts_primary_has_iban`
  dans la migration breaking de cette story ; le fichier d'epic (E11, inventaire « IBAN facultatif ») reste à corriger
  par l'orchestrateur.*
- *Q-38-2-b **close** par l'arbitrage O8 : délier n'est refusé que si le compte porte des transactions (AC-4).*
- *Q-38-2-c **close** par l'arbitrage O8 : un compte du plan en devise n'est lié qu'à un seul compte prestataire
  (`PROVIDER_ACCOUNT_LEDGER_SHARED`).*

## Change Log

- 2026-11-15 — rédaction (Opus). Écart à l'epic relevé au code : `chk_bank_accounts_bank_has_iban` interdit un compte
  `bank` sans IBAN (Q-38-2-a, bloquante).
- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 2 LOW ; remédiés dans la fiche sauf 38-2-P1-3 et
  38-2-P1-6 (inter-fiches). Trend : P1 = 6. HIGH : la migration est **breaking** (O1) — la fiche la disait non
  breaking (NFR-38-2-3, AC-1, Q-38-2-a) : relèvement de version, `match_type`, P2-bis, exemption P7, retour par la
  sauvegarde, recette de montée. MEDIUM : arbitrage O8 sur le délier (le refus ne vaut qu'avec des transactions) ;
  sites de dénombrement oubliés (`data_count`, `EMPTY_BEFORE_RESET`, squash `:176/:1167/:1180`) ; import de fichier
  sur compte prestataire refusé par O9. LOW : T0 ; `exemptions` décompte. Lignes lues :
  `20261110000001_…sql:39-55`, `post_restore.rs:639-653, 1252`, `20260513000001_reconciliation_rules.sql:40, 69`,
  `0001_schema_squash.sql:176, 1167, 1180`, `payment_batches.rs:158-185`, `invoice_pdf_service.rs:332-353`.
- 2026-11-15 — alignement inter-fiches P1 (Opus) : AC-1 complété du texte de 38-4 P1 — `MODIFY COLUMN match_type`
  exact (colonne d'origine sans `DEFAULT` ni `COMMENT`), `CHECK` `^[a-z][a-z_]{1,39}$` nommé, index reconstruit à
  l'`ALTER` sans `DROP INDEX`, test d'unicité après l'`ALTER`, P6 relevé, tri des `0.17.6` (`grep`, assertions de la
  version courante contre jalons historiques), CHANGELOG et sous-section du manuel par 38-9a ; 38-2-P1-3 close (décodage
  tolérant en 38-3a pour la source, en 38-4 pour les critères) ; AC-5 : `PROVIDER_ACCOUNT_IN_USE` liste `balance_seen`
  (instantanés, posée par 38-3a) et `FEE_ACCOUNT_INVALID` la raison `missing` (posée par 38-6a) ; renvois à 38-3 répartis
  entre 38-3a et 38-3b.
- 2026-11-15 — validation P2 (Opus) : 0 CRITICAL, 1 HIGH, 5 MEDIUM, 1 LOW ; remédiés dans la fiche sauf les reports
  inter-fiches de 38-2-P2-1 et 38-2-P2-3 (C-38-3, O8, 38-9a). Trend : P1 = 6 (1 HIGH, 3 MEDIUM) → P2 = 7 (1 HIGH,
  5 MEDIUM) — défauts **distincts** de ceux de la P1 et non nés de sa remédiation, hormis P2-5 (phrase de
  l'alignement inter-fiches) : pas de recyclage, signal de découpage non levé (règle de splitting, amendement D5).
  HIGH : `attach` d'un compte qui porte des transactions de fichier → doublons muets au premier passage (aucun
  `provider_tx_id`) et transactions CHF sous un solde en devise → refusé (`PROVIDER_ACCOUNT_HAS_TRANSACTIONS`, AC-3,
  course (e) à l'AC-6). MEDIUM : `PATCH`/`PUT` du lien n'appliquaient qu'une partie de la garde de l'AC-3 (un passif
  passait, `admits_ledger_type` l'admet pour `Bank`) → garde entière, une fonction ; `LEDGER_SHARED` limité à la
  devise alors qu'en CHF rien ne ferme le partage, ni dans un sens ni dans l'autre → toute devise, sens inverse gardé
  sur les comptes ordinaires (AC-3, AC-4, FR-38-2-6) ; `no_iban` rendait les réglages d'un compte créé (`create`)
  irretirables à jamais (devise prise, connexion insupprimable), contre la Note sur l'archivage → exception du compte
  archivé, chemin écrit (AC-5) ; « 38-3a y ajoute les soldes rapportés » au délier : contraire à O8 et absent de 38-3a
  → retiré (AC-4) ; refus pain.001 lu « sous le verrou déjà pris » à `payment_batches.rs:165`, qui est une lecture
  non verrouillante, et commentaire `:157-162` dit « reste vrai » alors qu'un `PUT` peut annuler l'IBAN → décision
  après `recheck_open_unarchived_locked_in_tx`, génération gardée aussi (AC-7, AC-1). LOW : `onboarding_audit_e2e.rs:2046` ;
  archivage gardé par `archive_once` (`routes/bank_accounts.rs:1645`), non par `:806` ; course (c) changée par 38-3a.
  Lignes lues : `20261110000001_…sql:36-56` ; `20260513000001_…sql:36-70` ; `0001_schema_squash.sql:176, 1167, 1180` ;
  `entities/bank_account.rs:47-69` ; `entities/reconciliation_rule.rs:20-110` ; `routes/bank_accounts.rs:338, 496-572,
  883, 964-969, 1383-1405, 1600-1660, 2032` ; `repositories/bank_accounts.rs:544-555, 570, 655-665, 730, 800-870,
  1544, 1650-1690, 1725, 1834` ; `payment_batches.rs:150-190, 232-250, 870-890` ; `post_restore.rs:415-445, 1245-1262` ;
  `errors.rs:820-832, 2655-2670` ; `retry.rs:40` ; `CHANGELOG.md:1-20` ; P6 recompté (`grep -rc`, mêmes nombres).
- 2026-11-15 — validation P3 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche (aucun finding
  inter-fiches). Périmètre : correctifs P2 (`attach` refusé avec transactions, garde entière du compte lié,
  `LEDGER_SHARED` toute devise, `no_iban` et archivé, pain.001 sous verrou). LOW 38-2-P3-1 : `reconciliation.rs:1097-1100`
  sans chemin (le fichier est `routes/reconciliation.rs`) → précisé. Lignes relues et justes :
  `payment_batches.rs:150-166, 232-250` (lecture de `:165` non verrouillante, revérification `LOCK IN SHARE MODE` après
  l'`INSERT`), `repositories/bank_accounts.rs:570, 730, 806, 1544, 1670`, `routes/bank_accounts.rs:1656` (`archive_once`),
  `routes/reconciliation.rs:1096-1104`. D-P2-5 sans objet ici (propositions : 38-6a, 38-6b, 38-9b).
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : CHANGELOG remplacé par une section *Note de
  version* ; compteurs « recomptés après chaque reprise de `main` ». Aucun PDF commité dans cette fiche.
