# Story 34-11a1 : un compte sans IBAN — le schéma, et tous les sites qui lisent l'IBAN le tolèrent (#972)

Status: ready-for-dev

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
| S11 | `repositories/bank_accounts.rs:54`, `:65-70`, `:254`, `:352-356`, `:382-387`, `:658-662` | lectures et écritures neutres | **B** | AC-3 |
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

- [ ] **T0 — étape 0** : constater ce que fait aujourd'hui un `accountKind` envoyé à `POST /bank-accounts` (AC-11), et relancer la commande de T1.
- [ ] **T1 — rouges d'abord**, inventaire rejoué (sortie triée au Dev Agent Record ; tout site neuf classé avant le code) :
  ```
  git grep -n -E "(\.|\b)iban\b" -- 'crates/*/src/**' 'frontend/src/**' ':!*.test.ts' ':!crates/kesh-core/src/types/**' ':!crates/kesh-qrbill/src/**' ':!crates/kesh-import/src/**' ':!crates/kesh-reconciliation/src/**' | /usr/bin/grep -v -E "creditor_iban|counterparty_iban|qr_iban|qrIban|creditorIban|is_qr_iban|//|^\S+:[0-9]+:\s*\*"
  ```
  Rouges : AC-4, AC-5 (le 500 d'aujourd'hui), AC-6, AC-7 (valeur figée), AC-8, AC-12, Vitest AC-10.
- [ ] **T2 — migration**, schéma de test, P5, P6, P7, bump Cargo, tri des `"0.17.0"` et compteur d'exemptions (AC-2). ⚠️ **gate complet même en
  cours de boucle** (`kesh-db`) ; démarrage réel sur une base persistante (P8).
- [ ] **T3 — tolérance** : entité, dépôt, S1/S2, S4, S5, S6, S7, S9, S10 ; frontend S12 à S17.
- [ ] **T4 — codes d'erreur** : `PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN`, `BANK_IMPORT_CAMT_NEEDS_IBAN` : variantes `AppError`, `errors.rs`, messages
  aux quatre `messages.ftl`, libellés frontend, tableau d'erreurs de `api-external.md`.
- [ ] **T5 — montée** : `scripts/recette-montee.sh` depuis l'image 0.17.5 publiée, base peuplée (comptes, imports, lots, factures QR). La recette
  constate (précédent de la 0.17.5) : le fichier `backup/pre-upgrade/` écrit par Kesh **avant** la migration (#847) ; l'image 0.17.5 qui **refuse**
  de démarrer sur la base migrée ; le fichier `pre-upgrade/` importé dans une 0.17.5 neuve, empreintes identiques à l'avant ; un `.keshbackup` 0.17.5
  importé dans une 0.17.6. Et un **test automatisé** dans `migrations_upgrade_path.rs` : lignes `bank_accounts` posées avant la migration, puis
  `account_kind = 'bank'`, IBAN intact, `CHECK`s satisfaits — montage **résolu par version** (`.position(|m| m.version == …)`, P6), non par position.
- [ ] **T6 — documentation** (AC-13), CHANGELOG (migration, avertissement de sauvegarde en tête), captures `kesh-gate-logs/e341/captures/34-11a1-*.png`
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

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
