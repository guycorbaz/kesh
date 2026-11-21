# Story 40-7b : réglages du compte Stripe et import du rapport, ventilé par devise (migration de l'epic)

Status: done

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, numéros de ligne relevés sur `origin/main` au commit `fd04099f`.
Epic 40, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 40-7a (lecteur) ; la **phase 1 de l'Epic 38
fusionnée** — 38-11b (colonnes `bank_accounts.provider`, `provider_account`, `fee_account_id`, contrainte
`chk_bank_accounts_provider` qui admet **déjà** `'paypal'`, `'wise'` et `'stripe'`, `bank_transactions.provider_type`,
`counterparty_email`, index de doublon, levée de C-37-9 pour un compte prestataire, `with_account_locks`, refus
`BANK_ACCOUNT_IS_PROVIDER`, route d'import PayPal) et la phase 1 de l'Epic 39 (39-11b : cœur d'import commun, objet
générique des réglages) ; l'Epic 37 publié en 0.19.0. **Suivie par** : 40-7c (rapprochement), 40-7d (écrans).*

## Story

En tant qu'**Administrateur**, je veux faire d'un compte bancaire Kesh le solde, dans une devise, de mon compte Stripe —
sans IBAN, lié au compte de transit Stripe de cette devise, avec son compte de frais et son **compte de versement** —,
puis, en tant que **Comptable**, importer d'un geste le rapport « Itemized payout reconciliation » d'une période, ventilé
par devise, sans doublon, afin que chaque mouvement du solde Stripe attende son rapprochement (40-7c).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-7b-1` | Un compte bancaire porte, facultativement, des réglages Stripe : le **compte Stripe** (libellé) et le **compte de frais** (ceux de 38-11b) et, seul ajout de Stripe, le **compte bancaire de versement** (Wise EUR/USD, banque CHF…), de **même devise**. | AC-1, AC-2 |
| `FR-40-7b-2` | Un compte Stripe a au plus un compte bancaire par devise ; IBAN facultatif et compte du plan en devise admis comme pour PayPal (même code, aucune seconde règle). | AC-2, AC-3 |
| `FR-40-7b-3` | Import **à la demande** (route et bouton) du rapport d'un compte Stripe : chaque devise va au compte bancaire de cette devise ; une devise sans compte refuse tout l'import en la nommant. | AC-4, AC-5 |
| `FR-40-7b-4` | Une ligne donne une transaction pour le brut et une pour les frais ; chaque versement donne une transaction (moins la somme de ses nets) ; doublons (même identifiant sur le même compte bancaire) ignorés et comptés. | AC-5, AC-6 |
| `FR-40-7b-5` | Le chemin d'import d'un prestataire (ventilation par devise, `bank_imports`, verrous, doublons, compte rendu) est **un seul code** pour PayPal et Stripe. | AC-7 |
| `NFR-40-7b-1` | **Une** migration, non breaking, DDL seul : une colonne, sa clé étrangère, une contrainte. | AC-1 |
| `NFR-40-7b-2` | Sauvegarde, réimport d'une sauvegarde de la version précédente, export CSV et remise à zéro restent justes. | AC-8 |

## Critères d'acceptation

1. **AC-1 — Migration `20261120000003_stripe_import.sql`** (après `20261120000001_paypal_import` de 38-11b ; `…0002`
   laissé à l'Epic 39, dont le repli éventuel est une migration ; si une migration de `main` ou d'un autre epic la
   dépasse à la reprise de `main`, la renuméroter **avant toute application hors base jetable**, P8). **Non breaking**
   au sens du ⚠️ de P1 : `kesh_version_min_required` inchangé, aucune version Cargo relevée ; en tête, le même argument
   que `20261118000001_grouped_settlements.sql` (un binaire antérieur refuse de démarrer sur la base migrée ; aucun
   chemin ne lui fait lire la colonne neuve). **Une seule instruction** (`ALTER TABLE bank_accounts`, formes
   `IF NOT EXISTS` là où MariaDB les admet, précédent `20261118000002`) : `ADD COLUMN payout_bank_account_id BIGINT
   NULL`, FK `fk_bank_accounts_payout_bank_account` → `bank_accounts(id)` `ON DELETE RESTRICT` (clé étrangère sur la
   même table : précédent `accounts.parent_id`, tolérée par la restauration, `crates/kesh-db/src/backup.rs:815-822`,
   vidage sous `FOREIGN_KEY_CHECKS = 0`, `:851`), et CHECK `chk_bank_accounts_payout_stripe`
   `(payout_bank_account_id IS NULL OR BINARY provider = 'stripe')` (même forme que la condition de
   `fee_account_id` dans `chk_bank_accounts_provider` : la co-existence d'une FK `RESTRICT` et d'un CHECK sur la
   colonne est prouvée sur MariaDB 10.11 à T0, au Change Log). **La contrainte `chk_bank_accounts_provider` n'est pas
   touchée** : 38-11b admet `'stripe'` d'emblée (C-40-K1 tenu ; décision de l'orchestrateur du 2026-11-20 : la
   migration de Stripe ne garde que cette colonne). **T0** : si la contrainte fusionnée n'admet pas `'stripe'`, la
   fiche **s'arrête** et le dit à l'Epic 38, elle ne repose pas la contrainte ici. Aucun `MODIFY COLUMN`, `DROP`,
   `RENAME` (P3 sans objet) ; aucun `UPDATE` (P7 sans objet ; `every_data_backfill_migration_is_triaged` vert).
   **P5** : ligne à `docs/migrations-idempotence-audit.md` (`tracked-by-sqlx`), en-tête et ligne `Total` recomptés
   (`ls crates/kesh-db/migrations/*.sql | wc -l` — 95 au relevé sur `main`, 96 après 38-11b, 97 ici ; `grep -c '^| \`20'
   docs/migrations-idempotence-audit.md`), partitions recomptées depuis le tableau. **P6** : `grep -rn
   "migrations.len()\|apply_migrations_up_to" crates/` exécuté, chaque site inspecté, au Change Log. **P8** : aucune
   migration appliquée touchée. Squash de test régénéré (`scripts/regen-test-schema.sh`, `test_schema_guard.rs` vert).
   Test de migration : `payout_bank_account_id` refusé sur un compte sans `provider = 'stripe'` ; `provider = 'Stripe'`
   refusé (octet) ; suppression d'un compte de versement référencé refusée (FK). **Gate complet** même en cours de
   boucle (exception `kesh-db`).
2. **AC-2 — Réglages Stripe d'un compte bancaire** (routes existantes `POST /api/v1/bank-accounts` et `PUT
   /api/v1/bank-accounts/{id}`, `crates/kesh-api/src/lib.rs:1053`, Comptable et plus). Le corps suit **la forme
   qu'a retenue 39-11b** (G10) : l'objet générique `provider: { kind: "paypal" | "wise" | "stripe", account,
   feeAccountId }` (ou, si 39-11b n'a pas renommé l'objet `paypal` de 38-11b, un objet `stripe` de même forme servi par
   la **même** fonction : décision au Change Log) ; pour `stripe` s'y ajoute `payoutBankAccountId: number | null`,
   refusé (400 `VALIDATION_ERROR`) pour un autre `kind`. Absent au `PUT` = inchangé, `null` = retiré. Les gardes de
   38-11b AC-2 (libellé, compte lié obligatoire, un compte bancaire par devise et par compte prestataire, compte de
   frais actif, imputable, de charge, CHF ou de la devise du compte lié, compte « en usage » non modifiable) **sont les
   mêmes fonctions**, paramétrées par le prestataire, avec les codes de 38-11b ou leurs noms généralisés
   (`PROVIDER_*`) par 39-11b : **aucun code `STRIPE_*` jumeau** (pas de seconde logique),
   les codes neufs de cette fiche étant ceux qui ne valent que pour Stripe. Garde propre à Stripe :
   `payoutBankAccountId` désigne un compte bancaire de la société, non archivé, **sans prestataire ou compte Wise** (un
   compte `stripe` ou `paypal` n'est jamais une destination de versement), dont le compte du plan est **dans la même
   devise** que le compte lié du compte Stripe ; sinon 400 `STRIPE_PAYOUT_ACCOUNT_INVALID`, `details: { bankAccountId,
   reason }`, `reason` ∈ `not_found`, `archived`, `closed`, `stripe`, `paypal`, `currency`. Absent admis (les versements vont
   alors « à examiner », 40-7c). Le compte de versement se change **même quand le compte Stripe porte des transactions** (il n'agit que sur
   les écritures à venir, comme le compte de frais). La réponse gagne `payoutBankAccountId` dans l'objet du
   prestataire ; entité `BankAccount` (`crates/kesh-db/src/entities/bank_account.rs`) et lectures du dépôt
   (`repositories/bank_accounts.rs`) portent la colonne ; la trace d'audit des créations et modifications l'inclut.
3. **AC-3 — IBAN, devise, refus aux autres chemins : aucune seconde règle.** `validate_bank_input`
   (`crates/kesh-api/src/routes/bank_accounts.rs:548`), la levée de C-37-9 (`locked_ledger_type`, `:435`, et le
   pré-vol des refus `ACCOUNT_IN_FOREIGN_CURRENCY`), le refus du `PATCH` du lien du compte du plan, et les refus
   **`BANK_ACCOUNT_IS_PROVIDER`** de 38-11b AC-3 (six sites : aperçu et création d'un import CAMT/CSV, aperçu et création
   d'un relevé saisi, règlement d'une facture client, paiement d'une facture fournisseur) jugent « compte prestataire » par **`provider IS NOT NULL`** (état résultant pour les
   réglages) : si 38-11b ou 39-11b les ont écrits `provider = 'paypal'` ou `'wise'`, 40-7b en fait **une** expression
   (testée pour les trois prestataires, T0). Un compte Stripe n'est jamais principal (CHECK de 38-11b), n'entre ni aux
   lots pain.001 ni à la QR-facture (test de 38-11b étendu à `stripe`), et son solde d'ouverture en devise est refusé
   comme celui d'un compte PayPal.
4. **AC-4 — Route d'import** `POST /api/v1/stripe-imports` (multipart : `file`, `account`), Comptable et plus, clés
   d'API `read-write` admises (comme `POST /api/v1/bank-imports`, `lib.rs:946`), inscrite à `LIB_ROUTES`
   (`crates/kesh-api/tests/audit_route_registry.rs:440`) en `Traced, SansEcritureAuJournal` (elle n'écrit **aucune
   écriture** : la revue et l'acceptation sont d'autres routes, 40-7c) ; le compte de `the_registry_partition_is_what_the_story_declares`
   (`:1082`) relevé de un avec sa raison, la phrase « N des M routes à verbe mutant » et la somme du manuel
   d'administration (`docs/manual/fr/admin-manual.tex`, lues par `le_manuel_dit_le_compte_du_registre`) mises à jour —
   **`.tex` seul**, PDF à la release ; les nombres se **recomptent** depuis la source après la fusion des Epics 38 et 39.
   Si 38-11b ou 39-11b exposent une route générique d'import prestataire, Stripe s'y branche (choix au Change Log). Taille maximale : celle de l'import bancaire. Ordre des
   refus, rien d'écrit à aucun : 400 `VALIDATION_ERROR` → 404 `PROVIDER_ACCOUNT_NOT_FOUND` (aucun compte bancaire non
   archivé `provider = 'stripe'` de ce libellé ; le code générique de 39-11b, ou celui de 38-11b généralisé) → 422
   `STRIPE_CSV_INVALID` (erreur de 40-7a ; `details` : le code de l'erreur, les colonnes manquantes — `source_id|charge_id`
   s'y nomme avec la marche à suivre, « dans Stripe, Rapports → Rapprochement des versements → Détaillé, ajoutez la
   colonne *source_id* » — ou les erreurs de ligne, forme `CsvLineErrorPayload`, `crates/kesh-api/src/errors.rs:1541`,
   par `csv_line_error_to_payload`, `routes/bank_imports.rs:702`) → 422 `PROVIDER_CURRENCY_ACCOUNT_MISSING` (`details:
   { currencies }`, jamais de solde créé d'office) → 409 `BANK_ACCOUNT_CLOSED` → 422 `BANK_IMPORT_DUPLICATE_FILE`
   (empreinte de l'AC-6). Aucun paramètre `match` : l'écran appelle l'import, puis recharge la revue (`GET /api/v1/stripe/review`, 40-7c).
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
   - **versement** : une transaction par `payout_id` de la devise **dont Σ net ≠ 0**, `amount = −Σ net` de ses lignes
     **dans le fichier** (négatif pour un versement ordinaire : l'argent quitte le solde), `booking_date =
     payout_date`, `transaction_id = payout_id`, `end_to_end_id = payout_id`, `provider_type = 'payout'`, `details` =
     « Versement Stripe ». (Le rapport est découpé par versement : toutes les lignes d'un versement y sont. ⚠️ Un
     fichier filtré par catégorie dans Stripe donne un total faux **que rien ne contrôle ici** — le montant du versement
     est calculé sur les mêmes lignes — : le seul garde-fou est que la réception du versement sur Wise ou à la banque
     ne trouve pas son montant, écart `payout_without_receipt` de 40-7c ; le manuel dit de ne pas filtrer le rapport.)
   La somme des transactions insérées d'une devise vaut zéro quand le fichier ne contient que des versements à Σ net ≠ 0
   (test). Import sous le verrou de **chaque** compte visé, par identifiant croissant : **`with_account_locks`**, le
   helper de 38-11b (`crates/kesh-reconciliation/src/mutex.rs`, à côté de `with_account_lock`, `:75`), appelé par le
   cœur commun de l'AC-7 ; une seule transaction de base : tout ou rien. Trace d'audit : celle de l'import de 38-11b,
   avec `stripe_import.created` (`details` : compte Stripe, fichier, compte rendu).
6. **AC-6 — Doublons et compte rendu.** Empreinte : SHA-256 de `fichier ‖ ":" ‖ devise` par `bank_imports`, refus de
   l'AC-4 si `find_by_company_and_hash` (`repositories/bank_imports.rs:308`) la trouve (même règle que 38-11b).
   **Continuité : aucune règle pour Stripe.** Le rapport est découpé par versement et ne porte **aucun solde** (les
   entrées `bank_imports` ont des soldes `NULL`, AC-5) : aucun avertissement de trou n'est calculé, et
   `previous_statement_closing` (`bank_imports.rs:358`) ignore de toute façon un relevé sans solde. Si une phase
   ultérieure en ajoutait une, elle lirait le chevauchement **comme la 39-11b AC-5** — `overlapping_statements` d'abord
   (chevauchement : pas d'avertissement), puis `previous_statement_closing` — une seule définition.
   **Doublon de transaction** : une transaction du même compte bancaire porte déjà ce `transaction_id` (brut, `:FEE` ou
   versement) → ignorée ; la fenêtre date-montant-référence des imports bancaires (`find_in_dedup_window`,
   `repositories/bank_transactions.rs:83`) ne s'applique pas. Un rapport par devise puis le rapport toutes devises de la
   même période s'importent sans rien doubler (test). Réponse `201` : `{ account, currencies: [{ currency,
   bankAccountId, importId | null, inserted, duplicates }], payouts: [{ payoutId, currency, lines, net }] }`.
7. **AC-7 — Un seul chemin d'import de prestataire (DRY).** La ventilation par devise, la recherche des comptes, les
   refus de l'AC-4 qui ne dépendent pas du format, les verrous, l'empreinte, les doublons et le compte rendu sont **un**
   cœur commun qui reçoit des lignes neutres (`ProviderLine` : devise, dates, montant, identifiants, référence,
   contrepartie, e-mail, type, détails). **Ce cœur est celui que 38-11b, ou à défaut 39-11b (NFR-39-11b-2), a
   factorisé** : 40-7b l'**appelle** ; si, à la reprise, aucun des deux ne l'a fait, 40-7b le **déplace** hors de la
   route PayPal (module `crates/kesh-api/src/routes/provider_imports.rs` ou dans `kesh-db`, selon ce que 38-11b a
   laissé), la route PayPal l'appelle, tests de 38-11b **inchangés en assertions** — décision et état trouvé au Change
   Log (T0). Seuls restent propres à Stripe : le lecteur (40-7a), la traduction ligne → `ProviderLine` (fonction pure,
   testée sans base) et les transactions de versement.
8. **AC-8 — Sauvegarde, export, remise à zéro, i18n.** `.keshbackup` : la colonne passe seule (lecture de
   `information_schema`) ; aller-retour d'un compte Stripe avec compte de versement testé (`admin_full_import_e2e.rs`) ;
   une sauvegarde de la version précédente s'importe (colonne `NULL`, non obligatoire : la règle générique
   `missing_required_columns` ne refuse qu'une colonne obligatoire) — test. ⚠️ **`ABSENCES_ADMISES_EPIC_37`
   (`crates/kesh-db/src/backup.rs:170`) n'est pas touchée** : la liste est close, propre aux
   sauvegardes **antérieures à l'Epic 37** (doc de `:155-169`), et le test qui la rejoue
   (`full_import_of_a_0_17_6_backup_takes_the_multi_currency_defaults`, `admin_full_import_e2e.rs:5947`) en compte
   les colonnes (`retirees == 13`) : y inscrire cette colonne le ferait échouer, sans rien apporter — une colonne
   nullable passe déjà par la règle générique (même choix que 38-11b AC-7). *(Le motif écrit en P1 — un garde « ≥
   `PREMIERE_MIGRATION_EPIC_37` » — était faux : `20261120000003` le satisfait.)* Export CSV : colonne ajoutée à `serialize_bank_accounts_csv`
   (`crates/kesh-api/src/exports/csv_tables.rs:680`) ; `chaque_colonne_du_schema_est_exportee_ou_ecartee` (`:2658`)
   vert. `TABLES_TO_TRUNCATE` (`backup.rs:46`) inchangé (vidage sous `FOREIGN_KEY_CHECKS = 0` ; FK sur la même
   table). Codes neufs : `STRIPE_PAYOUT_ACCOUNT_INVALID`, `STRIPE_CSV_INVALID` : `errors.rs` et messages aux quatre
   locales (`crates/kesh-i18n/locales/*`).

## Tâches

- [x] T0 — Reprendre `main`, les phases 1 des Epics 38 et 39 fusionnées ; relire au code : contrainte `provider`
  (`'stripe'` admis ?), objet des réglages et codes (`PAYPAL_*` ou `PROVIDER_*`), prédicat « compte prestataire » des
  cinq refus, cœur d'import commun ; date de la migration ; preuve MariaDB de la FK `RESTRICT` + CHECK sur la même
  colonne.
- [x] T1 — Migration, squash, audit, test de migration (AC-1).
- [x] T2 — Entité, dépôt, réglages Stripe aux routes des comptes bancaires (AC-2, AC-3).
- [x] T3 — Chemin d'import commun, déplacé de PayPal s'il le faut (AC-7).
- [x] T4 — Route, ventilation, versements, doublons, compte rendu (AC-4 à AC-6).
- [x] T5 — Sauvegarde, export, i18n (AC-8).

## Stratégie de tests

- **Unitaires** (sans base) : traduction `StripePayoutRow` → `ProviderLine` (brut, frais, signe des frais rendus d'un
  remboursement, versement = − Σ net, versement de Σ net nul non créé, référence `source_id` puis `charge_id`).
- **HTTP** (`crates/kesh-api/tests/stripe_import_e2e.rs`, `#[sqlx::test]` sur le squash), fixture de 40-7a : société
  multi-devises, comptes CHF, EUR, USD d'un compte Stripe, comptes de versement ; import → trois entrées, nombres
  attendus ; réimport → 422 ; rapport par devise puis toutes devises → doublons ; fichier sans `source_id` → 422 avec la
  marche à suivre ; devise sans compte → 422, rien d'écrit ; compte clos → 409 ; clé d'API `read` → 403. Réglages :
  chaque refus de l'AC-2 (compte de versement Wise admis ; `stripe`, `paypal`, autre devise, archivé refusés ; changé
  malgré des transactions) ; les six `BANK_ACCOUNT_IS_PROVIDER` sur un compte Stripe. Tests de l'import PayPal (et
  Wise) relancés (AC-7).
- **Migration** : test de l'AC-1 ; `test_schema_guard.rs`. Gate **complet** (exception `kesh-db`).

## Modules touchés (4)

`kesh-db` (migration, entité, dépôt, sauvegarde), `kesh-api` (routes `bank_accounts`, `stripe_imports` neuve, chemin
commun, erreurs, export), `kesh-import` (lecture seule de 40-7a), `kesh-i18n` ; plus une phrase du manuel
d'administration (`.tex`).

## Note de version

*Ajouté* — Un compte bancaire peut être le solde, dans une devise, d'un compte Stripe : sans IBAN, lié au compte de
transit de la devise, avec son compte de frais et son compte de versement. Le rapport « Itemized payout reconciliation »
s'importe d'un geste, ventilé par devise ; les doublons sont ignorés.
*Migration* — `20261120000003_stripe_import`, non breaking : la sauvegarde d'avant la mise à jour reste exigée (base,
`.keshbackup`, dossier des documents) ; le retour à la version précédente passe par elle.

## Notes de développement

- Remplace l'ancienne 40-1 pour les réglages (`superseded` : coffre et connexion de 38-1/38-2, tous deux `deferred`).
- ⚠️ Risque d'horodatage : 38-11b prend `20261120000001` ; si l'Epic 38 renumérote, suivre. Taille : la fiche
  reste dans la nuit tant que le cœur commun existe déjà ; sinon la factorisation (AC-7) est le morceau à détacher. La base de Beat ne reçoit la
  migration qu'à une version publiée, ou en connaissance de cause (P8).
- Le compte de versement est un **compte bancaire** (et non un compte du plan) pour que 40-7c trouve la réception du
  versement (« Argent reçu de Stripe » côté Wise, Epic 39 ; ligne du relevé CAMT côté banque) sur le bon compte.

## Change Log

- 2026-11-20 — **amendement d'AC (demande de Beat, Project Lead de l'epic, relayée par l'orchestrateur)** : le libellé (`details`) des
  transactions importées suit le format de son tableur Crésus — `<Encaissement si category = charge, sinon category> Stripe de
  <gross> <DEVISE> -<fee>=<net> <DEVISE> BRUT - <balance_transaction_id> (<created>- <description>) - <payout_id>` ; frais :
  même format préfixé « Frais Stripe · » ; versement : « Versement Stripe <payout_id> <Σ net> <DEVISE> » (AC-5, amendé ; la 40-7c
  reprendra ce texte pour ses écritures). Réalisation : champ `created` (cellule telle qu'écrite) ajouté à `StripePayoutRow`
  (40-7a, lecteur, additif), `row_label` dans `stripe_imports.rs`, tests exacts (`les_libelles_suivent_le_format_du_tableur`,
  E2E). Choix : le signe de `-<frais>` est littéral (frais rendus : `--0.60`) ; description absente : `(<created>- )`.

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 2 HIGH, 5 MEDIUM, 2 LOW, remédiés
  (rapport `kesh-gate-logs/epic-39-40-kiss/validation/40-7b-compte-stripe-et-import-P1.md`). HIGH : l'AC-8 inscrivait la
  colonne à `ABSENCES_ADMISES_EPIC_37`, dont le garde exige une migration de l'Epic 37 (test rouge ; 38-11b AC-7 l'a
  écarté) ; migration ramenée à une colonne, sa FK et un CHECK, `chk_bank_accounts_provider` n'étant pas reposée (38-11b
  admet `'stripe'`, décision de l'orchestrateur), plus de branche conditionnelle. MEDIUM : le contrôle de « fichier
  filtré » annoncé ne voit rien (le total du versement vient des mêmes lignes) → seul garde-fou la réception, dit comme
  tel ; verrous par `with_account_locks` (38-11b) et non `create_under_account_lock` ; route en `Traced,
  SansEcritureAuJournal`, compteurs du registre et phrase du manuel d'administration ; codes `STRIPE_*` jumeaux retirés
  au profit de `PROVIDER_*` (39-11b), corps des réglages aligné sur l'objet générique ; garde du compte de versement
  (compte Wise admis, PayPal refusé), refus `BANK_ACCOUNT_IS_PROVIDER` jugés sur `provider IS NOT NULL` ; cœur d'import
  commun attendu de 38-11b/39-11b, non « déplacé » d'office ; paramètre `match` retiré. LOW : `CsvLineErrorPayload`
  cité à `errors.rs:1541` ; versement de Σ net nul non créé.
- 2026-11-20 — validation P2 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW, remédiés (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/40-7b-compte-stripe-et-import-P2.md`). Remédiation P1 vérifiée au code de
  `origin/main` (migration, FK, CHECK, P1–P8, verrous, registre, sauvegarde). LOW : le motif de l'AC-8 pour ne pas
  toucher `ABSENCES_ADMISES_EPIC_37` était faux (`20261120000003` satisfait le garde « ≥ première migration de l'Epic
  37 ») → la liste est close, propre aux sauvegardes antérieures à l'Epic 37, et son test compte treize colonnes ;
  « cinq refus » `BANK_ACCOUNT_IS_PROVIDER` pour six sites ; `reason` `closed` du compte de versement. Propagé de la
  40-7c : revue rechargée après l'import (et non une route de rapprochement), écart `payout_without_receipt`.
- 2026-11-20 — complément P2 (Claude Opus 5.5), sur relevé de la P2 de l'Epic 39 : l'AC-6 dit qu'il n'y a pas de règle
  de continuité pour Stripe (rapport sans solde), et qu'une règle future lirait le chevauchement comme la 39-11b
  (`overlapping_statements`, puis `previous_statement_closing`).

## Dev Agent Record

### Implémentation (développement du 2026-11-20 / 21, branche `story/40-7b`)

- **T0 (relu au code, sur `epic/40-stripe` + 38-11b)** : la contrainte `chk_bank_accounts_provider` admet déjà `'stripe'`
  (migration `20261120000001`) ; la phase 1 de l'Epic 39 (39-11b) **n'est pas** dans la branche : pas d'objet générique, pas de
  codes `PROVIDER_*` renommés au-delà de ceux de 38-11b (déjà génériques) ; les six refus `BANK_ACCOUNT_IS_PROVIDER` jugent
  `provider IS NOT NULL` (`is_provider()`), rien à unifier (C-40-7b-3) ; 96 migrations avant, 97 après. FK `RESTRICT` + CHECK
  sur la même colonne, rejeu hors sqlx et SQL de retour joués sur MariaDB 10.11 (`kesh-gate-logs/s40-7b/T0.log`).
- **T1** : `20261120000003_stripe_import.sql` (une instruction), squash régénéré (`scripts/regen-test-schema.sh`, 97 migrations),
  `migrations.sha384`, audit P5 (97 : en-tête et `Total` ; partitions 13 `yes` + 84 `tracked-by-sqlx`, recomptées), P6
  (`migrations_upgrade_path.rs` : `total == 97`, fenêtre `total - 63`, frontière 34 ; les autres sites résolvent par version),
  `account_columns_inventory.rs` (colonne `*_account_id` vers `bank_accounts`), `stripe_import_schema.rs`.
  ⚠️ **Écart à la fiche** : la CHECK s'écrit `payout IS NULL OR (provider IS NOT NULL AND BINARY provider = 'stripe')` ; la forme
  de la fiche laissait passer une destination sur un compte sans prestataire (CHECK NULL, satisfaite) (C-40-7b-4).
- **T2** : entité (`StripeSettings`, `ProviderSettings`, `payout_bank_account_id`), dépôt (`payout_destination_locked`,
  colonne aux `SELECT`/`UPDATE`), routes des comptes bancaires : corps `stripe` à côté de `paypal` (C-40-7b-1), **mêmes**
  fonctions de garde (`ProviderSettings`), garde du compte de versement `check_payout_destination_in_tx`, garde 5 étendue au
  changement de prestataire, audit.
- **T3** : cœur d'import commun. *(Première version : copies de `kesh-wt38p2`, état non commité du 2026-11-21 ; **reprise à la
  revue P1 (M1)** : voir « Fusion de la 38-12a ».)* Noms `ProviderLine`, `ImportRequest`, `ImportSource`, `load_balances`,
  `import_lines`, `CurrencyReport` ; ajouts de la 40-7b au socle : `ProviderLine.value_date: Option<NaiveDate>` (Stripe :
  `available_on`, pris par `into_new_bank_transaction`), `AUDIT_ACTIONS` + `debug_assert!`, section d'aides communes avant
  `mod tests` (`DETAILS_MAX_CHARS`, `NAME_MAX_CHARS`, `PROVIDER_TYPE_MAX_CHARS`, `FILENAME_MAX_BYTES`, `TXID_COLUMN_MAX_CHARS`,
  `FEE_SUFFIX`, `truncate_chars`, `truncate_bytes`, `bounded`, `ProviderImportForm`, `parse_form`) — `paypal_imports.rs` garde ses
  copies privées (à replier sur ces aides, dette de l'intégration de l'epic). Le paramètre futur `rules: Option<&dyn
  DuplicateRules>` (Wise) s'ajoutera à `ImportRequest` sans toucher Stripe (`None`).
- **T4** : `routes/stripe_imports.rs` (route `POST /api/v1/stripe-imports`, `lib.rs`), `lines_of_currency` (brut, frais `−fee`,
  versement `−Σ net`, Σ nul non créé), `check_id_lengths`, `csv_error` (hint `source_id|charge_id`), erreurs
  `STRIPE_CSV_INVALID` et `STRIPE_PAYOUT_ACCOUNT_INVALID`.
- **T5** : export CSV (`payout_bank_account_id`), sauvegarde (aller-retour et sauvegarde de la version précédente,
  `admin_full_import_e2e.rs` ; `ABSENCES_ADMISES_EPIC_37` non touchée), i18n quatre locales (3 clés), registre d'audit
  (160 routes, 152 tracées, 164 avec les routes de test), phrase du manuel d'administration (`.tex`, 151 des 159 ; 151 + 4 + 4
  = 159), `docs/api-external.md`.

### Tests ajoutés (recomptés depuis la source ; périmètre `git diff 046f39e9..HEAD`, 046f39e9 = 40-7a)

25 tests nouveaux (recomptés à la source après la reprise de la revue P1 : attributs de test de chaque fichier, 046f39e9 contre HEAD) : `kesh-db/tests/stripe_import_schema.rs` 2 (CHECK jamais NULL, octet, FK `RESTRICT`) ;
`kesh-api/tests/stripe_import_e2e.rs` 10 (dont, à la revue P1, `renommer_un_compte_stripe_dont_la_destination_est_archivee` (L2) et `retirer_stripe_d_un_compte_sans_transaction` (L3) ; ventilation et versements, rapport par devise puis tous, devise manquante / doublon de
devise / compte clos, refus de tête et clés, création et refus du compte de versement, modification, six refus
`BANK_ACCOUNT_IS_PROVIDER` + lot pain.001 + solde d'ouverture, cohabitation PayPal) ; `routes/stripe_imports.rs` 9 unitaires ;
`admin_full_import_e2e.rs` 3 (aller-retour du compte de versement, sauvegarde sans la colonne, destination d'identifiant supérieur) ; `routes/provider_imports.rs` 1
(`deux_lignes_identiques_voisines_sont_deux_groupes`). (`provider_imports.rs` porte 2 tests : le second, `doublon_dans_le_fichier_et_en_base`, vient de la 38-12a.) Étendus : tests d'entité (JSON `stripe`), export CSV, registres d'audit
(159 routes, 151 tracées, 163 avec les routes de test ; `SITES_INDIRECTS` à dix ; `ACTIONS` + `stripe_import.created`),
`bank_account_opening_balance_e2e` (clé `stripe`), `migrations_upgrade_path` (97 / 63).

### Défaut trouvé dans la copie du cœur de la 38-12a — sans objet depuis la fusion (M1, L6)

La copie non commitée de la 38-12a groupait par **suites consécutives** de même `group_id` (`dedupe_lines`) : deux lignes
identiques côte à côte formaient un groupe ; la 40-7b l'avait corrigé dans sa copie. La 38-12a commitée a **changé de forme**
(`ProviderGroup`, `dedupe_groups` : un groupe = une ligne de la source) : le défaut n'y existe pas, le correctif disparaît avec la
copie, et le comportement PayPal d'avant (L6, deux lignes de même identifiant voisines, brut nul et frais seuls) est retrouvé.

### Fusion de la 38-12a (revue P1, M1)

`story/38-12a` fusionnée dans `story/40-7b` (commit de fusion ; **têtes de 38-12a fusionnées : `1b791613`, puis `1022e4cf`** (reprise faite à la remédiation P1 : `9bbf1388` ramène `AUDIT_ACTIONS` à deux actions chez elle, résolu à l'union des trois ; `dedupe_groups` seule, sans le défaut des doublons voisins) ; **une reprise de la 38-12a reste à refaire avant la fusion de l'epic si elle avance encore**). On adopte **sa** forme : `ProviderGroup`,
`dedupe_groups`, `by_currency: BTreeMap<String, Vec<ProviderGroup>>`, `provider_lines` de `paypal_imports.rs` rend des groupes ;
`stripe_imports.rs::lines_of_currency` produit un groupe par ligne du rapport (brut + frais) et un groupe d'une ligne
`payout:<id>` par versement non nul. Conflits résolus en gardant les deux intentions : `provider_imports.rs` (add/add : la
forme de la 38-12a + `value_date`, `AUDIT_ACTIONS`, `debug_assert!`, aides), `paypal_imports.rs` (la 38-12a + `value_date:
None`), `lib.rs` (routes Stripe et `paypal-downloads`), `errors.rs`, registres d'audit, quatre `messages.ftl`,
`kesh-import/src/lib.rs` (`paypal_api`, `stripe`). ⚠️ **Piège sans conflit de texte** : `AUDIT_ACTIONS` ne portait que
`paypal_import.created` et `stripe_import.created` alors que la 38-12a écrit `paypal_download.created` par le même cœur : un
test debug du téléchargement aurait paniqué au `debug_assert!` ; les trois actions sont unies (`AUDIT_ACTIONS`, site indirect
de `audit_label_registry`, `ACTIONS`). **Compteurs recomptés** : le registre des routes porte `LIB_ROUTES` 160 (159 + le
téléchargement), 152 tracées, 164 avec les routes de test, partition « sans écriture » 131 — ⚠️ git avait fusionné « 130 » sans
conflit, valeur que **les deux** branches avaient portée depuis 129 : la somme est 131 ; les deux phrases du `.tex`
d'administration : `152 des 160` et `152 + 4 + 4 = 160` ; migrations : 97 (la 38-12a n'en ajoute pas ; la migration Stripe garde
son horodatage `20261120000003`, `total == 97`, P6 inchangé) ; `sitesTotal` : frontend non touché.

### Mutations (sondes négatives rejouées après la reprise de la revue P1, `kesh-gate-logs/s40-7b/mut2.log`, fichier restauré après chacune)

Sept sur sept rouges, par panic de test et non erreur de compilation : garde de devise du compte de versement désactivée →
`creer_un_solde_stripe...` ; `in_source` des doublons voisins forcé à faux (`dedupe_groups`) →
`deux_lignes_identiques_voisines_sont_deux_groupes` ; signe des frais inversé → `brut_et_frais_chacun_de_son_signe` ; signe du
versement inversé → `le_versement_reprend_la_somme_des_nets` ; borne du libellé retirée → `une_longue_description_ne_coupe_pas_la_fin_du_libelle` ;
dispense `payout_unchanged` retirée → `renommer_un_compte_stripe_dont_la_destination_est_archivee` ; retrait de Stripe désactivé →
`retirer_stripe_d_un_compte_sans_transaction`. Non mutée : la CHECK `provider IS NOT NULL` (éprouvée par `stripe_import_schema.rs`).

### Gates — dernier commit de code `c5326214` (reprise de la revue P1), cible propre `kesh-targets/kesh-wt40b`, `CARGO_INCREMENTAL=0`

Journaux sous `kesh-gate-logs/s40-7b/`. Les gates de la passe précédente (`f30e65d0`) sont sans valeur de clôture.

- Gate ciblé (`r2-cible.sh`, fmt + sept binaires d'intégration et registres) : vert. Un premier passage a rougi
  `retirer_stripe_d_un_compte_sans_transaction` (mon test : un compte qui perd Stripe redevient bancaire et exige son IBAN) : test corrigé.
- `scripts/test-fast.sh --ci` (fmt, clippy `-D warnings`, nextest ; base remise à zéro) : **6313 exécutés, 6313 réussis, 14 ignorés** ;
  `cargo publish --dry-run -p kesh-import` vert. Un premier passage a rougi clippy (`too_many_arguments`, `update_once` à huit
  paramètres) : `#[allow]` motivé, passage rejoué.
- `scripts/gate-ci.sh` (copie `git archive` de `c5326214`) : **6332 passés, 0 échoué, 18 ignorés**, rc 0.
- E2E complet (`LANG=fr_CH.UTF-8`, port 3417) : **477 réussis, 10 échecs, 18 ignorés** (`e2e-complet-final.log`) : les sept de la
  KF-029 et, comme à la passe précédente de la story, `backup-restore-new-install.spec.ts:262` et `:381` (bouton d'import
  désactivé) et `opening-balances.spec.ts:156` (un toast recouvre le bouton). Rejoués seuls sur une base neuve : les trois
  rougissent encore. ⚠️ **Leur cause n'est pas établie** : ils étaient déjà rouges avant la reprise, aucun fichier du frontend ni de
  ces routes n'est touché par la story, mais le rejeu sur `main` n'a pas été fait ; à confirmer (issue à ouvrir si le rouge vient de la branche).
- Frontend : aucun fichier de `frontend/` modifié depuis le passage vert antérieur (`npm run check`, `lint-i18n-ownership`,
  `test:unit` 4612 tests, `build`).

Incidents d'infrastructure, sans rapport avec la story : un watcher de fond tué par la pression mémoire de la station (sans effet sur la chaîne).

### Limites et points hors périmètre relevés

- Cible cargo `kesh-targets/kesh-wt40b` supprimée à la clôture.
- Archiver (non supprimer) un compte désigné comme destination de versement n'est pas refusé : la garde `archived` ne vaut qu'à
  la pose, et un renommage ne la rejuge plus (revue P1, L2) ; la destination se relit à l'usage (40-7c) ; à tracer en issue.
- Aucun contrôle ne voit un rapport filtré par catégorie (écrit à la fiche et à l'API ; le manuel utilisateur relève de la 40-7d).
- Manuel utilisateur (`.tex`) : non touché (écrans et manuel : 40-7d) ; manuel d'administration : une phrase (compte du registre).

## Change Log

- 2026-11-21 — développement de la story (Claude Sonnet 5.5) : migration, réglages Stripe, cœur d'import commun aligné sur la
  38-12a, route d'import Stripe, sauvegarde, API, registre, i18n ; statut `review`.

- 2026-11-21 — remédiation de la revue P1 (Claude Sonnet 5.5) : reprise de la 38-12a (`1b791613`, puis `1022e4cf`), M1, L1 à L6 ; gates rejoués au dernier commit de code `c5326214`.

## Notes de version

*Ajouté* — Un compte bancaire peut être le solde, dans une devise, d'un compte Stripe : sans IBAN, lié au compte de transit de
la devise, avec son compte de frais et son compte de versement. Le rapport « Itemized payout reconciliation » s'importe d'un
geste (`POST /api/v1/stripe-imports`), ventilé par devise ; les doublons sont ignorés. Le libellé de chaque transaction suit le
format du tableur de Crésus de l'utilisateur (identifiants `txn_…` et `po_…` cherchables).
*Migration* — `20261120000003_stripe_import`, non breaking : la sauvegarde d'avant la mise à jour reste exigée (base,
`.keshbackup`, dossier des documents) ; le retour à la version précédente passe par elle.


### Clôture (2026-11-21, orchestrateur)

Boucle de revue : P1 Sonnet (0/0/1 MEDIUM/7 LOW) → remédiation (fusions de `story/38-12a`, L1–L6) → P2 Opus (0/0/0/8 LOW,
`kesh-gate-logs/s40-7b/revue-p2-rapport.md`). Gates au dernier commit de code `c5326214` : `test-fast --ci` 6313 / 14
ignorés ; `gate-ci.sh` 6332 / 0 / 18 ; mutations 7/7 rouges ; Vitest sur l'état fusionné 4612 (278 fichiers, passe P2) ;
E2E 477 / 10 / 18 — les 10 sont les attendus (sept KF-029 #97, `backup-restore-new-install.spec.ts:262` et `:381`
KF-065 #1282, `opening-balances.spec.ts:156` KF-067 #1284 ; correction de la mention « cause non établie » ci-dessus).
Décomptes corrigés par la P2 : 24 tests neufs (non 25) ; compteurs du registre 160 / 152 / 164. LOW de la P2 en dette pour
la story de solde 40-9 : borne de la catégorie dans le libellé (1), reprise de la nouvelle tête de `story/38-12a` (2 :
`AUDIT_ACTIONS` en tranche, `check_audit_action`, `period_windows`, `run_bounded`, `classify_*`, `parse_api_amount`,
`hyper`), angle mort de la destination élargi au changement de son compte du plan (3), deux lignes « PayPal » de
`api-external.md` (4), copies privées des aides dans `paypal_imports.rs` et `bounded` sans appelant (5). Issues à ouvrir :
`kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`.
