# Story 34-11a1 : un compte sans IBAN — le schéma, et tous les sites qui lisent l'IBAN le tolèrent (#972)

Status: done

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db` (inchangés à `b85b8f05`, commit de fiches seules). Release **0.17.6**, Epic 34. **Une migration, breaking**
(`kesh_version_min_required` → 0.17.6, C-e341-14) — **la seule de l'epic** (la 34-5 n'en porte pas, C-e343-1) : P1 à P8 du `CLAUDE.md`
s'appliquent. Registre : `e34-choix-autonomes-e341.md`, C-e341-13, 14, 17, 18, 19, 20, 21, 29, 39. Première moitié de l'ancienne 34-11a (découpée à la
validation P1, arbitrage de l'orchestrateur) ; la seconde est la **34-11a2** (`34-11a2-carte-credit-creation.md`), qui ouvre la création d'une
carte. Vient après **34-3**, **34-1**, **34-2**. Commits en `refs #972` ; à la PR : `refs #972`.*

## Story

En tant que mainteneur de Kesh, je veux que le schéma admette un compte sans IBAN et que **chaque** site qui lit l'IBAN d'un compte bancaire le
refuse ou l'affiche proprement, **avant** qu'aucun écran ne permette d'en créer un, pour que la 34-11a2 ouvre la carte de crédit sur un dépôt
déjà sûr, et que la migration breaking se joue seule sur une base 0.17.5 peuplée.

## Décisions de Guy (#972, fichier d'epic, 2026-11-10)

