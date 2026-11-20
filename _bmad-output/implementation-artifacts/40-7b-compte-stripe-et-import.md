# Story 40-7b : réglages du compte Stripe et import du rapport, ventilé par devise (migration de l'epic)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, numéros de ligne relevés sur `origin/main` au commit `fd04099f`.
Epic 40, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 40-7a (lecteur) ; la **phase 1 de l'Epic 38
fusionnée** — 38-11b (colonnes `bank_accounts.provider`, `provider_account`, `fee_account_id`,
`bank_transactions.provider_type`, `counterparty_email`, index de doublon, levée de C-37-9 pour un compte prestataire,
route d'import PayPal) ; l'Epic 37 publié en 0.19.0. **Suivie par** : 40-7c (rapprochement), 40-7d (écrans).*

## Story

En tant qu'**Administrateur**, je veux faire d'un compte bancaire Kesh le solde, dans une devise, de mon compte Stripe —
sans IBAN, lié au compte de transit Stripe de cette devise, avec son compte de frais et son **compte de versement** —,
puis, en tant que **Comptable**, importer d'un geste le rapport « Itemized payout reconciliation » d'une période, ventilé
par devise, sans doublon, afin que chaque mouvement du solde Stripe attende son rapprochement (40-7c).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-7b-1` | Un compte bancaire porte, facultativement, des réglages Stripe : le **compte Stripe** (libellé), le **compte de frais** (celui de 38-11b) et le **compte bancaire de versement** (Wise EUR/USD, banque CHF…), de **même devise**. | AC-1, AC-2 |
| `FR-40-7b-2` | Un compte Stripe a au plus un compte bancaire par devise ; IBAN facultatif et compte du plan en devise admis comme pour PayPal (même code, aucune seconde règle). | AC-2, AC-3 |
| `FR-40-7b-3` | Import **à la demande** (route et bouton) du rapport d'un compte Stripe : chaque devise va au compte bancaire de cette devise ; une devise sans compte refuse tout l'import en la nommant. | AC-4, AC-5 |
| `FR-40-7b-4` | Une ligne donne une transaction pour le brut et une pour les frais ; chaque versement donne une transaction (moins la somme de ses nets) ; doublons (même identifiant sur le même compte bancaire) ignorés et comptés. | AC-5, AC-6 |
| `FR-40-7b-5` | Le chemin d'import d'un prestataire (ventilation par devise, `bank_imports`, verrous, doublons, compte rendu) est **un seul code** pour PayPal et Stripe. | AC-7 |
| `NFR-40-7b-1` | **Une** migration, non breaking, DDL seul. | AC-1 |
| `NFR-40-7b-2` | Sauvegarde, réimport d'une sauvegarde de la version précédente, export CSV et remise à zéro restent justes. | AC-8 |

## Critères d'acceptation

1. **AC-1 — Migration `20261120000003_stripe_import.sql`** (après `20261120000001_paypal_import` de 38-11b ; `…0002`
   laissé à l'Epic 39 s'il en a besoin ; si une migration de `main` ou d'un autre epic la dépasse à la reprise de `main`,
   la renuméroter **avant toute application hors base jetable**, P8). **Non breaking** au sens du ⚠️ de P1 :
   `kesh_version_min_required` inchangé, aucune version Cargo relevée ; en tête, le même argument que
   `20261118000001_grouped_settlements.sql` (un binaire antérieur refuse de démarrer sur la base migrée ; aucun chemin ne
   lui fait lire la colonne neuve). Instructions :
   - `ALTER TABLE bank_accounts ADD COLUMN payout_bank_account_id BIGINT NULL` + FK
     `fk_bank_accounts_payout_bank_account` → `bank_accounts(id)` `ON DELETE RESTRICT` (clé étrangère sur la même table :
     précédent `accounts.parent_id`, tolérée par la restauration, `crates/kesh-db/src/backup.rs:815-822`, vidage sous
     `FOREIGN_KEY_CHECKS = 0`, `:851`) ;
   - **si** la contrainte `chk_bank_accounts_provider` posée par 38-11b n'admet que `'paypal'` (à relire au code fusionné,
     T0) : `DROP CONSTRAINT` puis `ADD CONSTRAINT` **sous le même nom**, identique à l'octet près hors la liste des valeurs
     (`BINARY provider IN ('paypal', 'stripe')`, et `'wise'` si l'Epic 39 l'a ajoutée) ; ajout, dans la même contrainte,
     de `(payout_bank_account_id IS NULL OR BINARY provider = 'stripe')`. Si 38-11b admet déjà `'stripe'` (recommandation
     faite à l'Epic 38, § *Contrats* du fichier d'epic, C-40-K1), seule cette dernière condition s'ajoute — par une
     contrainte neuve `chk_bank_accounts_payout_stripe`, sans toucher l'autre.
   Aucun `UPDATE` (P7 sans objet ; `every_data_backfill_migration_is_triaged` vert). **P5** : ligne à
   `docs/migrations-idempotence-audit.md` (`tracked-by-sqlx`), en-tête et ligne `Total` recomptés
   (`ls crates/kesh-db/migrations/*.sql | wc -l` — 95 au relevé sur `main`, 96 après 38-11b, 97 ici si 39 n'en ajoute
   pas ; `grep -c '^| \`20' docs/migrations-idempotence-audit.md`), partitions recomptées depuis le tableau. **P6** :
   `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` exécuté, chaque site inspecté, au Change Log. **P8** :
   aucune migration appliquée touchée. Squash de test régénéré (`scripts/regen-test-schema.sh`, `test_schema_guard.rs`
   vert). Test de migration : `payout_bank_account_id` refusé sur un compte sans `provider = 'stripe'` ; `provider =
   'Stripe'` refusé (octet) ; suppression d'un compte de versement référencé refusée (FK). **Gate complet** même en cours
   de boucle (exception `kesh-db`).
2. **AC-2 — Réglages Stripe d'un compte bancaire** (routes existantes `POST /api/v1/bank-accounts` et `PUT
   /api/v1/bank-accounts/{id}`, `crates/kesh-api/src/lib.rs:1053`). Le corps gagne `stripe: { account: string,
   feeAccountId: number | null, payoutBankAccountId: number | null }` ; absent au `PUT` = inchangé, `null` = retiré ;
   `paypal` et `stripe` ensemble → 400 `VALIDATION_ERROR`. Les gardes de 38-11b AC-2 (libellé, compte lié obligatoire,
   un compte bancaire par devise et par compte prestataire, compte de frais actif, imputable, de charge, CHF ou de la
   devise du compte lié, compte « en usage » non modifiable) **sont les mêmes fonctions**, paramétrées par le
   prestataire (`provider` = `stripe`), avec les codes généralisés si 38-11b les a nommés `PAYPAL_*` : renommage en
   `PROVIDER_*` dans le même commit, ou codes `STRIPE_*` jumeaux — à trancher en T0 sur le code fusionné, sans dupliquer
   la logique (décision au Change Log). Garde propre à Stripe : `payoutBankAccountId` désigne un compte bancaire de la
   société, non archivé, **qui n'est pas un compte Stripe**, dont le compte du plan est **dans la même devise** que le
   compte lié (sinon 400 `STRIPE_PAYOUT_ACCOUNT_INVALID`, `details: { bankAccountId, reason }`, `reason` ∈ `not_found`,
   `archived`, `stripe`, `currency`) ; absent admis (les versements vont alors « à examiner », 40-7c). La réponse gagne
   `stripe: { account, feeAccountId, payoutBankAccountId } | null` ; entité `BankAccount`
   (`crates/kesh-db/src/entities/bank_account.rs`) et lectures du dépôt (`repositories/bank_accounts.rs`) portent la
   colonne ; la trace d'audit des créations et modifications l'inclut.
3. **AC-3 — IBAN et devise : aucune seconde règle.** `validate_bank_input`
   (`crates/kesh-api/src/routes/bank_accounts.rs:548`) et la levée de C-37-9 (`locked_ledger_type`, `:435`, et le
   pré-vol des refus `ACCOUNT_IN_FOREIGN_CURRENCY`) appliquent à `stripe` exactement ce que 38-11b AC-3 applique à
   `paypal` : la condition « corps porte un prestataire » remplace « corps porte `paypal` » (une seule expression,
   testée pour les deux). Un compte Stripe n'est jamais principal, n'entre ni aux lots pain.001 ni à la QR-facture (test
   existant de 38-11b étendu à `stripe`).
4. **AC-4 — Route d'import** `POST /api/v1/stripe-imports` (multipart : `file`, `account`), Comptable et plus, clés
   d'API `read-write` admises (comme `POST /api/v1/bank-imports`, `lib.rs:946`), inscrite au registre des routes
   auditées (`crates/kesh-api/tests/audit_route_registry.rs`). Taille maximale : celle de l'import bancaire. Ordre des
   refus, rien d'écrit à aucun : 400 `VALIDATION_ERROR` → 404 `STRIPE_ACCOUNT_NOT_FOUND` → 422 `STRIPE_CSV_INVALID`
   (`details` : colonnes manquantes, avec la marche à suivre pour `source_id|charge_id`, ou erreurs de ligne, forme de
   `CsvLineErrorPayload`, `crates/kesh-api/src/routes/bank_imports.rs:702`) → 422 `STRIPE_CURRENCY_ACCOUNT_MISSING`
   (`details: { currencies }`) → 409 `BANK_ACCOUNT_CLOSED` → 422 `BANK_IMPORT_DUPLICATE_FILE` (empreinte de l'AC-6).
5. **AC-5 — Ventilation.** Par devise, **une** entrée `bank_imports` (`create_with_transactions`,
   `crates/kesh-db/src/repositories/bank_imports.rs:72`) sur le compte bancaire de la devise, `source_format = 'CSV'`
   (valeur existante, comme 38-11b), `filename`, période = première et dernière date de la devise, soldes `NULL`. Chaque
   ligne donne (`NewBankTransaction`, `crates/kesh-db/src/entities/bank_transaction.rs:113`, champs de 38-11b) :
   - **brut** (si `gross ≠ 0`) : `amount = gross`, `currency`, `booking_date = date`, `value_date = available_on`,
     `transaction_id = balance_transaction_id`, `reference` = `source_id` à défaut `charge_id` (l'identifiant que la
     boutique écrit dans le libellé de la vente, 40-7c), `end_to_end_id = payout_id`, `provider_type = category`,
     `details` = `description` (tronqué comme les autres imports) ;
   - **frais** (si `fee ≠ 0`) : mêmes champs, `amount = −fee`, `transaction_id = "<id>:FEE"` (la constante de suffixe de
     38-11b, partagée), `details` = « Frais Stripe · » suivi de la catégorie ;
   - **versement** : une transaction par `payout_id` de la devise, `amount = −Σ net` de ses lignes **dans le fichier**,
     `booking_date = payout_date`, `transaction_id = payout_id`, `end_to_end_id = payout_id`, `provider_type = 'payout'`,
     `details` = « Versement Stripe ». (Le rapport est découpé par versement : toutes les lignes d'un versement y sont.
     Un fichier filtré par catégorie dans Stripe donnerait un total faux : 40-7c le contrôle et le signale.)
   La somme des transactions insérées d'une devise vaut zéro quand le fichier ne contient que des versements complets
   (test). Import sous le verrou de **chaque** compte visé, par identifiant croissant (`with_account_lock`,
   `crates/kesh-reconciliation/src/mutex.rs:75`, comme `create_under_account_lock`,
   `crates/kesh-api/src/routes/bank_imports.rs:1532`) ; une seule transaction de base : tout ou rien. Trace d'audit :
   celle de l'import de 38-11b, avec `stripe_import.created` (`details` : compte Stripe, fichier, compte rendu).
6. **AC-6 — Doublons et compte rendu.** Empreinte : SHA-256 de `fichier ‖ ":" ‖ devise` par `bank_imports`, refus de
   l'AC-4 si `find_by_company_and_hash` (`repositories/bank_imports.rs:308`) la trouve (même règle que 38-11b).
   **Doublon de transaction** : une transaction du même compte bancaire porte déjà ce `transaction_id` (brut, `:FEE` ou
   versement) → ignorée ; la fenêtre date-montant-référence des imports bancaires (`find_in_dedup_window`,
   `repositories/bank_transactions.rs:83`) ne s'applique pas. Un rapport par devise puis le rapport toutes devises de la
   même période s'importent sans rien doubler (test). Réponse `201` : `{ account, currencies: [{ currency,
   bankAccountId, importId | null, inserted, duplicates }], payouts: [{ payoutId, currency, lines, net }] }`.
7. **AC-7 — Un seul chemin d'import de prestataire (DRY).** La ventilation par devise, la recherche des comptes, les
   refus de l'AC-4 qui ne dépendent pas du format, les verrous, l'empreinte, les doublons et le compte rendu sont **une**
   fonction commune (module `crates/kesh-api/src/routes/provider_imports.rs` ou dans `kesh-db`, selon ce que 38-11b a
   laissé), qui reçoit des lignes neutres (`ProviderLine { currency, booking_date, value_date, amount, transaction_id,
   reference, end_to_end_id, counterparty_name, counterparty_email, provider_type, details }`). Si 38-11b l'a écrite
   pour PayPal seul, 40-7b la **déplace** et la route PayPal l'appelle ; tests de 38-11b inchangés en assertions. Seuls
   restent propres à Stripe : le lecteur (40-7a), la traduction ligne → `ProviderLine` (fonction pure, testée sans base)
   et les transactions de versement.
8. **AC-8 — Sauvegarde, export, remise à zéro, i18n.** `.keshbackup` : la colonne passe seule ; aller-retour d'un compte
   Stripe avec compte de versement testé (`admin_full_import_e2e.rs`). Liste des absences admises
   (`ABSENCES_ADMISES_EPIC_37`, `crates/kesh-db/src/backup.rs:170`, migration ≥ `PREMIERE_MIGRATION_EPIC_37`, `:239`) :
   `bank_accounts.payout_bank_account_id` ajoutée — une sauvegarde de la version précédente s'importe. Export CSV :
   colonne ajoutée à `serialize_bank_accounts_csv` (`crates/kesh-api/src/exports/csv_tables.rs:680`) ;
   `chaque_colonne_du_schema_est_exportee_ou_ecartee` (`:2658`) vert. `TABLES_TO_TRUNCATE` (`backup.rs:46`) inchangé
   (vidage sous `FOREIGN_KEY_CHECKS = 0`). Codes neufs (`STRIPE_PAYOUT_ACCOUNT_INVALID`, `STRIPE_ACCOUNT_NOT_FOUND`,
   `STRIPE_CSV_INVALID`, `STRIPE_CURRENCY_ACCOUNT_MISSING`, et les codes jumeaux de l'AC-2 si c'est le choix) : `errors.rs`
   et messages aux quatre locales (`crates/kesh-i18n/locales/*`).

## Tâches

- [ ] T0 — Reprendre `main` et la branche de l'Epic 38 fusionnée ; relire 38-11b au code (contrainte, codes, route) ;
  date de la migration ; décision sur les codes de l'AC-2.
- [ ] T1 — Migration, squash, audit, test de migration (AC-1).
- [ ] T2 — Entité, dépôt, réglages Stripe aux routes des comptes bancaires (AC-2, AC-3).
- [ ] T3 — Chemin d'import commun, déplacé de PayPal s'il le faut (AC-7).
- [ ] T4 — Route, ventilation, versements, doublons, compte rendu (AC-4 à AC-6).
- [ ] T5 — Sauvegarde, export, i18n (AC-8).

## Stratégie de tests

- **Unitaires** (sans base) : traduction `StripePayoutRow` → `ProviderLine` (brut, frais, signe des frais rendus d'un
  remboursement, versement = − Σ net, référence `source_id` puis `charge_id`).
- **HTTP** (`crates/kesh-api/tests/stripe_import_e2e.rs`, `#[sqlx::test]` sur le squash), fixture de 40-7a : société
  multi-devises, comptes CHF, EUR, USD d'un compte Stripe, comptes de versement ; import → trois entrées, nombres
  attendus ; réimport → 422 ; rapport par devise puis toutes devises → doublons ; fichier sans `source_id` → 422 avec la
  marche à suivre ; devise sans compte → 422, rien d'écrit ; compte clos → 409 ; clé d'API `read` → 403. Réglages :
  chaque refus de l'AC-2 ; `paypal` et `stripe` ensemble → 400. Tests de l'import PayPal relancés (AC-7).
- **Migration** : test de l'AC-1 ; `test_schema_guard.rs`. Gate **complet** (exception `kesh-db`).

## Modules touchés (4)

`kesh-db` (migration, entité, dépôt, sauvegarde), `kesh-api` (routes `bank_accounts`, `stripe_imports` neuve, chemin
commun, erreurs, export), `kesh-import` (lecture seule de 40-7a), `kesh-i18n`.

## Note de version

*Ajouté* — Un compte bancaire peut être le solde, dans une devise, d'un compte Stripe : sans IBAN, lié au compte de
transit de la devise, avec son compte de frais et son compte de versement. Le rapport « Itemized payout reconciliation »
s'importe d'un geste, ventilé par devise ; les doublons sont ignorés.
*Migration* — `20261120000003_stripe_import`, non breaking : la sauvegarde d'avant la mise à jour reste exigée (base,
`.keshbackup`, dossier des documents) ; le retour à la version précédente passe par elle.

## Notes de développement

- Remplace l'ancienne 40-1 pour les réglages (`superseded` : coffre et connexion de 38-1/38-2, tous deux `deferred`).
- ⚠️ Risque d'horodatage : 38-11b prend `20261120000001` ; si l'Epic 38 renumérote, suivre. La base de Beat ne reçoit la
  migration qu'à une version publiée, ou en connaissance de cause (P8).
- Le compte de versement est un **compte bancaire** (et non un compte du plan) pour que 40-7c trouve la réception du
  versement (« Argent reçu de Stripe » côté Wise, Epic 39 ; ligne du relevé CAMT côté banque) sur le bon compte.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
