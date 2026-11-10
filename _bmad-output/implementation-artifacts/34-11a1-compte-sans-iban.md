# Story 34-11a : une carte de crédit se déclare comme un compte, sans IBAN, lié à un compte de dette (#972)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Une migration, breaking** (`kesh_version_min_required` → 0.17.6,
C-e341-14) : P1 à P8 du `CLAUDE.md` s'appliquent. Registre : `e34-choix-autonomes-e341.md`, C-e341-13 à C-e341-21. Vient après **34-3**
(ordre de l'epic), **34-1** et **34-2** (même formulaire). Commits en `refs #972` ; à la PR : `refs #972` (la 34-11b ferme l'issue).*

## Story

En tant que titulaire d'une carte de crédit, je veux la déclarer dans Kesh comme un compte, sans IBAN, lié à un compte de dette et identifié
par ses quatre derniers chiffres, pour y faire entrer ses relevés et la réconcilier comme un compte bancaire (34-11b), sans que Kesh n'en
garde jamais le numéro complet.

## Décisions de Guy (#972, fichier d'epic, 2026-11-10)

Une carte est tenue **comme un compte**, non par « Éclater ». Le compte **n'a pas d'IBAN** et il est **lié à un compte de dette**. **Seuls
les quatre derniers chiffres** de la carte sont gardés : le numéro complet n'est jamais enregistré. Le type « sans IBAN » est pensé assez
général pour servir plus tard (PayPal, caisse) ; aucun QR-IBAN, aucun paiement `pain.001` depuis ce compte.
⛔ Rappels : **le numéro d'un compte du plan ne déduit jamais un rôle** (Story 14-3a) — la carte se déclare par son **type**, jamais parce que
son compte lié serait un 2xxx ; et **un appariement automatique propose, il ne crée jamais** (rien ici n'apparie).

## Ce que le code dit (prémisses établies)

- **Le verrou** : `bank_accounts.iban` est `VARCHAR(34) NOT NULL` avec `chk_bank_accounts_iban_nonempty`
  (`migrations/20260410000001_bank_accounts.sql:8`, `:16` ; `test-schema/0001_schema_squash.sql:156`, `:171`). **Aucun index unique** sur
  l'IBAN (deux comptes peuvent déjà le partager). L'entité porte `iban: String` (`entities/bank_account.rs:39`, `:81`) ; la route l'exige
  (`routes/bank_accounts.rs:79`, `:99`) et le valide (`Iban::new`, `:322-325`).
- **Le lien à un passif est déjà permis** : `validate_journal_account_id` accepte `Asset | Liability` (`routes/bank_accounts.rs:382-390`).
- **Migration probable, confirmée** : 90 migrations (`ls crates/kesh-db/migrations/*.sql | wc -l`), aucune ne touche `iban` depuis la création.
- **Le précédent de la colonne d'énumération** : `20261011000001_bank_profiles_debit_sign.sql` (colonne `VARCHAR` + `CHECK`, bump
  `min_required`, entrée `EXEMPT_MIGRATIONS`, ligne d'audit P5) : patron à suivre.

### L'inventaire clos des sites qui lisent l'IBAN d'un compte bancaire

Établi en partant du symptôme (`git grep -n -E "(\.|\b)iban\b" -- 'crates/*/src/**' 'frontend/src/**' ':!*.test.ts' …`, 134 lignes triées,
commande exacte en T1), non d'une liste ; les lectures de `creditor_iban`, `counterparty_iban` et de la règle `iban_exact` portent sur d'autres
colonnes et en sont exclues. Classes : **A** refuse un compte sans IBAN ; **B** l'accepte et l'affiche autrement ; **C** inatteignable (justifié).
Chaque site est **résolu** par un AC, ou écrit angle mort.

| # | Site | Ce qu'il fait | Classe | Résolu par |
|---|---|---|---|---|
| S1 | `repositories/payment_batches.rs:862-869` `generate_pain001_xml` | `SELECT iban` dans un tuple `(String,)` → `<DbtrAcct><IBAN>` (`kesh-payment/src/pain001/mod.rs:196-201`) | **A** — ⚠️ le compilateur ne verra pas le passage à `NULL` : échec de décodage, **500**, le lot déjà créé | AC-5 |
| S2 | `repositories/payment_batches.rs:157-175` `create_batch` | contrôle archivé, clos, lié — **ne lit pas l'IBAN** | **A** | AC-5 |
| S3 | `routes/invoice_pdf_service.rs:223`, `:332-342` | QR-facture : le **principal** (`find_primary`, `repositories/bank_accounts.rs:104`), QR-IBAN préféré sinon IBAN | **A** | AC-4 (une carte n'est jamais principale) |
| S4 | `routes/bank_imports.rs:940-972` `select_statement_by_iban` ; appelée `:1060` (aperçu), `:1313` (création), `:1112`, `:1376` (alias 0.17.3, `:2087-2120`) | CAMT : relevé choisi par l'IBAN du compte | **A** | AC-6 |
| S5 | `routes/bank_import_fingerprint.rs:132` | IBAN normalisé dans l'empreinte d'aperçu (CAMT **et** CSV) | **B** | AC-7 |
| S6 | `routes/bank_imports.rs:2341`, type `:247` `PreviewStatement.account_iban: String` | aperçu CSV | **B** | AC-7 |
| S7 | `routes/bank_accounts.rs:674`, `:850`, `:859` ; `routes/onboarding.rs:669`, `:681` | audit `"iban_present": true` **codé en dur** | **B** — la trace mentirait | AC-8 |
| S8 | `routes/onboarding.rs:579`, `:603-604`, `:628` | l'assistant crée le **principal**, IBAN obligatoire | **A** (inchangé : reste un compte bancaire) | AC-4, C-e341-19 |
| S9 | `routes/companies.rs:418`, `:431` `BankAccountJson` | `GET /companies/current` | **B** | AC-3 |
| S10 | `exports/csv_tables.rs:647`, `:664` | export `bank_accounts.csv`, `txt(iban)` | **B** | AC-9 |
| S11 | `repositories/bank_accounts.rs:54` (`columns!`), `:65-70`, `:254`, `:352-356`, `:382-387`, `:658-662` | lectures et écritures neutres | **B** | AC-3 (type) |
| S12 | `frontend/src/lib/features/bank-accounts/format.ts:16-18` `shortIban` | `iban.length` — **`TypeError` sur `null`** | **B** | AC-10 |
| S13 | `format.ts:62`, `:87-92` `bankAccountOptionLabel` et ses sept appelants (`BankAccountJournalLinkForm.svelte:135`, `BankImportUpload.svelte:372`, `ManualStatementForm.svelte:346`, `SettleInvoiceDialog.svelte:249`, `payment-batches/+page.svelte:162`, `reconciliation/+page.svelte:68`, `supplier-invoices/[id]/+page.svelte:888`) | « Nom — IBAN » | **B** (+ **A** à l'écran pour les lots et le CAMT) | AC-10, AC-5, AC-6 |
| S14 | `homepage/BankAccountsCard.svelte:39` | `shortIban` sur l'accueil | **B** | AC-10 |
| S15 | `routes/(app)/bank-accounts/+page.svelte:209`, `:303`, `:328`, `:542` (`required`), `:653`, `:754` | formulaire et tableau | **B** | AC-10 |
| S16 | `routes/(app)/settings/+page.svelte:607` ; types `bank-accounts.api.ts:28`, `:83`, `:105`, `:210`, `:239`, `settings.types.ts:72` | affichage, types | **B** | AC-10 |
| S17 | `BankImportUpload.svelte:513`, `bank-import.types.ts:5`, `:15` | IBAN du relevé choisi | **B** | AC-7 |
| S18 | `docs/api-external.md:233`, `:245`, `:788`, `:790`, `:798` | contrat d'API | **B** | AC-11 |
| C1 | Serveur MCP (`crates/kesh-api/src/mcp/**`) | aucun outil ne rend un compte bancaire, seulement `compte_bancaire_id` (`outils/factures_fournisseur.rs:224`) | **C** | — |
| C2 | Sauvegarde / restauration (`kesh-db/src/backup.rs:283-341`, `:593-611`) | générique par `INFORMATION_SCHEMA`, `NULL` ↔ `Value::Null` | **C** pour la 0.17.6 ; le retour vers une 0.17.5 est fermé par le bump (C-e341-14) | AC-12 |
| C3 | Réconciliation (`kesh-reconciliation`, `routes/reconciliation*.rs:2541`, `:3506`, `:3896`) | ne lit que `journal_account_id` | **C** | — |
| C4 | Relevé saisi à la main (`routes/bank_statements_manual.rs`, `kesh-core/bank_imports/manual.rs`) | ne lit que `counterparty_iban` | **C** | — |
| C5 | Contrôles de clôture (`closing_checks.rs:444-470`), règlements (`invoice_settlements_write.rs:170`, `supplier_invoices.rs:2010`, `:2268`) | `bank_name`, `description`, `journal_account_id`, `closed_on`, `archived` | **C** | — |
| C6 | `kesh-report`, `kesh-seed` (`seed_demo` ne crée aucun compte bancaire), `post_restore/20260828000001…:26` | aucune lecture de `bank_accounts.iban` | **C** | — |

**Angles morts écrits** : (i) le flux aval de `kesh-qrbill` n'a pas été lu (S3 l'éteint en amont : une carte n'est jamais principale) ;
(ii) les messages `.ftl` qui disent « IBAN » n'ont pas été passés en revue (libellés à revoir par la revue, non des lectures de données) ;
(iii) les tests (≈ 316 lignes Rust, 66 Vitest, 64 Playwright) changeront de type sans changer de sens.

## Choix (détail au registre)

- **C-e341-13 — un type explicite** : colonne `account_kind` (`'bank'` | `'credit_card'`, `DEFAULT 'bank'`), non « IBAN nul » : le type se
  déclare, il ne se déduit pas (même principe que 14-3a). L'énumération s'élargira (PayPal, caisse) par une migration future.
- **C-e341-14 — breaking, bump `min_required` à 0.17.6** : `MODIFY COLUMN iban … NULL` est une opération de P3 ; un binaire 0.17.5 qui lirait
  un `iban` nul échouerait au décodage (`String`), et une 0.17.5 qui importerait un `.keshbackup` 0.17.6 ignorerait `account_kind` (une carte
  redeviendrait compte bancaire). Bump + versions Cargo du workspace à 0.17.6 dans la même PR (P2-bis).
- **C-e341-15 — les quatre derniers chiffres** : `card_last4 CHAR(4) NULL`, exactement quatre chiffres ASCII, refus sinon (jamais de
  troncature : un numéro complet ne transite pas jusqu'au stockage, il est refusé). Et **garde de Luhn** : une suite de 13 à 19 chiffres
  (espaces et tirets admis entre eux) qui satisfait la somme de Luhn, dans le nom, la description ou les quatre chiffres d'un compte, est refusée
  `400 CARD_NUMBER_NOT_ALLOWED` — pour les deux types (un numéro de carte n'a rien à faire dans un compte bancaire non plus).
- **C-e341-16 — la carte est liée à un passif, toujours** : à la création et au `PUT`, `journalAccountId` est **obligatoire** pour une carte et
  doit être `Liability` (`400 CREDIT_CARD_LEDGER_MUST_BE_LIABILITY`) ; le `PATCH` qui délie une carte est refusé. Un compte bancaire garde ses
  règles.
- **C-e341-17 — invariants au schéma** : `CHECK`s : `bank` ⇒ `iban IS NOT NULL` ; `credit_card` ⇒ `iban IS NULL AND qr_iban IS NULL AND
  is_primary = FALSE` ; `card_last4 IS NULL OR (account_kind = 'credit_card' AND card_last4 REGEXP '^[0-9]{4}$')`. Le type ne change pas après la
  création (le `PUT` refuse un changement, `400 BANK_ACCOUNT_KIND_IMMUTABLE`).
- **C-e341-18 — refus nommés aux sites A** : `pain.001` (`409 PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN`, à la création du lot, et le même refus défensif
  à la génération, sans 500) ; CAMT (`422 BANK_IMPORT_CAMT_NEEDS_IBAN`, avant toute lecture du fichier) ; QR-facture : inatteignable par le
  `CHECK` et le refus `400` d'une carte principale.
- **C-e341-19 — l'assistant reste bancaire** : il crée le compte principal qui porte la QR-facture ; IBAN obligatoire inchangé.
- **C-e341-20 — dérogation à la règle de splitting** : la story touche plus de cinq modules. La découper livrerait un état intermédiaire où un
  `iban` nul existe sans les gardes A (S1 : un 500 après la création du lot) ou sans les gardes B (S12 : l'accueil plante). Elle reste une, avec
  un **ordre de commits imposé** (T2 → T6) : d'abord tout le dépôt tolère `NULL` sans qu'aucune route n'en crée, ensuite seulement la création.
- **C-e341-21 — affichage** : une carte s'affiche « Visa — Personnel •••• 1234 » (fonction unique de `format.ts`) ; l'IBAN y est remplacé, jamais
  par « null » ni « — » dans un sélecteur.

## Acceptance Criteria

1. **AC-1 — la migration** : `ALTER TABLE bank_accounts` : `iban` nullable, `account_kind`, `card_last4`, les `CHECK`s de C-e341-17 ; dernière
   instruction `UPDATE _kesh_version SET kesh_version_min_required = '0.17.6' WHERE id = 1;` ; crates du workspace en 0.17.6. Schéma de test
   régénéré (`test_schema_guard.rs` vert). Une base 0.17.5 **peuplée** (comptes bancaires, imports, lots, factures avec QR) monte sans écart :
   empreintes des tables identiques hors colonnes neuves, chaque compte `bank`.
2. **AC-2 — P5 à P7** : ligne à `docs/migrations-idempotence-audit.md`, total et partitions **recomptés** (`ls … | wc -l` = 91 =
   `grep -c '^| \`20'`) ; `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` inspecté, chaque site résolu (P6) ; entrée
   `EXEMPT_MIGRATIONS` justifiée (seul statement d'écriture : le bump d'une table système, comme `20261011000001`) (P7).
3. **AC-3 — l'entité et le dépôt** : `iban: Option<String>`, `account_kind: BankAccountKind`, `card_last4: Option<String>` ; `columns!()` les
   lit ; aucune autre colonne ne change.
4. **AC-4 — créer une carte** : `POST /bank-accounts` avec `accountKind: "credit_card"`, `bankName`, `description?`, `cardLast4?`,
   `journalAccountId` (passif) → `201`, `iban` et `qrIban` `null`. Refus, chacun sans écriture : un IBAN ou un QR-IBAN fourni pour une carte ;
   `isPrimary: true` ; sans compte lié ; compte lié `Asset` ; `cardLast4` qui n'est pas exactement quatre chiffres (`"12345"`, `"12a4"`, 16 chiffres) ;
   un numéro de carte valide au sens de Luhn dans le nom ou la description. `accountKind` absent vaut `"bank"` : un corps 0.17.5 passe inchangé.
   Le solde d'ouverture de la 34-2 s'y applique (sens *Dette* présélectionné, compte `Liability`).
5. **AC-5 — pain.001** : `POST /payment-batches` sur une carte → `409 PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN`, rien d'écrit ; `generate_pain001_xml`
   lit `Option<String>` et rend le même refus au lieu d'un échec de décodage (test : lot forgé en base sur une carte, `GET` du fichier → `409`, pas
   `500`). L'écran des lots ne propose pas les cartes.
6. **AC-6 — CAMT** : aperçu et import CAMT sur une carte → `422 BANK_IMPORT_CAMT_NEEDS_IBAN` avant la lecture du fichier ; l'écran d'import
   n'offre que le CSV et la saisie manuelle pour une carte. CSV et saisie manuelle **acceptés** sur une carte (test d'un import CSV réussi).
7. **AC-7 — empreinte et aperçu CSV** : l'empreinte encode un IBAN absent de façon distincte de toute chaîne (`null`), stable d'un appel à
   l'autre ; un compte bancaire garde **la même** empreinte qu'en 0.17.5 (test sur une valeur figée). `PreviewStatement.accountIban` devient
   `string | null`.
8. **AC-8 — audit** : `iban_present` vaut `iban.is_some()` aux cinq sites S7 ; `bank_account.created` porte `account_kind` et
   `card_last4_present` — **jamais** les chiffres eux-mêmes ni l'IBAN.
9. **AC-9 — export** : `bank_accounts.csv` : `iban` vide pour une carte, colonnes `account_kind` et `card_last4` ajoutées **en fin de ligne**
   (les consommateurs par position ne cassent pas) ; `docs/` qui décrit l'export mis à jour.
10. **AC-10 — l'écran** : *Comptes bancaires* : un choix *Compte bancaire / Carte de crédit* à la création (non modifiable ensuite) ; pour une
    carte, IBAN, QR-IBAN et *principal* masqués, *Quatre derniers chiffres (facultatif)* et une aide « Kesh ne garde jamais le numéro complet » ;
    la liste des comptes liés ne propose que des passifs ; tableau, accueil, *Paramètres* et les sept sélecteurs affichent « •••• 1234 » sans
    planter (Vitest : `shortIban(null)` n'est plus atteint, `bankAccountOptionLabel` d'une carte).
11. **AC-11 — documentation** : `docs/api-external.md` (S18 ; corps, refus, empreinte), manuel utilisateur *Comptes bancaires* (le type carte,
    les quatre chiffres, ce que la carte ne fait pas : QR-facture, `pain.001`, CAMT), manuel d'administration et **notes de version** : la
    migration, **la sauvegarde d'avant la mise à jour en tête**, le retour à la 0.17.5 par cette sauvegarde seulement. PDF régénérés, aplatis.
12. **AC-12 — sauvegarde** : un `.keshbackup` 0.17.6 qui contient une carte se réimporte dans une 0.17.6 neuve à l'identique (carte, `NULL`,
    quatre chiffres) ; il est **refusé** par une 0.17.5 en le nommant (garde `min_required`, `routes/admin.rs:235-241`) ; un `.keshbackup` 0.17.5
    s'importe dans une 0.17.6 (comptes `bank`).

## Tâches

- [ ] **T1 — rouges d'abord** et **inventaire rejoué** : relancer la commande large au commit de départ, recompter (134 lignes au `a75835db`), et
  joindre la sortie triée au Dev Agent Record ; tout site neuf est classé avant d'écrire du code.
  ```
  git grep -n -E "(\.|\b)iban\b" -- 'crates/*/src/**' 'frontend/src/**' ':!*.test.ts' ':!crates/kesh-core/src/types/**' ':!crates/kesh-qrbill/src/**' ':!crates/kesh-import/src/**' ':!crates/kesh-reconciliation/src/**' | /usr/bin/grep -v -E "creditor_iban|counterparty_iban|qr_iban|qrIban|creditorIban|is_qr_iban|//|^\S+:[0-9]+:\s*\*"
  ```
  Tests rouges : AC-4 (refus et création), AC-5 (le 500 d'aujourd'hui, forgé en base), AC-6, AC-7 (valeur figée), AC-8, AC-12, Vitest AC-10.
- [ ] **T2 — migration** (`<horodatage>_bank_accounts_credit_card.sql`, numérotée par horodatage, l'intégration contrôle l'ordre avec la 34-5) ;
  schéma de test ; P5, P6, P7 ; bump Cargo. ⚠️ **gate complet même en cours de boucle** (`kesh-db`).
- [ ] **T3 — tolérance** (aucune route ne crée encore de carte) : entité, dépôt, S1 et S2 (refus), S4 (refus), S5, S6, S7, S9, S10, frontend S12 à S17.
- [ ] **T4 — création** : routes (corps, contrôles C-e341-15 à 17, refus nommés), `kesh-core` : la garde de Luhn (fonction pure, tests de table :
  numéros de test publics des réseaux, 15 et 16 chiffres, avec espaces et tirets ; un IBAN, un numéro de téléphone, un montant ne déclenchent pas).
- [ ] **T5 — écran** (AC-10), i18n quatre locales, `lint-i18n-ownership`.
- [ ] **T6 — montée** : `scripts/recette-montee.sh` depuis l'image 0.17.5 publiée, base peuplée ; puis E2E neuf `bank-accounts-credit-card.spec.ts`
  (créer une carte liée à un passif créé par l'API, `•••• 1234` dans la liste et dans le sélecteur de l'import, CAMT absent pour elle, lot refusé).
- [ ] **T7 — documentation** (AC-11), captures `kesh-gate-logs/e341/captures/34-11a-*.png`, CHANGELOG *Ajouté* + avertissement de migration.

### Mutations nommées

- **M1** : le refus de `create_batch` retiré → rouge AC-5 (création) ; **M2** : `generate_pain001_xml` remis en `(String,)` → rouge AC-5 (`500`).
- **M3** : le refus CAMT retiré → rouge AC-6 (`NO_MATCHING_STATEMENT` au lieu du code dédié).
- **M4** : l'empreinte encode `None` comme `""` → rouge AC-7 (distinction) ; **M5** : l'empreinte d'un compte bancaire change de forme → rouge
  AC-7 (valeur figée).
- **M6** : `iban_present: true` rétabli → rouge AC-8 ; **M7** : la garde de Luhn retirée de la description → rouge AC-4.
- **M8** : `CHECK` « carte ⇒ non principale » retiré **et** refus de route retiré → rouge AC-4 par la base ; chacun seul → l'autre tient (test).
- **M9** : `shortIban` appelé sans garde → rouge Vitest AC-10.

## Gates

`scripts/test-fast.sh` à chaque patch (kesh-db), gate frontend complet, E2E complet au dernier commit de code ; démarrage réel sur une base
persistante peuplée (P8 : le gate ne voit pas un checksum modifié).

## Manuels

Utilisateur, administration, notes de version, `api-external.md` (AC-11). **Prompt de revue : nommer les manuels et `api-external.md`**, PDF aplatis.

## Ce qui ne se traite pas — issues à demander

- **Un QR-IBAN sans IBAN** pour un compte bancaire : inatteignable (`bank` ⇒ IBAN), rien à tracer.
- **PayPal, caisse** : l'énumération est prête, le type ne l'est pas — rien à tracer tant que Guy ne le demande pas.
- **Un relevé de carte en CAMT** (`Acct/Id/Othr`) : le parseur l'exige en IBAN (`kesh-import/src/camt053/mod.rs:499-501`, `:827-829`) ; hors de
  cet epic (Guy reçoit du PDF et du CSV) : **issue demandée** (« Import CAMT : relevé d'un compte sans IBAN, `Acct/Id/Othr` »).

## Recoupements

`routes/bank_accounts.rs`, `bank-accounts/+page.svelte`, `format.ts` avec **34-1** et **34-2** ; `bank_imports.rs` avec **34-9** (import du dossier :
autres fonctions, même fichier) ; la migration avec **34-5** (ordre par horodatage). La **34-11b** s'appuie sur celle-ci.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5). Inventaire établi par une recherche dédiée (lecture seule), sites A et la forme du schéma recontrôlés à la
  main (`payment_batches.rs:858-869`, `20260410000001_bank_accounts.sql:5-17`, `bank_imports.rs:940-950`, `invoice_pdf_service.rs:220-226,330-344`,
  `iban_present` aux cinq lignes, `payment_batches.rs:155-176`).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