Une carte est tenue **comme un compte**, **sans IBAN**, liée à un compte de dette ; seuls les **quatre derniers chiffres** sont gardés ; le type
« sans IBAN » est pensé assez général (PayPal, caisse) ; **aucun QR-IBAN, aucun `pain.001`** depuis ce compte. ⛔ Le numéro d'un compte du plan ne
déduit jamais un rôle (14-3a) ; un appariement automatique propose, il ne crée jamais (rien ici n'apparie).

## Périmètre de cette moitié

- **Dans** : la migration entière (IBAN nullable, `account_kind`, `card_last4`, `CHECK`s), le bump, l'entité et le dépôt, les refus des sites A,
  la tolérance des sites B (backend et écran), l'audit, l'export, l'empreinte, la documentation de la migration.
- **Hors** (34-11a2) : toute route ou tout écran qui **crée** ou **modifie** un compte sans IBAN ; Luhn ; lien passif ; retypage ; manuel de la carte.
- ⚠️ Après cette moitié, un compte sans IBAN n'existe **que** forgé en base par un test : `POST` et `PUT /bank-accounts` exigent toujours l'IBAN
  (`routes/bank_accounts.rs:79`, `:99`). Ce que fait un `accountKind` envoyé (refusé ou ignoré) se constate à l'étape 0 et s'asserte (AC-11).

## Ce que le code dit (prémisses établies)

- **Le verrou** : `bank_accounts.iban` est `VARCHAR(34) NOT NULL` avec `chk_bank_accounts_iban_nonempty`
  (`migrations/20260410000001_bank_accounts.sql:8`, `:16` ; `test-schema/0001_schema_squash.sql:156`, `:171`). **Aucun index unique** sur l'IBAN.
  L'entité porte `iban: String` (`entities/bank_account.rs:39`, `:81`).
- **Migration probable, confirmée** : 90 migrations (`ls crates/kesh-db/migrations/*.sql | wc -l`), aucune ne touche `iban` depuis la création.
- **Le précédent** : `20261011000001_bank_profiles_debit_sign.sql` (colonne `VARCHAR` + `CHECK`, bump, entrée `EXEMPT_MIGRATIONS`, ligne P5).
- ⚠️ **Les assertions codées en dur que le bump fait rougir** (validation P1, 3.3 ; `git grep -n "0\.17\.0\"" -- 'crates/*/tests/*'`, relu) :
  `admin_full_import_e2e.rs:4649` (doc), `:4669`, `:4816`, `:4845`, `:4848`, `:5106` ; `migrations_fresh_install.rs:248` ;
  `migrations_upgrade_path.rs:549`, `:565`, `:581`, `:600`, `:683`, `:1122`, `:1622` ; et `post_restore.rs:1241` (`EXEMPT_MIGRATIONS.len()`
  comparé à **23**, historique en commentaire `:1210-1235`). Certains désignent l'état d'une base **0.17.0** (à garder), d'autres le minimum
  **courant** (à porter à 0.17.6) : le tri est une tâche (T2), sa sortie va au Dev Agent Record.

### L'inventaire clos des sites qui lisent l'IBAN d'un compte bancaire

Établi en partant du symptôme ; la commande exacte (T1) rend **135** lignes au `b85b8f05` (« 134 » écrit à la création de la fiche était faux :
le code n'a pas changé entre `a75835db` et `b85b8f05`, recompté à la validation P1). Les lectures de `creditor_iban`, `counterparty_iban` et de la
règle `iban_exact` portent sur d'autres colonnes. Classes : **A** refuse un compte sans IBAN ; **B** l'accepte et l'affiche autrement ; **C**
inatteignable (justifié). Chaque site est résolu par un AC de **cette** fiche, ou renvoyé nommément à la 34-11a2, ou écrit angle mort.

| # | Site | Ce qu'il fait | Classe | Résolu par |
|---|---|---|---|---|
| S1 | `repositories/payment_batches.rs:862-869` `generate_pain001_xml` | `SELECT iban` dans un tuple `(String,)` → `<DbtrAcct><IBAN>` (`kesh-payment/src/pain001/mod.rs:196-201`) | **A** — ⚠️ invisible au compilateur : échec de décodage, **500**, le lot déjà créé | AC-5 |
| S2 | `repositories/payment_batches.rs:157-175` `create_batch` | contrôle archivé, clos, lié — **ne lit pas l'IBAN** | **A** | AC-5 |
| S3 | `routes/invoice_pdf_service.rs:223`, `:332-342` | QR-facture : le **principal** (`find_primary`, `repositories/bank_accounts.rs:104`) | **A** | AC-4 (`CHECK` : sans IBAN ⇒ jamais principal) |
| S4 | `routes/bank_imports.rs:940-972` `select_statement_by_iban` ; `:1060`, `:1313`, `:1112`, `:1376` (alias 0.17.3, `:2087-2120`) | CAMT : relevé choisi par l'IBAN | **A** | AC-6 |
| S5 | `routes/bank_import_fingerprint.rs:132` | IBAN dans l'empreinte (CAMT **et** CSV) | **B** | AC-7 |
| S6 | `routes/bank_imports.rs:2341`, type `:247` | aperçu CSV `account_iban: String` | **B** | AC-7 |
| S7 | `routes/bank_accounts.rs:674`, `:850`, `:859` ; `routes/onboarding.rs:669`, `:681` | audit `"iban_present": true` **codé en dur** | **B** | AC-8 |
| S8 | `routes/onboarding.rs:579`, `:603-604`, `:628` | l'assistant crée le **principal**, IBAN obligatoire | **A**, inchangé (C-e341-19) | — |
| S9 | `routes/companies.rs:418`, `:431` | `GET /companies/current` | **B** | AC-3 |
| S10 | `exports/csv_tables.rs:647`, `:664` | export `bank_accounts.csv` | **B** | AC-9 |
| S11 | `repositories/bank_accounts.rs:54`, `:65-70`, `:254`, `:352-356`, `:382-387` ; et `:658-662` (`update_for_company`, `iban = ?` depuis `NewBankAccount.iban: String`) | lectures et écritures neutres pour un compte bancaire ; `:658-662` est le chemin du `PUT` d'une carte | **B** | AC-3 ; `:658-662` : **34-11a2** (AC-4, T4) |
| S12 | `frontend/…/bank-accounts/format.ts:16-18` `shortIban` | `iban.length` — **`TypeError` sur `null`** | **B** | AC-10 |
| S13 | `format.ts:62`, `:87-92` `bankAccountOptionLabel` et ses sept appelants (`BankAccountJournalLinkForm.svelte:135`, `BankImportUpload.svelte:372`, `ManualStatementForm.svelte:346`, `SettleInvoiceDialog.svelte:249`, `payment-batches/+page.svelte:162`, `reconciliation/+page.svelte:68`, `supplier-invoices/[id]/+page.svelte:888`) | « Nom — IBAN » | **B** (+ **A** à l'écran : lots, CAMT) | AC-10, AC-5, AC-6 ; qui **propose** une carte (règlement client, fournisseur) : 34-11a2 |
| S14 | `homepage/BankAccountsCard.svelte:39` | `shortIban` sur l'accueil | **B** | AC-10 |
| S15 | `routes/(app)/bank-accounts/+page.svelte:209`, `:303`, `:328`, `:542`, `:653`, `:754` | formulaire et tableau | **B** (tableau) ; le formulaire : 34-11a2 | AC-10 |
| S16 | `routes/(app)/settings/+page.svelte:607` ; types `bank-accounts.api.ts:28`, `:83`, `:105`, `:210`, `:239`, `settings.types.ts:72` | affichage, types | **B** | AC-10 |
| S17 | `BankImportUpload.svelte:513`, `bank-import.types.ts:5`, `:15` | IBAN du relevé choisi | **B** | AC-7 |
| S18 | `docs/api-external.md:233`, `:245`, `:788`, `:790`, `:798` | contrat d'API | **B** | AC-13 |
| C1 | serveur MCP (`crates/kesh-api/src/mcp/**`) | ne rend que `compte_bancaire_id` (`outils/factures_fournisseur.rs:224`) | **C** | — |
| C2 | sauvegarde (`kesh-db/src/backup.rs:283-341`, `:593-611`) | générique, `NULL` ↔ `Value::Null` ; le retour vers une 0.17.5 est fermé par le bump | **C** | AC-12 |
| C3 | réconciliation (`kesh-reconciliation`, `routes/reconciliation*.rs:2541`, `:3506`, `:3896`) | `journal_account_id` seul | **C** | — |
| C4 | relevé saisi (`routes/bank_statements_manual.rs`, `kesh-core/bank_imports/manual.rs`) | `counterparty_iban` seul | **C** | — |
| C5 | clôture (`closing_checks.rs:444-470`), règlements (`invoice_settlements_write.rs:170`, `supplier_invoices.rs:2010`, `:2268`) | `bank_name`, `description`, `journal_account_id`, `closed_on`, `archived` | **C** | — |
| C6 | `kesh-report`, `kesh-seed`, `post_restore/20260828000001…:26` | aucune lecture | **C** | — |

**Angles morts écrits** : (i) le flux aval de `kesh-qrbill` n'est pas lu (S3 l'éteint en amont) ; (ii) les messages `.ftl` qui disent « IBAN »
ne sont pas passés en revue ; (iii) **les tests** : ≈ 316 lignes Rust sous `crates/*/tests/`, plus les blocs `#[cfg(test)]` de production
(`backup.rs:1613`, `:1650`, `repositories/invoices.rs:4368`, `exports/csv_tables.rs:1825`, `bank_import_fingerprint.rs:190`, `:290`, `:327`,
`bank_imports.rs:2729-2905`), 66 Vitest, 64 Playwright : ils changent de **type** (`Option`), non de sens ; les `INSERT INTO bank_accounts
(… iban …)` des tests d'intégration restent valides (la colonne garde son nom, `account_kind` a un défaut).

## Choix (détail au registre)

- **C-e341-13** — colonne `account_kind` (`'bank'` | `'credit_card'`, `DEFAULT 'bank'`) : le type se déclare.
- **C-e341-14** — breaking, bump à 0.17.6, Cargo du workspace en 0.17.6 dans la même PR (P2-bis).
- **C-e341-17** — invariants au schéma : `bank` ⇒ `iban IS NOT NULL` ; `credit_card` ⇒ `iban IS NULL AND qr_iban IS NULL AND is_primary = FALSE` ;
  `card_last4 IS NULL OR (account_kind = 'credit_card' AND card_last4 REGEXP '^[0-9]{4}$')`.
- **C-e341-18** — refus nommés aux sites A, fondés sur **`iban IS NULL`** (non sur le type) : un futur type sans IBAN y tombe sans changement.
- **C-e341-29** — une seule migration pour les deux moitiés : la 34-11a2 n'en porte pas.
- **C-e341-20** — *remplacé* : la dérogation au splitting n'a pas été reçue (validation P1, 3.1) ; la story est découpée.

## Acceptance Criteria

1. **AC-1 — la migration** (`<horodatage>_bank_accounts_without_iban.sql`) : `iban` nullable, `account_kind`, `card_last4`, les `CHECK`s de
   C-e341-17 ; dernière instruction `UPDATE _kesh_version SET kesh_version_min_required = '0.17.6' WHERE id = 1;` ; crates en 0.17.6. Schéma de
   test régénéré (`test_schema_guard.rs` vert). Une base 0.17.5 **peuplée** monte sans écart : empreintes identiques hors colonnes neuves, chaque
   compte `bank`.
2. **AC-2 — P5 à P7, et les assertions du bump** : ligne P5, total et partitions **recomptés** (`ls … | wc -l` = 91 = `grep -c '^| \`20'`) ; P6
   inspecté ; entrée `EXEMPT_MIGRATIONS` justifiée, `post_restore.rs:1241` porté de **23 à 24** avec sa ligne d'historique (`:1210-1235`) ; les
   sites `"0.17.0"` listés en prémisse **triés** (gardés : état d'une base 0.17.0 ; portés à 0.17.6 : minimum courant), la sortie du tri au Dev
   Agent Record, `\b0\.17\.6\b` grepé après.
3. **AC-3 — entité et dépôt** : `iban: Option<String>`, `account_kind: BankAccountKind`, `card_last4: Option<String>` ; `columns!()` les lit ;
   `GET /bank-accounts` et `GET /companies/current` rendent `iban: null` pour une ligne forgée sans IBAN, et **désormais `accountKind` et `cardLast4`
   pour toute ligne** (le `Serialize` de l'entité les fait sortir, réponse du `POST` comprise) : changement additif, écrit à `api-external.md`.
   `NewBankAccount.iban` **reste `String`** (C-e341-39 : il ne sert qu'aux créations de comptes bancaires, assistant, dépôt, 68 sites de test) ; la
   34-11a2 lie l'IBAN d'une carte à part, comme la description. `invoice_pdf_service.rs:342`, sur un principal sans IBAN (inatteignable par le `CHECK`),
   rend une erreur interne (`AppError::Internal`), jamais `unwrap()` ni `expect()`.
4. **AC-4 — le principal** : une ligne forgée `credit_card` + `is_primary = TRUE` est refusée par la base ; une QR-facture ne peut donc pas lire
   un IBAN nul (test du `CHECK`).
5. **AC-5 — pain.001** : `POST /payment-batches` sur un compte sans IBAN → `409 PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN`, rien d'écrit ;
   `generate_pain001_xml` lit `Option<String>` et rend le même refus, sans `500` (lot forgé en base). L'écran des lots ne propose pas les comptes
   sans IBAN.
6. **AC-6 — CAMT** : sur un compte sans IBAN, aperçu et import CAMT → `422 BANK_IMPORT_CAMT_NEEDS_IBAN`, placé **après** la détection du format
   (`detect_import_format`, `bank_imports.rs:1040`) et **avant** `parse_camt053` (`:1055`), et de même dans `create` avant `:1313` ; CSV et saisie
   manuelle **acceptés** sur le même compte (test : CAMT refusé, CSV accepté). L'écran d'import n'offre pas le CAMT pour un tel compte.
7. **AC-7 — empreinte et aperçu CSV** : un IBAN absent s'encode distinctement de toute chaîne, stable ; un compte bancaire garde **la même**
   empreinte qu'en 0.17.5 (valeur figée) ; `PreviewStatement.accountIban` devient `string | null`.
8. **AC-8 — audit** : `iban_present` vaut `iban.is_some()` aux cinq sites S7 (les deux de l'assistant restent vrais par construction).
9. **AC-9 — export** : `bank_accounts.csv` : `iban` vide pour un compte sans IBAN ; colonnes `account_kind` et `card_last4` **en fin de ligne**.
10. **AC-10 — l'écran tolère** : tableau, accueil, *Paramètres* et les sept sélecteurs affichent un compte sans IBAN sans planter (« •••• 1234 » si
    `cardLast4`, le seul nom sinon) — **une** fonction de `format.ts` (C-e341-21) ; Vitest sur `shortIban` gardé et `bankAccountOptionLabel`.
    **Le filet qui ferme l'inventaire côté écran** : `iban: string | null` dans `bank-accounts.api.ts:28` et `settings.types.ts:72` fait rougir
    `npm run check` sur tout lecteur TypeScript ou Svelte non adapté — la seule preuve que l'écran n'a pas de lecteur hors de S12-S17 (le grep reste une
    liste) ; `npm run check` vert est exigé et déclaré au Dev Agent Record.
11. **AC-11 — rien ne crée encore un compte sans IBAN** : `POST /bank-accounts` sans IBAN → `400` comme en 0.17.5 ; un `accountKind:
    "credit_card"` envoyé ne crée pas de carte (refusé ou ignoré : le comportement constaté à l'étape 0 est asserté).
12. **AC-12 — sauvegarde** : un `.keshbackup` 0.17.6 qui contient une ligne forgée sans IBAN se réimporte à l'identique dans une 0.17.6 neuve ; il
    est refusé par une 0.17.5 en le nommant (`routes/admin.rs:235-241`) ; un `.keshbackup` 0.17.5 s'importe dans une 0.17.6 (comptes `bank`).
13. **AC-13 — documentation de la migration** : notes de version et manuel d'administration — **la sauvegarde d'avant la mise à jour en tête**,
    le retour à la 0.17.5 par cette sauvegarde seulement ; `docs/api-external.md` (S18, `iban` nullable, refus des sites A, empreinte) ; les codes
    neufs au tableau d'erreurs. PDF régénérés, aplatis.

## Tâches

- [x] **T0 — étape 0** : constater ce que fait aujourd'hui un `accountKind` envoyé à `POST /bank-accounts` (AC-11), et relancer la commande de T1.
- [x] **T1 — rouges d'abord** (⚠️ rouges établis après coup par mutation de retour, voir Dev Agent Record), inventaire rejoué (sortie triée au Dev Agent Record ; tout site neuf classé avant le code) :
  ```
  git grep -n -E "(\.|\b)iban\b" -- 'crates/*/src/**' 'frontend/src/**' ':!*.test.ts' ':!crates/kesh-core/src/types/**' ':!crates/kesh-qrbill/src/**' ':!crates/kesh-import/src/**' ':!crates/kesh-reconciliation/src/**' | /usr/bin/grep -v -E "creditor_iban|counterparty_iban|qr_iban|qrIban|creditorIban|is_qr_iban|//|^\S+:[0-9]+:\s*\*"
  ```
  Rouges : AC-4, AC-5 (le 500 d'aujourd'hui), AC-6, AC-7 (valeur figée), AC-8, AC-12, Vitest AC-10.
- [x] **T2 — migration**, schéma de test, P5, P6, P7, bump Cargo, tri des `"0.17.0"` et compteur d'exemptions (AC-2). ⚠️ **gate complet même en
  cours de boucle** (`kesh-db`) ; démarrage réel sur une base persistante (P8).
- [x] **T3 — tolérance** : entité, dépôt, S1/S2, S4, S5, S6, S7, S9, S10 ; frontend S12 à S17.
- [x] **T4 — codes d'erreur** : `PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN`, `BANK_IMPORT_CAMT_NEEDS_IBAN` : variantes `AppError`, `errors.rs`, messages
  aux quatre `messages.ftl`, libellés frontend, tableau d'erreurs de `api-external.md`.
- [x] **T5 — montée** (recette et complément, voir Dev Agent Record) : `scripts/recette-montee.sh` depuis l'image 0.17.5 publiée, base peuplée (comptes, imports, lots, factures QR). La recette
  constate (précédent de la 0.17.5) : le fichier `backup/pre-upgrade/` écrit par Kesh **avant** la migration (#847) ; l'image 0.17.5 qui **refuse**
  de démarrer sur la base migrée ; le fichier `pre-upgrade/` importé dans une 0.17.5 neuve, empreintes identiques à l'avant ; un `.keshbackup` 0.17.5
  importé dans une 0.17.6. Et un **test automatisé** dans `migrations_upgrade_path.rs` : lignes `bank_accounts` posées avant la migration, puis
  `account_kind = 'bank'`, IBAN intact, `CHECK`s satisfaits — montage **résolu par version** (`.position(|m| m.version == …)`, P6), non par position.
- [x] **T6 — documentation** (AC-13), CHANGELOG (migration, avertissement de sauvegarde en tête), captures `kesh-gate-logs/e341/captures/34-11a1-*.png`
  (tableau et accueil avec une ligne forgée).

### Mutations nommées

- **M1** : le refus de `create_batch` retiré → rouge AC-5 ; **M2** : `generate_pain001_xml` remis en `(String,)` → rouge AC-5 (`500`).
- **M3** : le refus CAMT retiré → rouge AC-6 ; **M3b** : le refus placé avant la détection du format → rouge « CSV accepté ».
- **M4** : `None` encodé `""` dans l'empreinte → rouge AC-7 ; **M5** : forme changée pour un compte bancaire → rouge AC-7 (valeur figée).
- **M6** : `iban_present: true` rétabli → rouge AC-8.
- **M7** : `CHECK` « sans IBAN ⇒ non principal » retiré → rouge AC-4.
- **M8** : `shortIban` sans garde → rouge Vitest AC-10.

## Gates

`scripts/test-fast.sh` à chaque patch (`kesh-db`), gate frontend complet, E2E complet au dernier commit de code, démarrage réel sur base persistante
peuplée (P8).

## Manuels

Administration et notes de version, `api-external.md` (AC-13). Le manuel utilisateur de la carte : 34-11a2. **Prompt de revue : les nommer**, PDF aplatis.

## Ce qui ne se traite pas — issue à demander

- **Un relevé CAMT d'un compte sans IBAN** (`Acct/Id/Othr`) : le parseur l'exige en IBAN (`kesh-import/src/camt053/mod.rs:499-501`, `:827-829`).
  **Issue demandée** (« Import CAMT : relevé d'un compte sans IBAN, `Acct/Id/Othr` »).

## Recoupements

`routes/bank_accounts.rs`, `format.ts`, `bank-accounts/+page.svelte` avec **34-1**, **34-2**, **34-11a2** ; `bank_imports.rs` avec **34-9**. Seule
migration de l'epic : aucun ordre à régler avec une autre story. La **34-11a2** et la **34-11b** s'appuient sur celle-ci.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5). Inventaire établi par une recherche dédiée (lecture seule), sites A et la forme du schéma recontrôlés à la
  main (`payment_batches.rs:858-869`, `20260410000001_bank_accounts.sql:5-17`, `bank_imports.rs:940-950`, `invoice_pdf_service.rs:220-226,330-344`,
  `iban_present` aux cinq lignes, `payment_batches.rs:155-176`).
- 2026-11-10 : **validation P1 (Sonnet)** — 0 C, 1 H, 10 M, 16 L sur le groupe ; pour l'ancienne 34-11a : 4 M (3.1 à 3.4), 5 L (3.5 à 3.9).
  Remédiation (Opus 5.5). Fichier renommé de `34-11a-carte-credit-compte.md` (`git mv`) ; **3.1** : découpe sur arbitrage de l'orchestrateur, cette
  fiche garde la migration et la tolérance, la création passe en 34-11a2 (C-e341-20 remplacé, C-e341-29) ; **3.3** : assertions `"0.17.0"` et
  `EXEMPT_MIGRATIONS.len()` nommées (relu : `git grep -n "0\.17\.0\"" -- 'crates/*/tests/*'`, `post_restore.rs:1241`, 23) ; **3.5** : 135 et non 134
  (recompté), blocs `#[cfg(test)]` ajoutés à l'angle mort (iii) ; **3.6** : refus CAMT placé entre détection et lecture (relu `bank_imports.rs:1040-1055`),
  mutation M3b ; **3.7** : tâche T4 des codes ; refus des sites A fondés sur `iban IS NULL` (C-e341-18 précisé). **3.2, 3.4, 3.8, 3.9** : en 34-11a2.
  LOW de la validation du groupe e343 (orchestrateur) : la fiche ne parle plus d'une migration à ordonner avec la 34-5, qui n'en porte pas (C-e343-1).
- 2026-11-10 : **validation P2 (Opus)** — pour cette fiche : 0 M, 5 L (3.1 à 3.5). Remédiation (Opus 5.5), tout appliqué. **3.1** : en-tête complété
  (C-e341-19, 21). **3.2** : `NewBankAccount.iban` reste `String`, la carte lie son IBAN à part (C-e341-39 ; relus `entities/bank_account.rs:81`,
  `routes/bank_accounts.rs:642`). **3.3** : `accountKind`, `cardLast4` dans les réponses, et `invoice_pdf_service.rs:342` sur `None` (AC-3). **3.4** :
  ce que la montée constate, et le test de montée résolu par version (T5). **3.5** : `npm run check` comme filet de l'inventaire (AC-10).
- 2026-11-10 : **validation P3 (Sonnet)** — pour cette fiche : 1 L (3.1). Remédiation (Opus 5.5) : S11 `:658-662` (`update_for_company`) renvoyé à la
  34-11a2, chemin du `PUT` d'une carte (relu `repositories/bank_accounts.rs:600-612`).
- 2026-11-10/11 : **développement (Opus 5.5)**, worktree `/home/gcorbaz/devel/kesh-e34a1`, base `07f714a2`. Commits `194faa78` (migration, bump,
  code, tests, documentation), `0d7aa7cc` (registre des checksums, prémisse du test des quatre chiffres), `661a2a01` (fixtures, Vitest des écrans,
  compteurs des gardes i18n), `93e2642f` (manuel, PDF, test AC-12 à l'import), puis ce compte rendu. Choix : `e34-choix-autonomes-e34a1.md`,
  C-e34a1-1 à 10. Statut `review`.
  - **Étape 0 — lignes lues.** LOW de la dernière validation qui couvre la fiche (P3, 3.1) : déjà appliqué (S11 renvoyé à la 34-11a2). AC-11 :
    `CreateBankAccountBody` (`routes/bank_accounts.rs:72-86`) n'a pas `deny_unknown_fields`, et l'extracteur (`extractors.rs:34-58`) rend
    `AppError::Validation` sur un champ requis manquant : un `accountKind` envoyé est **ignoré** (asserté). T1 : la commande rend **135** lignes à
    `07f714a2` (`kesh-gate-logs/e34a1/t1-inventaire-avant.txt`), **151** après (`…-apres.txt`) ; chaque ligne neuve tombe dans un site déjà classé
    (gardes neuves de S1, S3, S4, S5, S7, S12, S13 ; tests `#[cfg(test)]` ; doc-comments d'`errors.rs` et de `post_restore.rs`) : aucun site neuf.
  - **Écart à la consigne « rouges d'abord ».** Code et tests ont été écrits ensemble, avant toute exécution (verrou de la station tenu, puis pause
    des commandes lourdes). Le rouge de chaque test d'AC a été établi **après**, par la mutation de retour qui retire le correctif (M1 à M8, toutes
    tuées par le test attendu). Journal : `kesh-gate-logs/e34a1/rouges.log`.
  - **Assertions du bump triées (AC-2)** : gardées `admin_full_import_e2e.rs:4845`, `:4848`, `:5106` (état d'une base ou d'une sauvegarde 0.17.0) ;
    portées à `"0.17.6"` : `admin_full_import_e2e.rs:4649` (doc), `:4669`, `:4816`, `migrations_fresh_install.rs:248`, `migrations_upgrade_path.rs:549`,
    `:565`, `:581`, `:600`, `:683`, `:1122`, `:1622` (minimum courant) ; ajouté `downgrade_protection_refuses_0_17_5`. `\b0\.17\.6\b` grepé après.
  - **P5** : ligne ajoutée, `ls crates/kesh-db/migrations/*.sql | wc -l` = 91 = `grep -c '^| \`20'` ; partitions recomptées au tableau : 11 `yes` +
    80 `tracked-by-sqlx` + 0 `no`. **P6** : `git grep -n "migrations.len()\|apply_migrations_up_to" crates/` (`kesh-gate-logs/e34a1/p6-grep.txt`) ; seul
    site positionnel, `upgrade_path_preserves_data` : `total` 90 → 91 et soustracteur 56 → 57, frontière 34 inchangée (C-e34a1-9) ; les autres sont
    résolus par version avec assertion de montage. **P7** : entrée `EXEMPT_MIGRATIONS` `Durable`, compteur 23 → 24 avec sa ligne d'historique.
    **P8** : migration jamais modifiée après sa première application ; registre `crates/kesh-db/migrations.sha384` complété (rouge
    `published_migrations_keep_their_checksums` au premier gate ciblé) ; démarrage réel sur base persistante peuplée : la recette de montée et son
    complément (binaire neuf sur une base 0.17.5), et l'E2E (`kesh_e2e_e34a1` migrée par `sqlx migrate run`, puis démarrage du binaire).
  - **Ce qui a été constaté au premier gate et corrigé** : `"1234 "` est admis par la base — MariaDB retire en silence l'espace final d'un
    `CHAR(4)`, la valeur stockée est `1234`, valide (prémisse de test fausse, remplacée) ; le registre des checksums ; cinq fixtures Vitest sans
    `accountKind`/`cardLast4` (`npm run check`, filet de l'AC-10 : **aucun lecteur de production** hors de S12-S17) ; les compteurs codés en dur
    `i18n-keys` (`sitesTotal` 2922 → 2924, `BankImportUpload.svelte` 55 → 57 `i18nMsg(` recomptés) et `i18n-libelle-en-dur` (85 → 86, `cardLast4Label` ;
    `conforme` 71 → 72).
  - **T4** : les écrans affichent le `message` localisé du serveur (`errorMessageOf`) ; aucun libellé propre aux deux codes n'est nécessaire, et
    les écrans évitent les deux refus (lots, import : C-e34a1-8).
  - **Édition** : tout le code par l'outil Edit ; `cargo fmt --all` lancé une fois (cinq assertions reformatées) — le formateur du projet, non un
    script de remplacement. Scripts seulement pour les mutations (`kesh-gate-logs/e34a1/mut-e34a1.py`, motif vérifié unique, restauration
    `git checkout --` puis `touch`).
- 2026-11-11/12 : **revue de code P1 (Sonnet, trois lentilles)** *(annoté le 2026-11-12 : Opus, selon l'orchestrateur qui l'a lancée)* — 0 CRITICAL, 0 HIGH, **3 MEDIUM** (E1, A1, A2), 21 LOW, recomptés depuis les trois
  rapports (`kesh-gate-logs/e34a1-review-p1-{B,E,A}.md` : B1 à B7, E1 à E7, A1 à A10). **Remédiation (Opus 5.5)**, commit `75ab21ae`, puis ce compte rendu :
  - **E1 (MEDIUM)** — `CHECK (BINARY account_kind IN …)`, la migration non publiée **corrigée en place** (C-e34a1-11) : sous `utf8mb4_unicode_ci`,
    `'BANK'` et `'bank '` passaient le `CHECK` et faisaient échouer le décodage. Éprouvé à la main sur MariaDB 10.11.19 : `ascii_bin` est `PAD SPACE`
    (admet `'bank '`), seul `BINARY` ferme à l'octet ; `card_last4` déjà exact sous la collation de la table (chiffres non ASCII refusés). ⚠️ **P8** :
    la migration change après sa première application aux bases de dev — bases `kesh_e34a1`, `kesh_e2e_e34a1`, `kesh_ci_e34a1` **reconstruites**,
    squash régénéré, `migrations.sha384` porté au nouveau checksum (`9339ac2b…`, `sha384sum` du fichier). Aucune base persistante hors des miennes
    ne l'a appliquée (migration jamais poussée). Cas refusés ajoutés (`BANK`, `Bank`, `bank `, `Credit_Card`, `credit_card `) ; témoin sur la base
    **migrée** (`sans_iban_la_montee_garde_chaque_compte_bancaire`).
  - **A1 (MEDIUM)** — § *Passer à la 0.17.6* en `\sloppy`, `\fussy` rétabli avant la section suivante : `KESH_SKIP_PRE_UPGRADE_BACKUP=0.17.6 dans` est
    entier dans le PDF aplati, plus aucun `Overfull` aux lignes 2531-2552 du `.tex` (`admin-manual.log` ; les 178 autres `Overfull` du manuel sont
    antérieurs). Ma déclaration d'avant, « PDF contrôlé aplati », n'avait pas contrôlé la phrase qui porte la valeur. *(Annoté le 2026-11-12, revue
    P2 A1 : le journal du PDF livré porte **173** `Overfull` — `grep -c "^Overfull" fr/admin-manual.log` ; « 178 » venait du journal précédant
    le `\sloppy`.)*
  - **A2 (MEDIUM) + E4** — `full_import_round_trip_keeps_a_card_without_iban` (export d'une carte, base modifiée, import, colonnes relues à
    l'identique) et `full_import_of_a_0_17_5_backup_makes_every_account_a_bank_account` (sauvegarde sans `account_kind` ni `card_last4`). ⚠️ La
    sauvegarde est générique (C2) : aucune mutation de production propre à la carte n'existe pour les voir rouges ; leur preuve est le montage
    (carte présente avant, modifiée entre l'export et l'import).
  - **LOW traités** : E2 = A10 (`compte_sans_iban_le_refus_camt_precede_l_empreinte`, mutation M9) ; E3 = B1 = B2 (carte close et archivée au lot,
    `create_batch_refuses_a_closed_or_archived_card_by_its_state_first`, M10 ; au CAMT, `compte_sans_iban_archive_ou_clos_garde_son_refus_d_abord`) ;
    E7 (contrainte nommée) ; B3, B4 (Vitest, M12, M13 ; commentaire corrigé) ; B6 (« avant le hash, l'empreinte et l'analyse », trois sites :
    `errors.rs`, `bank_imports.rs`, `api-external.md`) ; A5 (aucun conteneur ne restait : il était en `--rm`, en cours de suppression au moment de
    la liste ; le script attend désormais) ; A6 (motif nommé ci-dessous) ; A7 (U+00A0, `CARD_LAST4_SEPARATOR`) ; A8 (ligne `422` rangée) ; A9
    (`create_accepts_an_account_without_iban`, `compte_sans_iban_le_put_ignore_le_type`) ; A4 (crochet de la recette : QR-facture et lot relus après
    la montée) ; A3 (gates complets et E2E au dernier commit de code `75ab21ae`).
  - **LOW renvoyés à la 34-11a2** (fiche `34-11a2-carte-credit-creation.md`, § *Points hérités*) : E5 = B7, B5, E6.
  - Mutation M11 ajoutée (`BINARY` retiré du squash). Toutes les modifications de code par l'outil Edit ; `cargo fmt --all` une fois (cinq tests).
- 2026-11-12 : **revue de code P2 (Sonnet, trois lentilles)** — **0 au-dessus de LOW** (rapports `kesh-gate-logs/e34a1-review-p2-{B,E,A}.md` :
  B1 à B3, E1 à E3, A1 à A6, tous LOW). La boucle se clôt sur cette passe. **Remédiation (Opus 5.5), sans aucune ligne de code de production**
  (un commentaire seul, A6), commit `3875ffca` :
  - **Tests** : B1 = E1 = A5 (quatre chiffres pleine chasse et arabo-indiens refusés ; type `bank` + tabulation, `bank` + NUL, chaîne vide,
    `bänk` refusés — sur le squash et sur la base migrée par le vrai `MIGRATOR`) ; B2 (le témoin sur la base migrée rougit si `BINARY` quitte le
    **fichier de migration** : mutation M14, jouée puis restaurée, bases reconstruites) ; E2 (`full_import_refuses_a_forged_account_kind_and_keeps_the_base`) ;
    B3 côté tests (U+00A0 par `String.fromCharCode(0xa0)`).
  - **Commentaire** : A6 (`bank_imports.rs:1253-1254`, « avant le hash, l'empreinte, le verrou et l'analyse »).
  - **Documents** : A1 (173, annotation datée ci-dessus) ; A2 — la recette `r2` ne journalise pas le binaire qu'elle a joué : `target/debug/kesh-api`
    construit à `75ab21ae` (`build-r1.log`), et la recette **se rejouera à l'intégration** sur la branche de release ; A3 — captures reprises (thème
    clair, base de démonstration jetable), l'espace insécable changeant l'affichage ; A4 — sans objet : la 34-11a2 part de `27ca0bd9`, qui porte déjà
    le nouveau checksum (`9339ac2b…`), et aucune autre base persistante n'a appliqué la migration.
  - **Renvoyés à la 34-11a2** (transmis par l'orchestrateur) : E3 (apostrophe typographique de `bank-import-labels-drop-zone-csv-only`), B3 côté
    production (`format.ts:26`, U+00A0 en littéral invisible).
  - ⚠️ **Écart de l'outil Edit**, à signaler : un `' '` écrit dans `new_string` est **converti** en U+00A0 littéral dans le fichier (octets
    `C2 A0`) — c'est l'origine de B3 ; `'\\u00a0'` est écrit avec deux barres obliques. D'où `String.fromCharCode(0xa0)`, sans séquence d'échappement.
- **Tendance de la boucle de revue de code** : P1 (Opus) 0 C / 0 H / 3 M / 21 L → P2 (Sonnet) 0 C / 0 H / 0 M / 12 L. Développement et
  remédiations : Opus 5.5. La P2 n'a rien trouvé qui vienne de la remédiation P1 au-dessus de LOW.
- **Gates de clôture, à `3875ffca`** (dernier commit qui touche du code — des tests et un commentaire) : `fmt` et `clippy` du workspace verts
  (`clippy-r2.log`) ; gate ciblé 243 passés (`cible-r2.log`) ; `test-fast.sh --ci` **5163 passés, 12 ignorés** (`gate-test-fast-r2.log`) ;
  `gate-ci.sh` **5170 passés, 0 échec, 16 ignorés** (`gate-ci-r2.log`) ; Vitest **2890** (`front-unit-r2.log`) ; mutation **M14** tuée par
  `sans_iban_la_montee_garde_chaque_compte_bancaire` (`mut/M14.log`), fichier restauré (`sha384sum` `9339ac2b…`), bases reconstruites. L'E2E
  complet n'est pas rejoué : aucune ligne de production n'a changé depuis `75ab21ae`, où il a tourné. Captures reprises (`captures-r2.log`).
  **Statut `done`.**

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), en autonomie.

### Completion Notes List

**Gates de la remédiation P1, au dernier commit de code `75ab21ae`** (journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e34a1/`) :

- `cargo fmt --all -- --check` vert (`test-fast.sh`) ; gate ciblé avant commit : **810 passés** (`cible-r1.log`), Vitest ciblé 49 (`vitest-cible-r1.log`).
- `scripts/test-fast.sh --ci`, bases remises à neuf : **5162 passés, 12 ignorés** (`gate-test-fast-r1.log`) ; `cargo build --workspace --all-targets`
  vert (`build-r1.log`) ; `scripts/gate-ci.sh kesh_ci_e34a1` : **5169 passés, 0 échec, 16 ignorés** (`gate-ci-r1.log`).
- Frontend (voie `.gate-front.lock`) : `npm run check` **0 erreur** (29 avertissements préexistants), `lint-i18n-ownership` vert, Vitest
  **2890 tests, 222 fichiers**, `npm run build` vert (`front-*-r1.log`).
- **E2E complet** à `75ab21ae` (`e2e-r2/playwright.log`) : **385 passés, 7 échecs, 18 ignorés** — les sept KF-029 (#97) de `docs/testing.md`, aucun autre.
- **Mutations** : M1 à M8 rejouées, M9 à M13 neuves — **14 tuées sur 14**, chacune par le test attendu (`mut/campagne-r1-rust.log`,
  `mut/campagne-r1-front.log`, `mut/M*.log`) ; worktree propre ensuite (hors le registre et le prompt P2, non commités à ce moment).
- **Montée rejouée** (la migration a changé) : premier passage `recette-montee-r1.log` **en échec au crochet** — l'IBAN d'exemple de la recette est un
  QR-IBAN, que la QR-facture refuse sans référence QRR (comportement antérieur) ; crochet corrigé (IBAN ordinaire synthétique sur la ligne 0.17.5),
  second passage `recette-montee-r2.log` : **cinq phases vertes** ; crochet (`recette-montee-journal-r2/apres.log`) : colonnes d'avant identiques,
  comptes `bank`, carte forgée refusée au lot, **QR-facture produite** (71 301 octets) et **lot généré avec son `pain.001`** (débiteur relu) après
  la montée. Complément (`recette-complement-r1.log`, `recette-c/resume.txt`) : `pre-upgrade/` écrit avant la migration, aucun écart des colonnes
  0.17.5, 0.17.5 neuve qui refuse le `.keshbackup` 0.17.6 (409) et importe le fichier `pre-upgrade/` (200), empreintes identiques ; « conteneurs
  restants : » vide.
- **Tests neufs**, recomptés de `07f714a2` à `75ab21ae`, motif `^\s*#\[(sqlx::test|test|tokio::test)\b` sur les fichiers `crates/*.rs` touchés :
  **452 → 476, +24** (+17 au développement, +7 à la remédiation) ; Vitest (`^\s*it\(`) **+9** (`format.test.ts` 11 → 17, `BankImportUpload.test.ts`
  30 → 32, `payment-batches-page.test.ts` 8 → 9). *(Annoté le 2026-11-12, revue P1 A6 : le décompte « 439 → 456 » ci-dessous mêlait `#[tokio::test]` et
  une ligne de commentaire ; le delta +17 restait juste.)*

**Gates du développement (avant la revue P1)**, tels qu'ils ont tourné :

- `cargo fmt --all -- --check` vert ; `cargo clippy --workspace --all-targets -- -D warnings` vert (`clippy-1.log` au code, `clippy-2.log` au
  dernier commit `93e2642f`).
- `scripts/test-fast.sh --ci`, bases `kesh_e34a1` remises à neuf : **5154 passés, 12 ignorés** (`gate-test-fast-1.log`, arbre de `0d7aa7cc`).
- `cargo build --workspace --all-targets` vert (`build-1.log`, `0d7aa7cc`).
- `scripts/gate-ci.sh kesh_ci_e34a1` : **5161 passés, 0 échec, 16 ignorés** (`gate-ci-1.log`, `gate-ci-journal-1.log`, `0d7aa7cc`).
- Frontend : `npm run check` **0 erreur** (29 avertissements préexistants, aucun dans les fichiers de la story ; `front-check-2.log`) ;
  `npm run lint-i18n-ownership` vert ; `npm run test:unit` **2889 tests, 222 fichiers** (`front-unit-2.log`) ; `npm run build` vert.
- **E2E complet** à `661a2a01`, base `kesh_e2e_e34a1` remise à neuf, port 3361 : **385 passés, 7 échecs, 18 ignorés** (`e2e-r1/playwright.log`) ;
  les sept échecs sont exactement les sept KF-029 (#97) de `docs/testing.md` § *Les échecs attendus* (`mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) ; aucun backend à l'écoute avant ni après.
- ⚠️ **Dernier commit de code, `93e2642f`** : il n'ajoute qu'un `#[test]` à `admin_full_import_e2e.rs` (et le manuel) ; il est couvert par le binaire
  ciblé `admin_full_import_e2e` (**71 passés**, le test neuf compris, `cible-2-admin-import.log`) et par `clippy` du workspace, **non** par un
  nouveau gate complet ni un nouvel E2E.
- **Mutations** (`mut/campagne.log`, `mut/M*.log`) : M1, M2, M3, M3b, M4, M5, M6, M7, M8 — **9 tuées sur 9**, chacune par le test attendu ; worktree
  propre après la campagne. M6 aux sites de création et de l'assistant : **équivalente** dans la 34-11a1 (ces routes exigent l'IBAN, C-e34a1-5).
- **Montée (T5)** : `scripts/recette-montee.sh --image gcorbaz/kesh:0.17.5` (image tirée de Docker Hub, `sha256:ca197b5b…`) ,
  `--migrations-attendues 1 --diff-attendu migrations,bank_accounts,min_required --apres apres-sans-iban.sh --import` : **cinq phases vertes**
  (`recette-montee-1.log`, journal copié dans `recette-montee-journal/`) — fichier `pre-upgrade/…-0.17.5-to-0.17.6.keshbackup` écrit avant la
  migration ; crochet : colonnes d'avant de `bank_accounts` identiques, chaque compte `bank`, l'API rend `accountKind`/`cardLast4`, une carte forgée
  reçoit `409 PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN` ; l'image 0.17.5 refuse la base migrée (« La base a été migrée par Kesh v0.17.6… ») ; retour par le
  dump, empreinte identique ; `.keshbackup` 0.17.5 importé dans le binaire neuf. **Complément** (`recette-complement-e34a1.sh`, `recette-c/resume.txt`) :
  0.17.5 neuve qui refuse le `.keshbackup` 0.17.6 (`409 IMPORT_VERSION_INCOMPATIBLE`) et importe le fichier `pre-upgrade/` (200), empreintes identiques
  à l'avant, toutes colonnes. Données synthétiques seulement ; aucun conteneur restant *(annoté le 2026-11-12, revue P1 A5 : le journal du complément
  en listait un, `kesh-e34a1c-db2-…`, en cours de suppression `--rm` ; constaté absent ensuite par `docker ps -a`)*.
- **Captures** (thème clair, instance jetable, carte forgée) : `kesh-gate-logs/e34a1/captures/34-11a1-01-tableau-comptes.png`, `-02-accueil.png`,
  `-03-parametres.png`, `-04-import-carte.png`, `-05-lot-sans-la-carte.png` (le compte bancaire de cette base n'étant pas lié au grand livre, le
  sélecteur du lot n'y propose aucun compte : la capture prouve l'absence de la carte, non la présence de l'autre ; le Vitest le prouve).
- **Tests neufs**, recomptés de `07f714a2` à l'arbre final : Rust **+17** (correspondances `#[test]`/`#[sqlx::test]` des fichiers touchés,
  439 → 456 ; la ligne 799 d'`admin_full_import_e2e.rs`, un commentaire, compte aux deux bornes) ; Vitest **+8**
  (`format.test.ts` 11 → 17, `BankImportUpload.test.ts` 30 → 31, `payment-batches-page.test.ts` 8 → 9).
- **PDF** du manuel d'administration régénéré (`make admin`), contrôlé aplati (`flat-admin-manual.txt`) : « Passer à la 0.17.6 », le nom de la migration
  entier (il débordait la marge avant un `\allowbreak`), le message FATAL relevé.

**Limites voisines à tracer (issues demandées, une par problème)** :
1. Import CAMT d'un relevé de compte sans IBAN (`Acct/Id/Othr`) — déjà demandée par la fiche.
2. Tuile *Comptes bancaires* de l'accueil : à 1280 px, le lien « Solde non disponible — lier au plan comptable » chevauche le nom du compte
   (capture `34-11a1-02-accueil.png`) ; préexistant, vrai d'un compte bancaire.

### File List

- `crates/kesh-db/migrations/20261110000001_bank_accounts_without_iban.sql` (neuf), `crates/kesh-db/migrations.sha384`,
  `crates/kesh-db/test-schema/0001_schema_squash.sql` (régénéré), `Cargo.lock`, `crates/*/Cargo.toml` (dix, 0.17.6)
- `crates/kesh-db/src/entities/bank_account.rs`, `entities/mod.rs`, `errors.rs`, `post_restore.rs`, `repositories/bank_accounts.rs`,
  `repositories/payment_batches.rs`
- `crates/kesh-api/src/errors.rs`, `exports/csv_tables.rs`, `routes/bank_accounts.rs`, `routes/bank_import_fingerprint.rs`, `routes/bank_imports.rs`,
  `routes/companies.rs`, `routes/invoice_pdf_service.rs`, `routes/onboarding.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- Tests : `crates/kesh-db/tests/{bank_accounts_repository,bank_imports_repository,migrations_fresh_install,migrations_upgrade_path,payment_batches_repository}.rs`,
  `crates/kesh-api/tests/{admin_full_import_e2e,bank_accounts_e2e,bank_imports_e2e}.rs`
- `frontend/src/lib/features/bank-accounts/{bank-accounts.api.ts,format.ts,format.test.ts,BankAccountJournalLinkForm.test.ts}`,
  `bank-import/{BankImportUpload.svelte,BankImportUpload.test.ts,bank-import.types.ts}`, `homepage/{BankAccountsCard.svelte,HomepageCards.test.ts,homepage.test.ts}`,
  `settings/settings.types.ts`, `shared/{i18n-keys.test.ts,i18n-libelle-en-dur.test.ts}`,
  `frontend/src/routes/(app)/{bank-accounts/+page.svelte,bank-accounts/bank-accounts-page.test.ts,payment-batches/+page.svelte,payment-batches/payment-batches-page.test.ts,settings/+page.svelte}`
- `docs/api-external.md`, `docs/migrations-idempotence-audit.md`, `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`, `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e34a1.md` (neuf), cette fiche, `sprint-status.yaml`,
  `34-11a1-compte-sans-iban-review-prompt-p1.md` (neuf)
- Remédiation P1 : en plus, `crates/kesh-api/tests/bank_statements_manual_e2e.rs`, `34-11a2-carte-credit-creation.md` (§ *Points hérités*),
  `34-11a1-compte-sans-iban-review-prompt-p2.md` (neuf)
