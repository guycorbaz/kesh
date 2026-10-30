# Story v016-2 : saisir un relevé bancaire à la main

Status: ready-for-dev

*Fiche rédigée le 2026-10-30, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0162`, branche
`story/v016-2-releve-manuel` partie de `c5101b07` (tête de `story/v016-1-mcp-sorties`, candidate `v0.16.1-rc1`, PR #815
brouillon) ; la story se fusionne sur cette branche de release. Registre : `v016-2-choix-autonomes.md` (C-v016-2-1 à
C-v016-2-15). **Amendée le 2026-10-30 : la release devient la 0.17.0** (décision du Project Lead, C-v016-2-15) — la
clé et le nom de la story restent `v016-2-releve-manuel`. **Contrainte du Project Lead du même jour (C-v016-2-16)** :
les données de l'installation `v0.16.1-rc1` du NAS se gardent — montée sans perte, sauvegarde 0.16.x importable, rien
de jetable sur la base de Guy (AC-2-bis, AC-3-bis, AC-29, § *Ce que la release doit contenir*). **Remédiée après la
validation P1** (2026-10-30, Change Log ; choix C-v016-2-17). Clé courte : `v0162`. Issue : **#816** (enhancement), en `refs` seulement, dans les commits **et** dans la
PR de la story ; la fermeture se porte sur la **PR de release** (§ *Ce que la release doit contenir*). Les numéros de
ligne cités sont ceux de `c5101b07`.*

## Story

En tant que Guy, qui tient ses livres réels dans Kesh et dont un compte bancaire n'offre ni CAMT.053 ni CSV, je veux
**saisir un relevé à la main** — période, soldes de départ et d'arrivée, puis ses lignes — afin que ce compte entre dans
le circuit bancaire de Kesh comme les autres : transactions à rapprocher, propositions, annulation, suppression,
solde du relevé confronté aux livres sur la tuile et au contrôle de clôture, et une trace qui dit, dans la piste de
contrôle, que ce relevé a été tapé et non importé.

## Origine

- **#816** (2026-10-30, recette de la 0.16.1-rc1) : « Certaines banques ne fournissent aucun de ces formats pour un
  compte donné […]. Pour ce compte, le relevé n'existe que sur papier ou en PDF. » Critères de l'issue : (1) un relevé
  saisi produit des transactions rapprochables, comme un import ; (2) soldes de départ et d'arrivée contrôlés contre la
  somme des lignes ; (3) une saisie se distingue d'un import dans la piste de contrôle ; (4) les manuels décrivent les
  deux voies.
- **Décision du Project Lead du 2026-10-30** : la fonctionnalité entre dans la release en cours. *Première rédaction :
  la 0.16.1, par une candidate `v0.16.1-rc2`.* **Amendement du même jour (C-v016-2-15)** : la release devient la
  **0.17.0** — elle réunit le correctif v016-1 (#811, #812), publié comme candidate `v0.16.1-rc1` mais jamais en version
  finale, et cette story ; première candidate `v0.17.0-rc1`, sur laquelle la recette Claude Code et les tests se jouent
  une seule fois.
- Voisinage : **#418** (annuler un rapprochement, fermée) et **#526** (une transaction rejetée se retrouve et se
  dé-rejette, fermée) — le circuit dont un relevé saisi hérite ; **#671** (P2, ouverte) et **#687** (P3, ouverte) —
  défauts du rapprochement, sans lien avec l'origine d'une transaction : ni aggravés ni réglés ici.

## Ce que le code dit (relevé le 2026-10-30, `c5101b07`)

### Prémisses de #816 — confirmées

- **« Les transactions bancaires n'entrent dans Kesh que par l'import d'un fichier »** : vrai. Le seul créateur de
  `bank_transactions` hors tests est `bank_imports::create_with_transactions`
  (`crates/kesh-db/src/repositories/bank_imports.rs:72`), appelé par `create_import_or_roll_back`
  (`crates/kesh-api/src/routes/bank_imports.rs:83-98`) aux deux chemins de `POST /api/v1/bank-imports` (CAMT `:1154`,
  CSV `create_csv` `:2017`). L'inventaire `crates/kesh-api/tests/bank_account_closed_inventory.rs` le tient clos.
- **« Taper un CSV à la main puis l'importer : possible aujourd'hui »** : vrai, mais il faut un profil CSV, et un import
  CSV ne porte **aucun solde** (`user-manual.tex:2040-2041`, « Seul le CAMT.053 porte le solde de clôture […] ; un
  import CSV n'en a pas ») : la tuile du tableau de bord et le contrôle K3 de la clôture ne verraient jamais ce compte
  comme ayant un relevé (`bank_accounts.rs:1798-1804`, `closing_balance IS NOT NULL` ; `closing_checks.rs:450-457`,
  motif `NO_BALANCE_STATEMENT`).
- **« Passer directement les écritures au journal : le compte échappe au rapprochement »** : vrai ; et, plus
  précisément, il échappe aussi à la tuile (aucun relevé) et à K3.

### Prémisses du mandat — une corrigée

- **« Les transactions saisies sont-elles lues par les outils MCP comme les autres ? (Elles devraient l'être sans rien
  changer) »** : **inexact dans sa forme**. Aucun outil MCP ne lit `bank_transactions` ni `bank_imports` —
  `grep -rln "bank_transactions\|bank_imports" crates/kesh-api/src/mcp/` ne rend rien ; `mcp/outils/` porte
  `balance_agee`, `contacts`, `ecritures`, `exercices`, `factures`, `factures_fournisseur`, `grand_livre`,
  `kesh_version`, `pieces`, `plan_comptable`, `postes_ouverts`, `rapports`. Ce que l'agent voit d'un relevé, saisi ou
  importé, ce sont les **écritures** que produit son rapprochement. Conclusion inchangée : rien à faire côté MCP
  (C-v016-2-13).

### Ce sur quoi la conception s'appuie

- `bank_imports.source_format` : `VARCHAR(32) NOT NULL`, **sans** contrainte de valeur, délibérément
  (`20260504000001_bank_imports.sql:16-17`, `:25`) ; bornée par l'énumération Rust `BankImportSourceFormat`
  (`crates/kesh-db/src/entities/bank_import.rs:26-31`), dont `from_str` refuse toute autre valeur (`:53-60`).
  ⚠️ D'où la migration de C-v016-2-2 : une valeur nouvelle ne se décode pas dans un binaire antérieur.
- `bank_transactions.amount DECIMAL(18,2)` signé, positif = crédit du titulaire (`20260504000001_bank_imports.sql:7`,
  `:62`) ; `details TEXT NOT NULL` (`:65`) ; `reference VARCHAR(255)` (`:64`).
- `bank_imports.filename VARCHAR(255) NOT NULL`, `file_hash CHAR(64) NOT NULL` avec
  `CHECK (CHAR_LENGTH(file_hash) = 64)` (`:23-24`, `:51`) ; plus d'unicité depuis la 8-3 (le doublon de fichier est
  refusé par la route, `find_by_company_and_hash`).
- L'équilibre de l'import tolère un centime (`kesh-core/src/bank_imports.rs:254-271`, `diff > Decimal::new(1, 2)`) et
  s'admet sur confirmation (`routes/bank_imports.rs:1259-1278`).
- Les doublons de lignes de l'import sont **ignorés par défaut** (`ConfirmDuplicateLines::Skip`,
  `routes/bank_imports.rs:389-398` ; `apply_duplicate_lines_filter` `:1644-1696`).
- La trace de l'import : une action canonique `bank_import.created` (`insert_canonical_audit_log`,
  `routes/bank_imports.rs:1698-1750`) ; la suppression : `bank_import.deleted`, dont les détails portent déjà
  `source_format` (`:1540-1549`).
- Le rapprochement note référence 40 %, montant 50 %, contrepartie 10 % (`kesh-reconciliation/src/matching.rs:183`) ;
  aucun chemin de rapprochement ne lit `source_format`, `filename` ni `import_id` autrement que comme colonne
  (`grep -rn "source_format\|filename" crates/kesh-api/src/routes/reconciliation*.rs crates/kesh-db/src/repositories/reconciliation*.rs crates/kesh-reconciliation/src`
  : rien).
- Rôles : `POST /bank-imports/preview` et `POST /bank-imports` aux routes Comptable+ (`crates/kesh-api/src/lib.rs:792-803`),
  ouverts aux clés `read-write` (`docs/api-external.md:223`) ; la suppression fermée à toute clé
  (`routes/bank_imports.rs:1490-1493`).

## Décisions (détail et alternatives au registre)

| # | Décision | Registre |
|---|---|---|
| D1 | Un relevé saisi est un import de `source_format = 'MANUAL'`, créé par `create_with_transactions` ; aucune table, aucune colonne | C-v016-2-1 |
| D2 | Une migration dont la seule instruction relève `kesh_version_min_required` à `0.17.0` ; les dix crates passent à `0.17.0` dans le même commit (P2-bis) | C-v016-2-2, C-v016-2-15 **(Q1)** |
| D3 | Entête : compte, période, numéro facultatif, soldes signés ; lignes : date, valeur facultative, libellé, contrepartie et référence facultatives, montant positif + sens ; deux décimales ; 0 à 500 lignes ; CHF | C-v016-2-3 **(Q4)** |
| D4 | Équilibre **exact**, bloquant, sans confirmation ; dates de comptabilisation dans la période | C-v016-2-4 |
| D5 | Solde de départ ≠ clôture du relevé précédent : signalé à l'aperçu, confirmation exigée à la création, jamais bloqué | C-v016-2-5 **(Q3)** |
| D6 | Doublons de lignes : mêmes empreinte et fenêtre que l'import, **sans** « ignorer » — confirmer ou corriger ; relevé identique : `BANK_IMPORT_DUPLICATE_FILE` ; chevauchement : information ; saisi puis importé : décrit, issue demandée | C-v016-2-6 **(Q6)** |
| D7 | Correction : suppression du relevé entier (route existante), puis ressaisie ; aucune modification de ligne | C-v016-2-7 **(Q2)** |
| D8 | Écran `/bank-import/manual`, lien depuis `/bank-import`, aperçu puis enregistrement, totaux et écart en continu | C-v016-2-8 |
| D9 | Rôles et clés : ceux de l'import (Comptable+, clés `read-write`) | C-v016-2-9 **(Q5)** |
| D10 | Trace : action `bank_import.entered_manually` ; suppression : `bank_import.deleted` inchangée | C-v016-2-10 |
| D11 | `filename` = `saisie-manuelle-<du>-<au>` ; `file_hash` = SHA-256 du contenu canonique | C-v016-2-11 |
| D12 | API JSON `POST /api/v1/bank-imports/manual/preview` et `POST /api/v1/bank-imports/manual`, sans empreinte d'aperçu | C-v016-2-12 |
| D13 | MCP : rien à changer | C-v016-2-13 |
| D14 | Pas de découpage (cinq modules, un domaine) | C-v016-2-14 |

### Le corps de requête (préview et création)

```json
{
  "bankAccountId": 3,
  "periodFrom": "2026-09-01",
  "periodTo": "2026-09-30",
  "statementNumber": "9/2026",
  "openingBalance": "1250.40",
  "closingBalance": "980.15",
  "lines": [
    { "bookingDate": "2026-09-03", "valueDate": null, "details": "Prélèvement assurance",
      "counterpartyName": "Assurance Exemple SA", "reference": null, "amount": "320.25", "direction": "debit" },
    { "bookingDate": "2026-09-15", "valueDate": "2026-09-15", "details": "Virement reçu",
      "counterpartyName": null, "reference": "F-2026-0042", "amount": "50.00", "direction": "credit" }
  ],
  "confirmDuplicateFile": false,
  "confirmDuplicateLines": false,
  "confirmOpeningBalanceGap": false
}
```

(Données d'exemple **synthétiques**.) Montants et soldes en **chaînes** décimales (comme partout dans l'API) ; l'aperçu
ignore les trois drapeaux. `statementNumber`, `valueDate`, `counterpartyName`, `reference` : facultatifs, une chaîne
vide ou faite de blancs vaut absence. La transaction enregistrée porte `amount = +montant` pour `credit`, `−montant`
pour `debit`, `currency = "CHF"`, `end_to_end_id`, `transaction_id`, `counterparty_iban` à `NULL`.

### Réponse de l'aperçu (`200`, aucune écriture)

`{ "totalCredits", "totalDebits", "computedClosingBalance", "transactionCount", "warnings": { "duplicateFile":
DuplicateFilePayload | null, "duplicateLines": [{ "line": <rang 1-based>, "existingTransactionId", "key" }],
"openingBalanceGap": { "expected", "entered", "previousImportId", "previousPeriodTo" } | null,
"overlappingStatements": [{ "importId", "periodFrom", "periodTo", "sourceFormat" }] } }`. Un relevé déséquilibré ou
invalide rend à l'aperçu **la même** erreur qu'à la création (une seule fonction de validation).

### Codes d'erreur

| Code | HTTP | Quand | Charge utile |
|---|---|---|---|
| `BANK_STATEMENT_INVALID` | 400 | une règle de l'AC-4 (r. 1 à 5) violée ; `field` et `reason` pris dans les ensembles **fermés** ci-dessous | `{ "line": <rang 1-based dans le tableau envoyé> \| null, "field": "<nom JSON>", "reason": "<code>" }` |
| `BANK_STATEMENT_UNBALANCED` | 422 | `départ + crédits − débits ≠ arrivée` | `{ "opening", "closing", "computedClosing", "diff" }` |
| `BANK_STATEMENT_DUPLICATE_LINES` | 422 | doublons sans `confirmDuplicateLines` | `{ "lines": [{ "line", "existingTransactionId" }] }` |
| `BANK_STATEMENT_OPENING_BALANCE_GAP` | 422 | écart de solde de départ sans `confirmOpeningBalanceGap` | `{ "expected", "entered", "previousImportId", "previousPeriodTo" }` |
| `BANK_IMPORT_DUPLICATE_FILE` | 422 | relevé identique déjà enregistré, sans `confirmDuplicateFile` | celle de l'import |
| `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` | 404 | compte absent, d'une autre société ou archivé | — |
| `BANK_ACCOUNT_CLOSED` | 409 | compte clos | celle de l'import |

**Ensembles fermés de `BANK_STATEMENT_INVALID`** (validation P1, F7) — `field` ∈ { `periodFrom`, `periodTo`,
`statementNumber`, `openingBalance`, `closingBalance`, `lines`, `bookingDate`, `valueDate`, `details`,
`counterpartyName`, `reference`, `amount`, `direction` } (`line` non nul pour les sept derniers) ; `reason` ∈ {
`required` (absent ou vide), `invalid_date` (chaîne qui n'est pas `AAAA-MM-JJ`), `invalid_number` (chaîne qui n'est pas
un décimal), `period_inverted` (r. 1), `too_many_lines` (r. 2), `out_of_period` (r. 3), `too_long` (r. 4),
`not_positive` (r. 5, montant), `too_many_decimals` (r. 5), `too_many_digits` (r. 5), `unknown_direction` }. Dates et
montants arrivent en **chaînes** et sont analysés par la route, afin que toute faute de forme rende ce code avec son
rang (et non un rejet Axum sans ligne). Côté frontend, une table `reason → clé i18n` littérale
(`bank-import-manual-reason-*`, une clé par `reason`, aucune clé fabriquée par interpolation) et un test de couverture
réciproque (chaque `reason` du serveur a sa clé, chaque clé a son `reason`), sur le patron de `CSV_LINE_ERROR_LABELS`
(`frontend/src/lib/shared/i18n-keys.test.ts`).

Ordre des refus à la création, chacun **sans écriture** : authentification et rôle (middlewares) ; corps JSON illisible
(`400`, Axum) ; compte (`404`, `409`, pré-vol sur le pool comme l'import, `routes/bank_imports.rs:1161-1180`) ;
`BANK_STATEMENT_INVALID` ; `BANK_STATEMENT_UNBALANCED` ; puis, **dans** la transaction ouverte, relevé identique,
doublons de lignes, écart de solde de départ — dans cet ordre, le premier qui manque de sa confirmation l'emporte.

## Critères d'acceptation

### Le modèle et la migration

- **AC-1** — `BankImportSourceFormat` (`kesh-db`) et `SourceFormatTag` (`kesh-core`) gagnent la variante `Manual`, codée
  `"MANUAL"` ; `as_str`/`as_db_str`, `from_str` et `Display` la couvrent ; `source_format_tag_for_kesh_core`
  (`routes/bank_imports.rs:978-984`) aussi. Aucun chemin d'**import de fichier** ne peut produire `MANUAL`
  (`version_to_source_format`, `from_imported` inchangés). Tests : aller-retour des quatre valeurs ; `"manual"`
  (minuscules) refusé.
- **AC-2** — Migration `crates/kesh-db/migrations/20261030000001_bank_imports_manual_source.sql` : un en-tête qui dit
  pourquoi (C-v016-2-2), puis **une seule** instruction, `UPDATE _kesh_version SET kesh_version_min_required =
  '0.17.0' WHERE id = 1;`. **P2-bis, dans le même commit que la migration** : les dix crates du workspace
  (`crates/*/Cargo.toml`, `version = "0.16.1"` à `c5101b07`) passent à `version = "0.17.0"`, et `Cargo.lock` suit
  (régénéré par le build, non édité à la main) ; contrôle : `grep -n '^version' crates/*/Cargo.toml` rend dix fois
  `0.17.0`. Puis **greper la valeur** `grep -rn "0\.16\.1" crates frontend/src docs scripts README.md CHANGELOG.md
  website --exclude-dir=node_modules` et trier chaque occurrence à la main : à `c5101b07`, hors `Cargo.toml`, `crates/`
  ne la porte qu'en commentaires « depuis la 0.16.1 » qui datent le correctif v016-1 (`mcp/prompts.rs:206`,
  `mcp/sortie.rs:8`, `:446`, `mcp/politique.rs:17`, `tests/mcp_transport_e2e.rs:110`, `tests/mcp_outils_e2e.rs:1415`,
  `tests/mcp_envois_e2e.rs:188`, `tests/common/mcp_pieges.rs:126`, `tests/common/mcp.rs:556`) — ils passent à « la
  0.17.0 », version qui livre le correctif ; aucun test ne fige `"0.16.1"` en littéral (ceux qui lisent la version
  passent par `env!("CARGO_PKG_VERSION")`). Les occurrences des manuels, du README, du site et des macros relèvent de
  la release (§ *Ce que la release doit contenir*), sauf le § *Passer à la 0.16.1* (AC-29). Le tri, occurrence par
  occurrence, au Dev Agent Record. P5 : une ligne à `docs/migrations-idempotence-audit.md` (verdict, justification) et les
  **deux** sites du total recomptés (`ls crates/kesh-db/migrations/*.sql | wc -l` → 87 ;
  `grep -c '^| \`20' docs/migrations-idempotence-audit.md` → 87), les trois compteurs de partition recomptés depuis le
  tableau. P6 : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` (46 occurrences à `c5101b07`), chaque site
  inspecté, sa conclusion au Dev Agent Record. P7 : entrée `EXEMPT_MIGRATIONS` (`crates/kesh-db/src/post_restore.rs`,
  `ExemptionBasis::Durable`, justification « UPDATE sur _kesh_version (bump kesh_version_min_required), table système
  jamais restaurée. La migration ne porte aucune autre instruction. »), et le compteur figé de
  `post_restore.rs:1216-1219` (`assert_eq!(EXEMPT_MIGRATIONS.len(), 21, …)`) passe à **22**, avec sa ligne
  d'historique (« 21 → 22 (Story v016-2) »). Squash régénéré par
  `scripts/regen-test-schema.sh` (sa ligne `_kesh_version` passe à `'0.17.0'`), `test_schema_guard` vert. Les tests qui
  figent le dernier bump (`migrations_fresh_install.rs:230-247`, `migrations_upgrade_path.rs:537-569`, et ce que
  `grep -rn "0\.14\.0" crates --include=*.rs` rend d'autre) passent à `0.17.0`, avec leur commentaire ; un test
  `check_downgrade_protection(&pool, "0.16.1")` → `DowngradeRefused` (la dernière version antérieure, candidate
  comprise : la `v0.16.1-rc1` porte la version Cargo `0.16.1`).
- **AC-2-bis — Montée sans perte depuis une base 0.16.x peuplée** (C-v016-2-16). L'installation du NAS tourne en
  `v0.16.1-rc1` (schéma de la 0.16.0, 86 migrations ; v016-1 n'en ajoute aucune) et porte des données à garder. Un test
  `#[sqlx::test(migrations = false)]` de `crates/kesh-db/tests/migrations_upgrade_path.rs`, sur le patron de
  `imported_documents_without_qr_preserves_existing_rows` (`:668-…`), monte le schéma **jusqu'à la migration
  exclue** (`common::migrations_before(20261030000001, "bank_imports_manual_source")` — résolution par version, P6),
  puis insère, dans une société de test : un import `CAMT053_V04` à soldes **et à `statement_id` non nul**, un import `CSV` sans solde, leurs
  transactions (au moins une `pending`, une `reconciled` avec `matched_entry_id` vers une écriture, une rejetée —
  `auto_match_rejected_at` posé ; `value_date`, `reference` et `counterparty_name` non nuls sur au moins une), et lit
  `_kesh_version`. Il applique ensuite **le reste** des migrations et vérifie :
  1. chaque ligne de `bank_imports` et de `bank_transactions` se relit **colonne par colonne à l'identique** (même
     comparaison avant/après, `version` et `updated_at` compris : la migration ne touche aucune ligne) ;
  2. les mêmes lignes se relisent **par le code applicatif** : `bank_imports::find_by_id_for_company` (donc
     `BankImportSourceFormat::from_str` sur `CAMT053_V04` et `CSV`) et `bank_transactions::list_by_import` réussissent
     et rendent les valeurs posées ;
  3. le rapprochement tient : `status`, `matched_entry_id`, l'écriture liée et ses lignes inchangées ;
  4. `_kesh_version.kesh_version_min_required` vaut `0.17.0` ; `kesh_version_last_applied` inchangé (le boot l'écrit,
     non la migration).
  Et, inventaire décidable : la migration ne porte **aucune** instruction autre que l'`UPDATE _kesh_version` — un test
  (ou l'assertion du triage P7) lit le fichier décommenté et l'établit ; ni `DROP`, ni `ALTER`, ni écriture d'une table
  applicative.
- **AC-3** — La sauvegarde et la restauration : un `.keshbackup` exporté d'une base qui porte un relevé `MANUAL` se
  réimporte dans la même version, relevé et transactions identiques (`source_format`, soldes, lignes) ; son manifeste
  porte `kesh_version_min_required = "0.17.0"`. Test : l'aller-retour, sur le patron des tests d'import d'installation
  existants (`admin_full_import_e2e.rs` ou voisin — à citer au Dev Agent Record).
- **AC-3-bis — Une sauvegarde `.keshbackup` faite en 0.16.x s'importe dans la 0.17.0** (C-v016-2-16). La 0.17.0
  n'ajoute **aucune table** : un test l'établit (l'inventaire des tables applicatives après le `MIGRATOR` complet égale
  celui de `migrations_before(20261030000001, …)`). Un test e2e de `crates/kesh-api/tests/` importe dans la 0.17.0 une
  sauvegarde **dont le manifeste est celui d'une 0.16.x** (`keshVersion` `0.16.1`, `keshVersionMinRequired` `0.14.0`),
  portant un import CAMT, un import CSV, des transactions dont une rapprochée : `200`, puis chaque import et chaque
  transaction se relit à l'identique par `GET /api/v1/bank-imports` et `GET /api/v1/bank-imports/{id}` ; la base garde
  `min_required = 0.17.0` (`_kesh_version` n'est pas restaurée) ; un relevé saisi se crée ensuite normalement. Moyen,
  par ordre de préférence, le choix et sa raison au Dev Agent Record : (a) exporter depuis la 0.17.0 puis réécrire le
  manifeste comme l'écrirait une 0.16.x, si le format de l'archive le permet sans contourner sa vérification
  d'intégrité (lire `parse_and_verify`) ; (b) sinon, une fixture `.keshbackup` **synthétique** produite par un binaire
  `0.16.1-rc1` sur une installation **jetable** (jamais celle du NAS, jamais de données réelles), versée sous
  `crates/kesh-api/tests/fixtures/`, avec sa recette de production en commentaire. Le sens inverse reste refusé :
  une sauvegarde de la 0.17.0 dans un binaire `0.16.1` → `409` (`ImportVersionIncompatible`), testé par
  `check_import_version_compat("0.17.0", "0.16.1")`.

### La validation (fonction pure, `kesh-core`)

- **AC-4** — `kesh_core::bank_imports::manual` (module nouveau, **zéro I/O**) expose le type d'entrée du relevé, une
  fonction `validate_manual_statement(&input) -> Result<ManualStatementSums, ManualStatementError>` et une fonction
  `manual_statement_to_drafts(&input, bank_account_id, company_id, file_hash, filename, imported_at, user_id) ->
  (BankImportDraft, Vec<BankTransactionDraft>)` (`source_format = SourceFormatTag::Manual`, `opening_balance` et
  `closing_balance` à `Some`, `statement_id` = numéro, montants signés selon le sens), et
  `manual_statement_canonical_bytes(&input) -> Vec<u8>` (préfixe `kesh-manual-statement-v1`, puis compte, période,
  numéro, soldes, et chaque ligne dans l'ordre avec tous ses champs normalisés — `trim`, absence ≡ chaîne vide,
  montants `normalize()`-és ; **chaque champ encodé par sa longueur sur quatre octets gros-boutiens suivie de ses
  octets UTF-8**, le nombre de lignes de même, si bien qu'aucun contenu ne peut imiter un séparateur — validation P1,
  R8). Règles de `validate_manual_statement`, chacune testée :
  1. `periodFrom ≤ periodTo` ;
  2. 0 à 500 lignes ;
  3. chaque `bookingDate` dans `[periodFrom, periodTo]` ; `valueDate` libre ;
  4. `details` non vide après `trim`, au plus 500 caractères ; `counterpartyName`, `reference`, `statementNumber` au
     plus 255 **caractères** (`chars().count()`, comme `routes/api_keys.rs:130` : en MariaDB, `VARCHAR(255)` compte des
     caractères — validation P1, R7) ;
  5. montant `> 0`, au plus deux décimales **après `normalize()`** (`"50.00"` et `"50.000"` admis, `"50.0100"` ≡
     `"50.01"` admis, `"50.001"` refusé — `"50.000"` est le cas qui distingue la règle de sa mutation M4), au plus
     treize chiffres entiers ; soldes : mêmes bornes, signe libre ;
  6. **équilibre exact** : `opening + Σ crédits − Σ débits == closing`, sans tolérance (`"0.01"` d'écart refusé).
  L'erreur porte le rang de ligne (1-based) et le champ en cause. `validate_balance` et `from_imported` ne changent pas.
- **AC-5** — L'empreinte canonique est **stable** (même entrée → mêmes octets, quel que soit l'ordre d'émission des
  champs JSON côté client, `"50"` ≡ `"50.00"`) et **sensible** à chacun des champs qu'elle porte : un test par champ
  (compte, bornes, numéro, chaque solde, et pour une ligne : date, valeur, libellé, contrepartie, référence, montant,
  sens), plus l'ordre des lignes, plus **l'injectivité** : `details = "a|b", reference = "c"` et `details = "a",
  reference = "b|c"` (et toute paire qui ne diffère que par la frontière entre deux champs voisins) donnent des octets
  différents.

### Le dépôt (`kesh-db`) — deux lectures, aucune écriture nouvelle

- **AC-6** — `bank_imports::previous_statement_closing(executor, company_id, bank_account_id, before: NaiveDate) ->
  Option<(i64, NaiveDate, Decimal)>` : le relevé du compte **à solde** (`closing_balance IS NOT NULL`) dont
  `period_to < before`, le plus tardif selon `period_to DESC, imported_at DESC, id DESC` (l'ordre de
  `bank_accounts.rs:1802`). Tests `#[sqlx::test]` : ordre (deux relevés, le plus tardif gagne ; égalité de `period_to`
  départagée), exclusion d'un relevé sans solde (CSV), d'un relevé qui finit le jour même de `before`, d'un autre compte,
  d'une autre société.
- **AC-7** — `bank_imports::overlapping_statements(executor, company_id, bank_account_id, from, to) ->
  Vec<BankImport>` : `period_from <= to AND period_to >= from`. Tests : bornes jointives incluses (un relevé qui finit
  le jour où l'autre commence chevauche), disjoints exclus, autre compte et autre société exclus.
- **AC-8** — Aucune instruction `INSERT`, `UPDATE`, `DELETE` ou `REPLACE` nouvelle dans `crates/kesh-db/src/` ni
  `crates/kesh-api/src/` : la création passe par `create_with_transactions`, la suppression par
  `delete_unreconciled_in_tx`. `bank_account_closed_inventory.rs` reste vert **sans modification** de son `REGISTRE`
  ni de sa `FENETRE` (s'il faut les toucher, c'est qu'un écrivain est né : le dire au Dev Agent Record et le justifier).

### Les routes (`kesh-api`)

- **AC-9** — `POST /api/v1/bank-imports/manual/preview` et `POST /api/v1/bank-imports/manual`, montés **aux routes
  Comptable+** à côté de l'import (`lib.rs:792-806`), corps JSON (§ *Le corps de requête*), dans un module nouveau
  `crates/kesh-api/src/routes/bank_statements_manual.rs`. Ils réutilisent, rendues `pub(crate)` si besoin :
  `create_import_or_roll_back`, `import_create_error`, `compute_sha256_hex`, `duplicate_file_payload`,
  `import_to_response`, `insert_canonical_audit_log` (l'action en paramètre), `detect_duplicate_lines` /
  `dedup_key_scalar` (via un adaptateur sur `BankTransactionDraft`, sans recopier la conversion de
  `detect_duplicate_lines_for_imported`). Aucune requête SQL écrite dans le module.
- **AC-10** — Création réussie : `201` et le `BankImportResponse` de l'import (`sourceFormat: "MANUAL"`,
  `filename: "saisie-manuelle-<du>-<au>"`, `statementId`, soldes, `transactionCount`) ; en base, l'entête et **toutes**
  les lignes, `status = 'pending'`, montants signés selon le sens, `currency = 'CHF'` ; **une** trace
  `bank_import.entered_manually` (`entity_type = 'bank_imports'`, `entity_id` = l'import) dont les détails portent
  `filename`, `transaction_count`, `source_format: "MANUAL"`, `modifiers` (triés, dédoublonnés), `period_from`,
  `period_to`, `opening_balance`, `closing_balance`, `statement_id`, et, selon le cas, `duplicate_lines_imported`,
  `expected_opening_balance`. Entête, lignes et trace dans **la même** transaction (un échec de la trace n'écrit rien).
- **AC-11** — Chaque refus du tableau *Codes d'erreur* rend son code et sa charge utile, et **n'écrit rien** (ni
  entête, ni ligne, ni trace) — vérifié en base pour chacun. Avec la confirmation correspondante, la création passe et
  la trace porte le modificateur : `duplicate_file`, `duplicate_lines_imported`, `opening_balance_gap`. Les doublons de
  lignes ne sont **jamais** ignorés en silence : sans confirmation, refus ; avec, toutes les lignes sont enregistrées.
- **AC-12** — L'aperçu n'écrit rien (aucune ligne d'aucune table, trace comprise) et rend les quatre avertissements ;
  il rend `BANK_STATEMENT_INVALID` et `BANK_STATEMENT_UNBALANCED` comme la création ; compte archivé, clos ou étranger :
  mêmes refus que la création.
- **AC-13** — Rôles et clés : Consultation → `403` ; clé `read` → `403 API_KEY_READ_ONLY` ; clé `read-write` → `201`,
  et la trace porte l'auteur clé (`for_actor`, comme l'import). La suppression d'un relevé saisi par une clé reste
  refusée (`403 API_KEY_OPERATION_FORBIDDEN`, inchangé).
- **AC-14** — Concurrence : la garde du compte clos/archivé est celle de `create_with_transactions` (garde de tête et
  revérification verrouillante) ; un test montre qu'un compte **clos entre le pré-vol et la création** fait refuser la
  création sans écriture. Ce test est de **niveau dépôt** (la fenêtre ne s'atteint pas en HTTP sans crochet) : le
  patron est `crates/kesh-db/tests/bank_accounts_repository.rs:2481` (`un_import_commence_avant_l_archivage_est_refuse`)
  et `:2252` (`l_import_refuse_l_archive_et_le_clos_au_depot`), étendus à un `NewBankImport` de `source_format =
  Manual` ; côté HTTP, seul le pré-vol (`404`, `409`) se teste.
- **AC-15** — Registres : `tests/audit_route_registry.rs` classe les deux routes (`preview` : `NoMatter`, aucune
  mutation ; `create` : `Traced`, `SansEcritureAuJournal`) ; `audit_labels.rs` gagne `bank_import.entered_manually` et
  les quatre catalogues son libellé (`fr-CH` : « Relevé bancaire saisi à la main ») ; `tests/audit_label_registry.rs`
  vert. **Forme prescrite** (validation P1, R1) : `insert_canonical_audit_log` reçoit un type défini **dans
  `routes/bank_imports.rs`** — `enum ImportAuditAction { Created, EnteredManually }` et sa méthode `code()`, dont le
  `match` porte les deux littéraux `"bank_import.created"` et `"bank_import.entered_manually"` dans ce fichier — ;
  l'appel `NewAuditLogEntry::for_actor(…, action.code(), …)` devient indirect, et une entrée de `SITES_INDIRECTS`
  (`audit_label_registry.rs`, patron du ternaire `if paused` d'`invoices.rs`, `:110-114`) déclare
  `crates/kesh-api/src/routes/bank_imports.rs` avec les **deux** codes, que le sens 1 de la garde (`:534-545`) trouve
  alors dans le fichier. Le module `bank_statements_manual.rs` ne passe que `ImportAuditAction::EnteredManually`, sans
  littéral d'action. Gate ciblé : `binary(audit_label_registry)`.
  **Compteurs figés du registre des routes** (R2), à recompter, non à ajuster de confiance :
  `audit_route_registry.rs:909` (`LIB_ROUTES.len()`, 135 → 137), `:915` (`traced`, 129 → 130), `:939` (`no_matter`,
  2 → 3), `:944` (138 → 140), `:1324` (`assert_eq!(tout.len(), 138)` → 140), et les commentaires d'en-tête (l. 27-33).
  Le test `le_manuel_dit_le_compte_du_registre` (`:955`) lit `docs/manual/fr/admin-manual.tex:2966` : la phrase devient
  « 130 des 137 routes à verbe mutant », « trois autres routes qui n'écrivent rien parce qu'elles ne modifient rien »,
  « 130 + 4 + 3 = 137 » (vérifier au `.tex` la forme exacte que le test exige).

### Le circuit — ce qu'un relevé saisi hérite (tests de bout en bout HTTP)

- **AC-16** — Une transaction saisie au crédit, du montant du reste dû d'une facture client validée, apparaît dans
  `GET /api/v1/reconciliation/proposals?bankAccountId=…` avec cette facture pour candidate, s'accepte
  (`POST /reconciliation/accept`), passe `reconciled` et règle la facture ; son rapprochement s'**annule** (route
  d'annulation de #418) ; une ligne au débit se rapproche **manuellement** d'un compte de charges. Rien de cela ne lit
  `source_format` : le test le prouve sur un relevé `MANUAL`, non sur un import.
- **AC-17** — Suppression : `DELETE /api/v1/bank-imports/{id}` d'un relevé saisi non rapproché → `204`, entête et lignes
  disparus, trace `bank_import.deleted` avec `source_format: "MANUAL"` ; rapproché → `409
  BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`. Après suppression, le même relevé se ressaisit **sans**
  `confirmDuplicateFile`.
- **AC-18** — Le solde : après la saisie d'un relevé sur un compte lié à un compte du grand livre, `GET
  /api/v1/bank-accounts` rend, pour ce compte, `statementClosingBalance` = le solde d'arrivée saisi et
  `statementDate` = la fin de la période (champs JSON de `routes/bank_accounts.rs:176-187` ; ce que la tuile affiche),
  et `statementGap` selon les écritures ; et le contrôle K3 de la préparation de clôture ne rend plus `NO_BALANCE_STATEMENT` pour ce
  compte (un test par l'une des routes existantes de la préparation de clôture).

### L'écran (`frontend`)

- **AC-19** — `/bank-import` porte, pour un Comptable ou un Administrateur, un lien **« Saisir un relevé à la main »**
  (`data-testid="bank-import-manual-link"`) vers `/bank-import/manual` ; un utilisateur Consultation ne le voit pas, et
  `/bank-import/manual` le renvoie comme le font les autres écrans de saisie (patron du dépôt, à citer).
- **AC-20** — Le formulaire `ManualStatementForm.svelte` (`frontend/src/lib/features/bank-import/`) : compte (comptes
  de `GET /companies/current`, comme le dépôt), période (du, au), numéro (facultatif), solde de départ, solde d'arrivée,
  puis la table des lignes — Date, Valeur, Libellé, Contrepartie, Référence, **Débit**, **Crédit** — avec *Ajouter une
  ligne* et un bouton de retrait par ligne. Une ligne porte exactement un des deux montants. Le formulaire s'ouvre avec
  **une** ligne vide ; une ligne entièrement vide est retirée à l'envoi, et le formulaire **garde la correspondance**
  entre le rang envoyé et la ligne affichée : tout rang rendu par le serveur (`line` d'une erreur, `line` d'un doublon)
  se traduit en ligne de l'écran avant d'être affiché (validation P1, R10/F5). Vitest : une ligne vide au milieu, une
  faute ou un doublon sur la suivante → c'est **cette** ligne qui est désignée.
- **AC-21** — Sous la table, mis à jour à chaque frappe, par une fonction pure `manual-statement.ts` (`big.js`, jamais
  `parseFloat`) : total des débits, total des crédits, solde calculé (`départ + crédits − débits`), solde d'arrivée
  saisi, **écart** (`arrivée − calculé`), au format suisse ; un solde vide n'est **pas** zéro : l'écart ne s'affiche
  pas tant que les deux soldes ne sont pas saisis. *Enregistrer* est **inactif** tant que : l'écart n'est pas nul ; un
  solde est vide ou invalide ; une ligne non vide viole une règle de l'AC-4 vue côté client — date de comptabilisation
  vide, illisible ou hors période (la date de valeur est libre), libellé vide ou de plus de 500 caractères,
  contrepartie, référence ou numéro de plus de 255 caractères, zéro ou deux montants, montant nul ou hors forme — ;
  la période est inversée ou les lignes dépassent 500 ; l'aperçu n'a pas été fait sur l'état courant du formulaire ;
  ou une case de confirmation **exigée** par l'aperçu n'est pas cochée. La raison est dite à côté du bouton. Le format
  se contrôle par `isValidAmount` **généralisée** (validation P1, R3/F4) : options `maxDecimals` (défaut 4, deux ici),
  `maxIntegerDigits` (défaut 15, treize ici), `allowNegative` (défaut non ; oui pour les soldes, non pour les montants
  de ligne) et `allowEmpty` (défaut oui, comme le journal ; non pour les soldes) — **aucune seconde expression
  régulière** ; la virgule est admise à la saisie et **normalisée en point** dans le corps JSON (`parseAmount`).
  Vitest : solde `-120,50` accepté et envoyé `"-120.50"` ; quatorze chiffres refusés ; solde vide refusé et sans écart
  affiché ; `"0"` refusé comme montant de ligne ; `balance.test.ts` inchangé et vert (défauts du journal intacts).
- **AC-22** — *Vérifier* appelle l'aperçu ; ses avertissements s'affichent (lignes en doublon désignées par leur rang,
  relevé identique, solde de départ attendu et saisi, relevés qui chevauchent), chacun avec sa case de confirmation
  quand la création l'exige. Toute modification **d'un champ du relevé** après l'aperçu le périme (cases décochées,
  *Enregistrer* inactif jusqu'au nouvel aperçu) ; cocher ou décocher une case ne le périme pas. Les cases cochées
  sont **envoyées** dans le corps (`confirmDuplicateFile`, `confirmDuplicateLines`, `confirmOpeningBalanceGap`) —
  Vitest sur le corps envoyé. Une erreur du serveur (`BANK_STATEMENT_*`) s'affiche traduite, la ligne fautive
  désignée.
- **AC-23** — Succès : notification, retour à `/bank-import`, le relevé en tête de la liste. Dans la liste
  (`BankImportList.svelte`) et au détail (`/bank-import/[id]`), un import `MANUAL` s'affiche **« Saisie manuelle »** (et
  son numéro s'il en a un) au lieu du `filename`, et son format **« Saisie manuelle »** au lieu de `MANUAL` ; la boîte de
  suppression et sa notification parlent d'un « relevé saisi à la main » (clés propres, non le texte « Le fichier
  { $filename } »). **Inventaire fermé** (validation P1, F9) — pour un import `MANUAL`, des variantes propres de :
  `DeleteBankImportAction.svelte` — titre (`bank-import-delete-dialog-title`), bouton (`bank-import-delete-button`),
  paragraphe bloqué (`bank-import-delete-blocked`), corps (`bank-import-delete-dialog-body`, dont « réimporter le même
  fichier », faux pour une saisie), erreurs (`bank-import-delete-error-reconciled`, `-not-found`) — ; notification
  `bank-import-delete-success` (`[id]/+page.svelte:37`) ; au détail (`[id]/+page.svelte:113-118`), les libellés
  « Fichier », « Importé le », « Format source », le titre de page : passés en clés `bank-import-detail-*` aux quatre
  catalogues, avec, pour un relevé saisi, « Relevé », « Saisi le », « Saisie manuelle » et le numéro.
- **AC-24** — i18n : toutes les clés nouvelles (`bank-import-manual-*`, et les codes d'erreur `BANK_STATEMENT_*`) dans
  les **quatre** catalogues (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`), aucun texte en dur dans le
  balisage ; `npm run lint-i18n-ownership` vert (préfixe `bank-import-` admis pour `features/bank-import/`, aucune clé
  d'un autre domaine réutilisée — pas de `journal-entry-form-*`). Gardes qui rougiront et se recomptent depuis la
  source (validation P1, F8) : `frontend/src/lib/shared/i18n-keys.test.ts` (`ATTENDU.sitesTotal`, cardinalité exacte),
  `i18n-un-repli-par-cle.test.ts`, `i18n-repli-divergent-actif.test.ts` (repli = valeur `fr-CH`),
  `i18n-libelle-en-dur.test.ts`, et la parité des catalogues (`crates/kesh-i18n/src/loader.rs`,
  `parity_between_locales`).
- **AC-25** — Recette visuelle (`CLAUDE.md` § *Recette visuelle*, thème clair) : captures, sur une base de
  démonstration, de `/bank-import` (le lien), du formulaire vide, d'un relevé de trois lignes **déséquilibré** (écart
  affiché, *Enregistrer* inactif), du même **équilibré** après aperçu, des avertissements (doublon, solde de départ),
  de la liste avec un relevé saisi, du détail, et de la boîte de suppression ; et aussi (validation P1, F10) : une ligne
  invalide avec la raison à côté du bouton, une erreur serveur avec la ligne désignée, l'aperçu périmé, l'avertissement
  « relevé identique » et l'information de chevauchement, les cases cochées, la notification de succès, le détail d'un
  relevé saisi avec ses transactions, `/bank-import` vu par un Consultation (lien absent), et le filtre d'action du
  journal d'audit avec l'option nouvelle. Rangées dans
  `/home/gcorbaz/devel/kesh-gate-logs/v0162/captures/`, listées au Dev Agent Record et jointes au prompt de revue.

### Le parcours complet (Playwright)

- **AC-26** — `frontend/tests/e2e/bank-manual-statement.spec.ts` : un Comptable (montants **uniques** par exécution,
  la base étant partagée entre specs — patron de `reconciliation-amount-due.spec.ts`) crée et valide une facture par
  l'API — datée du jour, et les lignes du relevé datées du jour ou de la veille, dans l'exercice ouvert du seed et dans
  la fenêtre de proposition (au plus 30 jours, Story 28-7, `reconciliation-amount-due.spec.ts:80-81`) — ; sur un
  **compte bancaire propre à l'exécution** (`wiredBankAccountViaApi` avec un IBAN suisse synthétique **unique**, clé
  MOD-97 calculée), si bien qu'aucun relevé antérieur n'existe et qu'aucun avertissement de solde de départ ne peut
  surgir d'une exécution à l'autre ; ouvre `/bank-import`, suit le lien, saisit un relevé de **trois** lignes dont un crédit égal au TTC de la
  facture, constate l'écart, corrige le solde d'arrivée, *Vérifier*, *Enregistrer* ; retrouve le relevé « Saisie
  manuelle » dans la liste ; ouvre `/reconciliation`, choisit le compte, accepte la proposition de la facture ; ouvre
  `/audit-log`, filtre sur l'action « Relevé bancaire saisi à la main » et retrouve l'entrée. Un second test : un
  utilisateur Consultation ne voit pas le lien.

### Les textes

- **AC-27** — Manuel utilisateur FR (`docs/manual/fr/user-manual.tex`, PDF régénéré) : une sous-section
  *Saisir un relevé à la main* (`\label{sec:releve-manuel}`) dans *Import bancaire* — quand l'employer (un compte sans
  relevé électronique), ce qu'on saisit, le contrôle d'équilibre exact, le solde de départ attendu, les doublons (pas
  d'« ignorer »), la correction (supprimer puis ressaisir ; annuler d'abord un rapprochement), et **la conduite si la
  banque se met plus tard au CAMT** (C-v016-2-6). Les phrases devenues fausses se récrivent : `user-manual.tex:2039-2041`
  (« Kesh supporte deux formats. Seul le CAMT.053 porte le solde de clôture […] »), `:255-258` (« Seuls les relevés
  CAMT.053 portent un solde de clôture »), `:2264-2276` (doublons : la saisie ne propose pas « ignorer »). **Inventaire
  complété à la validation P1 (F1)** — *la citation « `:803` » de la première rédaction était fausse : la phrase K3
  est à `:813`* : `:813` (« Les relevés de fin d'exercice ne se comparent que pour les relevés CAMT, qui portent un
  solde ; un fichier CSV n'en porte pas » — et « un compte alimenté par CSV seulement » : ni saisi), `:814` (« une
  transaction importée » → importée ou saisie), `:2046-2052` (« À défaut de CAMT.053, importez le relevé en CSV… le
  format MT940 n'est pas lu » : **renvoyer** à `\ref{sec:releve-manuel}`, c'est le cas de #816), `:2126` (« portant un
  solde de clôture (un CAMT.053) »), `:2216` (« Un import CSV ne porte donc toujours pas de solde » : à relire),
  `:49` (« d'importer vos relevés bancaires (CAMT.053 et CSV) »), `:1925`, `:1958`, `:2029` (« transactions
  importées » : une transaction saisie compte de même), `:2097-…` (§ *Supprimer un import* : « le nom du fichier »,
  « le même fichier peut ensuite être réimporté ») et `:2462` (« puis le nom du fichier »), et
  `admin-manual.tex:82` (« Import bancaire CAMT.053 … et CSV »). *(Le rapport F citait aussi `:2363` : c'est le rejet
  d'une transaction, qui ne nomme pas le fichier — écarté après lecture.)* Le filet n'est plus un `grep` de phrases
  coupées par les retours à la ligne : le texte **aplati** (`tr '\n' ' ' | tr -s ' '`) du `.tex` et du PDF est cherché
  sur `CAMT`, `CSV`, `import`, `fichier`, `solde de clôture`, et chaque occurrence est triée au Dev Agent Record. La
  conduite « la banque passe au CAMT » (validation P1, R9, F13) décrit **trois** cas — référence saisie égale à celle du
  fichier (reconnue doublon, ignorée par défaut à l'import), différente (non reconnue : le mouvement entre deux fois),
  absente des deux côtés (CSV sans référence : même clé, reconnue doublon, ignorée par défaut) — et dit que la voie
  normale est d'importer **à partir du lendemain** de la dernière période saisie ; supprimer un relevé saisi déjà
  rapproché exige d'annuler d'abord chaque rapprochement (`\ref{sec:annuler-rapprochement}`). La FAQ gagne une entrée
  « Ma banque ne propose ni CAMT.053 ni CSV » qui renvoie à la sous-section.
  Les manuels DE/IT/EN ne sont pas traduits dans cette version : rien à y faire.
- **AC-28** — `docs/api-external.md` : les deux routes (tableau des ressources, `:223`), le corps, les codes nouveaux
  (tableau des codes d'erreur), la phrase « *(Depuis la v0.17.0.)* » ; et (validation P1, F11) : la liste des écritures
  refusées sur un compte clos (`:243`, « aperçu et création d'un import… ») gagne la saisie, `:866` qui y renvoie se
  relit ; la phrase du rejeu (`:878`, « L'import d'un relevé (`POST /bank-imports`) n'est pas rejoué ») couvre aussi la
  saisie ; les valeurs de `sourceFormat` (`CAMT053_V04`, `CAMT053_V08`, `CSV`, `MANUAL`) et la forme du `filename` d'un
  relevé saisi sont documentées à côté des routes de lecture.
- **AC-29** — **CHANGELOG et § de mise à jour, passés à la 0.17.0** (C-v016-2-15), dans le commit de la migration ou
  le suivant :
  1. l'entrée `## [0.16.1] — 2026-10-29` devient `## [0.17.0] — Non publié` (forme que `scripts/prepare-release.sh`
     date à la release — la lire, ne pas l'exécuter) ;
  2. elle gagne une section **`### Ajouté`** (en tête des sections) : la saisie d'un relevé à la main, en une entrée,
     `refs` [#816] par un lien, sans mot-clé de fermeture ; les sections *Modifié* et *Corrigé* (#811, #812) restent ;
  3. son en-tête se récrit : la phrase « Il n'ajoute **aucune migration** : la 0.16.0 redémarre sur une base passée par
     la 0.16.1, et une sauvegarde de l'une s'importe dans l'autre — le retour à la 0.16.0 ne demande aucune sauvegarde
     particulière […] » est **supprimée** et remplacée par : la 0.17.0 ajoute **une migration**
     (`20261030000001_bank_imports_manual_source`) qui relève `kesh_version_min_required` à **0.17.0** — la 0.16.x ne
     démarre plus sur une base passée par la 0.17.0, une sauvegarde de la 0.17.0 ne s'importe plus dans une 0.16.x, et
     le retour passe par la sauvegarde d'avant la mise à jour (base, `.keshbackup`, dossier des documents) ;
  4. une **note sur la candidate `0.16.1-rc1`** : publiée en pré-version le 2026-10-29 (`gcorbaz/kesh:0.16.1-rc1`,
     `:latest` non déplacé), non poursuivie ; son contenu (#811, #812) est livré par la 0.17.0 ; une installation passée
     par la rc1 se met à jour vers la 0.17.0 comme depuis la 0.16.0 ;
  5. au manuel d'administration, `\subsubsection{Passer à la 0.16.1}\label{sec:maj-0-16-1}` (`admin-manual.tex:2481`)
     devient *Passer à la 0.17.0* (`sec:maj-0-17-0`, chaque `\ref` suivi : `grep -rn "maj-0-16-1" docs`), et son texte
     dit la migration et la sauvegarde d'avant la mise à jour comme le point 3, au `.tex` — **sauvegarde complète
     (base, `.keshbackup`, dossier des documents) avant la mise à jour** ; le retour à la `0.16.1-rc1` (ou à la 0.16.0)
     ne passe **que** par elle ; une sauvegarde `.keshbackup` faite en 0.16.x s'importe dans la 0.17.0 (AC-3-bis), non
     l'inverse ; le PDF se régénère à la
     release (macros).
  6. **tri des `0.16.1` du texte qui décrit le contenu de la version** (validation P1, F2) : au CHANGELOG `[0.17.0]`,
     la phrase d'ouverture (« Correctif de l'Epic 32 … » : la version porte désormais une fonctionnalité), le renvoi
     « *Passer à la 0.16.1* » (l. 19) et « reste à confirmer avec Claude Code sur la 0.16.1 » (l. 15) ; au § renommé,
     `admin-manual.tex:2483` (« La 0.16.1 ne demande ni nouveau compose… ») et `:2489` (« la 0.16.1 n'en émettant
     plus ») ; `user-manual.tex:3265` (« à confirmer avec Claude Code sur la 0.16.1 ») ; `README.md:49` (« corrigée côté
     Kesh en 0.16.1 ») ;
  7. `admin-manual.tex:2270` (§ *Sauvegarde et restauration*) : « ce seuil n'est relevé que par les migrations qui
     changent le schéma de façon incompatible » devient faux (la migration de la 0.17.0 relève le seuil sans changer le
     schéma) — récrire : « … par les migrations qui changent le schéma de façon incompatible, ou dont les données ne se
     liraient plus dans une version antérieure ».
  Le développeur ne touche **ni** les macros `\keshVersion`…, **ni** les URL `raw.githubusercontent.com/…/v0.16.1`,
  **ni** la ligne de feuille de route du README, **ni** le site : ils suivent le bump de `\keshVersion` (garde G19) au
  **tag final** (§ *Ce que la release doit contenir*, B).
- **AC-30** — README, section *Fonctionnalités*, ligne *Import bancaire* (`README.md:41`) : « **saisie manuelle d'un
  relevé** quand la banque n'en fournit pas de fichier ✓ ». La feuille de route ne change pas (une ligne de version
  est l'affaire de la release).

## Tâches

1. **Étape 0** — recontrôler au code chaque ligne du § *À recontrôler à l'étape 0* et le noter au Change Log, ligne
   lue citée. Créer la base `kesh_v0162` (et `kesh_e2e_v0162`) selon `docs/agents-orchestration.md` § 4.
2. **Tests rouges d'abord** (§ *Tests et mutations*), commités rouges avec un message qui le dit.
3. `kesh-core` : variante `Manual` ; module `manual` (AC-4, AC-5).
4. `kesh-db` : variante `Manual` (AC-1) ; migration **et**, dans le même commit, les dix crates en `0.17.0` (P2-bis),
   audit, P6, P7, squash, tri des occurrences de `0.16.1` (AC-2) ; deux lectures (AC-6, AC-7) ; CHANGELOG `[0.17.0]` et
   § *Passer à la 0.17.0* (AC-29).
5. `kesh-api` : erreurs (`errors.rs` : variantes, codes, statuts, clés i18n), module de routes, montage, audit,
   registres (AC-9 à AC-15) ; tests HTTP (AC-10 à AC-18).
6. `frontend` : types, `manual-statement.ts`, généralisation de `isValidAmount`, `ManualStatementForm.svelte`, page,
   lien, libellés de liste/détail/suppression, i18n (AC-19 à AC-24) ; Vitest.
7. E2E Playwright (AC-26) ; captures (AC-25).
8. Textes (AC-27 à AC-30) ; PDF du manuel utilisateur régénéré (`make fr` dans `docs/manual/`, sous verrou et
   `mem-guard`) ; **greper la valeur** de chaque phrase corrigée (§ *Propagation*).
9. Gates (§ *Gates*), mutations, Dev Agent Record.

## Tests et mutations prévus

**Rouges d'abord** : les tests de `kesh-core` (AC-4, AC-5), de dépôt (AC-6, AC-7), le test e2e HTTP de création
nominale (AC-10) et de déséquilibre (AC-11), et le Vitest de `manual-statement.ts` (AC-21) s'écrivent et rougissent
avant le code (compilation incluse : un test qui ne compile pas parce que le type n'existe pas compte comme rouge, à
condition que l'erreur nomme ce type).

Fichiers prévus : `crates/kesh-core/src/bank_imports.rs` (ou `bank_imports/manual.rs`) `mod tests` ;
`crates/kesh-db/src/repositories/bank_imports.rs` (tests `#[sqlx::test]`, squash) ; `crates/kesh-api/tests/bank_statements_manual_e2e.rs`
(nouveau) ; `crates/kesh-db/tests/migrations_*.rs` (AC-2) ; `frontend/src/lib/features/bank-import/manual-statement.test.ts`,
`ManualStatementForm.test.ts`, les tests existants de `BankImportList`, `DeleteBankImportAction`, de la page détail, et
`journal-entries/balance.test.ts` (la généralisation ne change pas le défaut) ; `frontend/tests/e2e/bank-manual-statement.spec.ts`.

| # | Mutation (une à la fois, motif vérifié unique, restauration `git checkout -- f && touch f`) | Doit rougir |
|---|---|---|
| M1 | équilibre : `!=` remplacé par une tolérance `abs() > 0.01` | AC-4 r. 6 (écart d'un centime refusé) |
| M2 | sens : `debit` enregistré positif | AC-10 (montants signés en base) |
| M3 | règle 3 retirée (date hors période admise) | AC-4 r. 3, AC-11 |
| M4 | règle 5 : `normalize()` retiré avant le contrôle d'échelle | AC-4 r. 5 (`"50.00"` admis) |
| M5 | doublons de lignes : sans confirmation, filtrés au lieu de refusés | AC-11 (refus, rien d'écrit) |
| M6 | `confirmOpeningBalanceGap` ignoré (toujours vrai) | AC-11 (`BANK_STATEMENT_OPENING_BALANCE_GAP`) |
| M7 | `previous_statement_closing` : `period_to < ?` → `<= ?` | AC-6 (relevé qui finit le jour même exclu) |
| M8 | `previous_statement_closing` : `ORDER BY period_to ASC` | AC-6 (ordre) |
| M9 | empreinte canonique : le sens d'une ligne omis | AC-5 (sensibilité au sens) |
| M10 | `source_format` écrit `Csv` au lieu de `Manual` | AC-10 |
| M11 | action d'audit `bank_import.created` au lieu de `entered_manually` | AC-10, AC-15 |
| M12 | aperçu : la trace d'audit écrite | AC-12 |
| M13 | `manual-statement.ts` : les débits ajoutés au lieu d'être retranchés | Vitest AC-21 |
| M14 | *Enregistrer* actif quand l'écart est non nul | Vitest AC-21 |
| M15 | aperçu non périmé par une modification du formulaire | Vitest AC-22 |
| M16 | migration : `'0.17.0'` → `'0.16.1'` | AC-2 (test `DowngradeRefused` sur `0.16.1` et `min_required`) |
| M17 | migration : ajouter `UPDATE bank_imports SET statement_id = NULL;` avant le relèvement | AC-2-bis (relecture à l'identique ; inventaire des instructions) |
| M18 | `BankImportSourceFormat::from_str` : arme `"CSV"` retirée | AC-2-bis point 2, AC-3-bis (relecture applicative) |
| M19 | formulaire : les cases cochées non transmises dans le corps | Vitest AC-22 (corps envoyé) |
| M20 | `POST /bank-imports/manual` monté hors du groupe Comptable+ | AC-13 (Consultation → `403`) |
| M21 | `confirmDuplicateFile` ignoré (toujours vrai) | AC-11 (`BANK_IMPORT_DUPLICATE_FILE`) |
| M22 | « un échec de la trace n'écrit rien » : `tx.commit()` avant l'écriture de la trace | AC-10 (test qui fait échouer la trace — p. ex. `user_id` inexistant par un acteur de test — et constate qu'aucun entête n'est resté) |

Chaque mutation : le test qui rougit et la raison, au Dev Agent Record. Une mutation **survivante** est un défaut de test
à corriger, non à déclarer.

## Gates

`kesh-db` est touché (migration, dépôt) : **gate complet même en cours de boucle** (`CLAUDE.md` § *Exception kesh-db*).
Base de gate remise à zéro avant chaque gate complet. Au **dernier commit de code** : `scripts/test-fast.sh --ci` sous
`flock "$GATE_LOGS/.gate.lock"`, `cargo build --workspace --all-targets`, Vitest complet, `npm run check`,
`npm run lint-i18n-ownership`, `npm run build`, et la **suite E2E complète** selon la recette de `docs/testing.md` à la
lettre (dossiers propres au worktree passés au backend **et** au runner) — échecs jugés fichier par fichier contre
`docs/testing.md` § *Les échecs attendus*. La migration rend aussi nécessaire un démarrage réel contre une base
**persistante** (`kesh_e2e_v0162` migrée par la branche) : c'est la suite E2E qui le fait (P8). Ne déclarer que ce qui a
tourné, chiffres recomptés.

## Ce que les gardes ne voient pas

- **Saisi puis importé** : l'import d'un fichier qui couvre une période déjà saisie à la main n'avertit pas de la
  période ; ligne par ligne, l'empreinte reconnaît le doublon si la référence est égale des deux côtés ou absente des
  deux côtés (il est alors **ignoré par défaut** à l'import), et ne le reconnaît pas si les références diffèrent ou si
  une seule en porte une — le mouvement entre alors deux fois (C-v016-2-6 ; trois cas corrigés à la validation P1, R9).
  Écrit au manuel ; avertissement à l'import demandé en issue.
- **Le relevé voisin** : le solde de départ n'est comparé qu'au relevé **précédent** à solde, dont la période finit
  **strictement** avant la nouvelle (`period_to < du`) ; deux relevés successifs qui partagent leur jour-frontière ne
  se voient pas, et un mois saisi entre deux relevés existants n'est jamais confronté au solde d'ouverture du
  **suivant** (validation P1, R14).
- **Doublons dans un même relevé** : deux lignes identiques d'un même relevé saisi ne sont pas signalées (limitation
  L15 de l'import, `kesh-core/src/bank_imports.rs`, « doublons intra-import non détectés ») — légitime pour deux retraits
  du même montant le même jour.
- **L'heure du serveur** : aucun contrôle de date future ; une année tapée de travers (2062) passe si l'équilibre tient.
  La tuile et K3 la montreraient. Écarté délibérément (C-v016-2-3).
- *(Première rédaction : « Un binaire 0.16.1-rc1 accepte une sauvegarde de la rc2 ». Levé par l'amendement
  C-v016-2-15 : avec `min_required = 0.17.0`, la rc1, de version Cargo `0.16.1`, refuse la base et les sauvegardes de la
  0.17.0.)*

## Ce que la release doit contenir (orchestrateur ; ne pas prescrire au développeur)

*Amendé le 2026-10-30 (C-v016-2-15) : la release est la **0.17.0**, première candidate `v0.17.0-rc1`. Le CHANGELOG
`[0.17.0]` et le § *Passer à la 0.17.0* sont posés par le développeur (AC-29) ; la release les relit.*

*Remédié à la validation P1 (F3) : la liste distingue ce qui précède la candidate de ce qui attend le tag final.
Constat, `docs/ci.md:165-169` : un tag `vX.Y.Z-rcN` ne pousse que `:X.Y.Z-rcN` (`:latest` ne bouge pas) et « les
candidates servent à jouer une recette sur l'image publiée avant de figer les textes du tag final » ;
`kesh-style.sty:72-74` porte `\keshVersion{0.16.1}`, `\keshReleaseDate{2026-10-29}` (date de la `0.16.1-rc1`),
`\keshTargetRelease{v0.16}` ; `git tag -l 'v0.16*'` rend `v0.16.0` et `v0.16.1-rc1` — le tag `v0.16.1` n'existe pas et
n'existera pas.*

**A. Avant `v0.17.0-rc1`**

1. **CHANGELOG `[0.17.0] — Non publié`** : relire ce que pose AC-29 (sections *Ajouté* #816, *Modifié*, *Corrigé* #811
   et #812 ; la phrase « aucune migration » supprimée ; la migration et le retour par la sauvegarde ; la note sur la
   candidate `0.16.1-rc1`). **Ne pas lancer `scripts/prepare-release.sh` pour la candidate** : il daterait l'entrée
   (« ne date pas deux fois ») et figerait la date de la candidate dans celle de la version finale. Si Guy retient Q1
   (b), la migration et ses phrases tombent — à arbitrer avant le développement.
2. **Manuel d'administration**, § *Passer à la 0.17.0* et § *Sauvegarde et restauration* (`:2270`) : relus au `.tex`.
3. **Macros, URL et PDF : inchangés pour la candidate** (décision de l'orchestrateur, comme pour la `0.16.1-rc1`) :
   `\keshVersion{0.16.1}` et les URL `raw.githubusercontent.com/…/v0.16.1` restent jusqu'au tag final, et la garde G19
   reste verte. ⚠️ Conséquence écrite : pendant la candidate, ces URL visent un tag `v0.16.1` qui n'existe pas (404), et
   la page de titre des manuels dit 0.16.1 alors qu'ils décrivent la 0.17.0 — à dire dans les notes de la candidate
   (point 4), avec le lien vers les fichiers du tag `v0.17.0-rc1`.
4. **Notes de pré-version de `v0.17.0-rc1`** : **avant la mise à jour, une sauvegarde complète** (base, `.keshbackup`,
   dossier des documents) de l'installation `0.16.1-rc1` ; le retour à la rc1 ne passe que par elle (`min_required`
   0.17.0) ; une sauvegarde `.keshbackup` de la 0.16.x s'importe dans la 0.17.0 ; la migration ; l'image à tirer :
   `gcorbaz/kesh:0.17.0-rc1` (`:latest` ne bouge pas : une installation qui suit `:latest` reste en 0.16.0) ; la
   `0.16.1-rc1` non poursuivie (une installation passée par elle se met à jour normalement, puis la rc1 refuse la base,
   sa version Cargo étant `0.16.1`) ; la recette Claude Code et les tests joués une seule fois, sur cette candidate ; les
   manuels encore titrés 0.16.1 (point 3).
5. **Rien de jetable sur la base du NAS** (C-v016-2-16) : la recette Claude Code, toute écriture de démonstration
   (`4321.09` compris), les clés de recette et les captures se jouent sur une installation **jetable** ; la base de
   Guy ne reçoit que la mise à jour elle-même, après sa sauvegarde. Aucune procédure de cette fiche n'écrit dans la base
   du NAS (les captures d'AC-25 se prennent sur une base de démonstration, l'E2E sur `kesh_e2e_v0162`).

**B. Au tag final `v0.17.0`**

6. `scripts/prepare-release.sh` (pré-vol, exemptions datées ; il trouve les crates déjà en `0.17.0`, saute le bump et
   date l'entrée du CHANGELOG du jour de la publication finale).
7. Macros `\keshVersion{0.17.0}`, `\keshReleaseDate{<date de publication Docker Hub du final>}`,
   `\keshTargetRelease{v0.17}`, URL `raw.githubusercontent.com/…/v0.16.1` portées à `v0.17.0` (garde G19) ; PDF des
   trois manuels régénérés.
8. README : la ligne `v0.16.1` de la feuille de route devient la ligne `v0.17.0` (saisie manuelle, correctif MCP,
   migration) ; site `website/` : `index.html` (`:70` porte `0.16.1`), `roadmap.html`, **`issues.html`** (liste des KF
   et de la feuille de route connue, liste de contrôle pré-release du `CLAUDE.md`, point 3) ; brochure
   (`marketing-brochure.tex:165`, `:388`, « Import natif des extraits CAMT.053 … et CSV ») si le positionnement le
   demande (F14 c).

**C. Communs**

9. PR de release : `closes #816` dans le **titre ou le corps**, à côté de `closes #811` et `closes #812`.
10. Décompte des issues au tag (objectif, non barrière) ; l'issue demandée ci-dessous comprise.
11. Issue à ouvrir (demandée par la fiche) : **« L'import d'un fichier ne signale pas une période déjà saisie à la main »**
   — enhancement ; C-v016-2-6.

## Dev Notes

### Fichiers touchés (prévus) et modules

| Module | Fichiers |
|---|---|
| `kesh-core` | `src/bank_imports.rs` (variante `Manual`, module `manual` ou sous-fichier) |
| `kesh-db` | `src/entities/bank_import.rs`, `src/repositories/bank_imports.rs`, `migrations/20261030000001_bank_imports_manual_source.sql`, `src/post_restore.rs`, `test-schema/0001_schema_squash.sql` (régénéré), `tests/migrations_fresh_install.rs`, `tests/migrations_upgrade_path.rs`, `docs/migrations-idempotence-audit.md` |
| `kesh-api` | `src/routes/bank_statements_manual.rs` (nouveau), `src/routes/mod.rs`, `src/routes/bank_imports.rs` (visibilité, action d'audit en paramètre, arme `Manual` de `source_format_tag_for_kesh_core`), `src/lib.rs`, `src/errors.rs`, `src/audit_labels.rs`, `tests/bank_statements_manual_e2e.rs` (nouveau), `tests/audit_route_registry.rs`, `tests/audit_label_registry.rs` |
| `kesh-i18n` | `locales/{fr,de,it,en}-CH/messages.ftl` |
| `frontend` | `src/lib/features/bank-import/{ManualStatementForm.svelte, manual-statement.ts, *.test.ts, bank-import.api.ts, bank-import.types.ts, BankImportList.svelte, DeleteBankImportAction.svelte}`, `src/lib/features/journal-entries/balance.ts` (généralisation), `src/routes/(app)/bank-import/+page.svelte`, `src/routes/(app)/bank-import/manual/+page.svelte` (nouveau), `src/routes/(app)/bank-import/[id]/+page.svelte`, `tests/e2e/bank-manual-statement.spec.ts` |
| docs | `docs/manual/fr/user-manual.tex` (+ PDF), `docs/manual/fr/admin-manual.tex` (`:82`, `:2270`, `:2481-2495`, `:2966`), `docs/api-external.md`, `CHANGELOG.md`, `README.md` (`:41`, `:49`) |
| gardes touchées | `crates/kesh-db/src/post_restore.rs` (compteur 21 → 22), `crates/kesh-api/tests/audit_route_registry.rs` (compteurs), `crates/kesh-api/tests/audit_label_registry.rs` (`SITES_INDIRECTS`), `crates/kesh-db/tests/bank_accounts_repository.rs` (AC-14), `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) |

Six modules au grain des exemples de la règle de splitting (voir § *Dérogation règle de splitting*), un seul domaine. Le symétrique existe : tout ce que l'import fait déjà se réutilise
(DRY) ; ce qui est neuf est la validation de saisie, l'empreinte canonique, les deux lectures et l'écran.

### Dérogation règle de splitting

*(Écrite à la validation P1, R5 ; décision de l'orchestrateur : pas de découpage.)*

- **Décompte honnête.** Au grain « crates et paquet npm », cinq modules : `kesh-core`, `kesh-db`, `kesh-api`,
  `kesh-i18n`, `frontend`. Au grain des exemples de la règle (`kesh-core/accounting`, `kesh-api/routes/invoices`,
  `frontend/src/features/invoices`), **six** : `kesh-core/bank_imports`, `kesh-db` (entité, dépôt `bank_imports`,
  migration, `post_restore`), `kesh-api/routes/bank_imports` (et le module voisin `bank_statements_manual`, `errors`,
  `audit_labels`, `lib`), `kesh-i18n/locales`, `frontend/src/lib/features/bank-import` (avec sa page), et
  `frontend/src/lib/features/journal-entries` (la seule généralisation d'`isValidAmount`). Le seuil « plus de cinq »
  est donc franchi.
- **Justification.** Un seul domaine métier, l'import bancaire ; **un seul chemin d'écriture**, réutilisé tel quel
  (`create_with_transactions` ; aucun SQL d'écriture nouveau, AC-8) ; le sixième module ne reçoit qu'un paramètre de
  plus sur une fonction pure, défauts inchangés. Un découpage backend / écran ne livrerait rien d'utilisable seul et
  doublerait le cycle de revue d'une candidate attendue.
- **Risque accepté.** Une revue adversariale tient un périmètre plus large qu'une story au seuil : des défauts de
  remédiation peuvent échapper aux passes tardives. Atténuation : passes complètes Sonnet ↔ Opus, passe ciblée sur
  chaque remédiation, gate complet (exception `kesh-db`).
- **Ligne de découpe de repli** (si la sévérité stagne sur des défauts recyclés, ou si la boucle approche le plafond) :
  `v016-2a` — `kesh-core`, `kesh-db`, `kesh-api`, catalogues des codes d'erreur et du libellé d'audit, tests HTTP ;
  `v016-2b` — écran, généralisation d'`isValidAmount`, manuels, E2E, captures.

### Le manuel (axe de revue)

Le PDF se contrôle aplati (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`). La revue vérifie
que chaque promesse de la sous-section nouvelle est tenue par le code (pas d'« ignorer » ; équilibre exact ; confirmation
du solde de départ ; suppression puis ressaisie) et qu'aucune phrase ancienne ne dit encore « seul le CAMT.053 porte un
solde ».

### Propagation

Après chaque correctif de texte, greper la **valeur** : `deux formats`, `Seul le CAMT`, `Seuls les relevés`,
`CSV n'en a`, `aucune migration` (pour le signaler à la release, non pour le corriger), `bank_import.created` (les sites
qui le comptent comme seule trace de création d'un import : vues, tests, manuels).

### Références

- `CLAUDE.md` : *Migration breaking policy* (P1–P8), *Un appariement automatique propose, il ne crée jamais*,
  *Pattern batch* (sans objet : création unitaire, pas de `{ accepted, failed }`), *Règle de splitting préventif*,
  *Recette visuelle*, *Test Locally First*.
- `docs/agents-orchestration.md` § 2 à § 7.
- Stories : 8-1b, 8-3 (doublons), 27-1 (suppression d'un import), 27-3 (empreinte d'aperçu), 28-4a/29-4a (compte clos),
  25-6-a (tuile), 29-1a (K3).

## À recontrôler à l'étape 0 du développement

- `routes/bank_imports.rs:83-98`, `:389-398`, `:1154-1180`, `:1259-1278`, `:1484-1576`, `:1698-1750`, `:978-984` — ce
  que la fiche leur fait dire.
- `bank_accounts.rs:1798-1804` (ordre du relevé de référence) et `closing_checks.rs:455-470` (critère « à solde »).
- `entities/bank_import.rs:53-60` — le refus des valeurs inconnues (fonde C-v016-2-2).
- `routes/admin.rs:217-223` — le refus d'une sauvegarde de `min_required` supérieur.
- La forme d'inscription à `SITES_INDIRECTS` (`tests/audit_label_registry.rs`) et au registre des routes.
- Que `GET /companies/current` ne rend que les comptes ni archivés ni clos (`docs/api-external.md:240`).
- Le patron de garde des écrans Comptable+ (redirection d'un Consultation) à reprendre pour `/bank-import/manual`.

## Questions au Project Lead

Tranchées au registre ; listées ici pour être soumises à Guy **avant** la validation.

- **Q1 (C-v016-2-2, C-v016-2-15)** — Une migration qui relève `kesh_version_min_required` à 0.17.0 : la 0.16.x
  (candidate `0.16.1-rc1` comprise) refuse alors la base et les sauvegardes de la 0.17.0. Sans elle (option b), la
  0.16.0 redémarrerait mais rendrait une erreur sur les écrans d'import dès qu'un relevé saisi existe, et une sauvegarde
  de la 0.17.0 s'y importerait sans s'y lire. Retenu :
  la migration. D'accord, sachant que l'en-tête du CHANGELOG de la rc1 (« aucune migration ») change ?
- **Q2 (C-v016-2-7)** — Corriger une faute (date, libellé) après l'enregistrement : retenu, supprimer le relevé et le
  ressaisir. Faut-il, dès cette version, un bouton « Corriger » qui rouvre le relevé prérempli et le **remplace** en une
  fois ?
- **Q3 (C-v016-2-5)** — Un solde de départ différent de la clôture du relevé précédent : retenu, signalé et à confirmer.
  Préférez-vous qu'il bloque ?
- **Q4 (C-v016-2-3)** — Deux colonnes Débit / Crédit comme sur le papier (plutôt qu'un montant et un sens), et une
  colonne Contrepartie facultative (elle aide les propositions et les règles d'affectation) : cela vous convient-il ?
- **Q5 (C-v016-2-9)** — Une clé d'API `read-write` peut saisir un relevé, comme elle peut importer un fichier. À fermer ?
- **Q6 (C-v016-2-6)** — Si cette banque passe un jour au CAMT, l'import d'une période déjà saisie ne le signalera pas :
  la fiche demande une issue pour un avertissement à l'import. D'accord pour l'ouvrir (elle fera monter le décompte) ?
- **Q7 (C-v016-2-4)** — *(ajoutée à la validation P1, F12/R15 ; soumise à Guy, réponse attendue)* L'équilibre est
  **exact, bloquant, sans confirmation** : un relevé papier dont les soldes imprimés ne se retrouvent pas (page
  manquante, ligne illisible, report) ne s'enregistre pas tant que la ligne manquante n'est pas saisie. Recommandation
  retenue en attendant : garder ce choix (une confirmation « malgré l'écart » enregistrerait un relevé faux que la tuile
  et K3 compareraient ensuite aux livres). Si Guy tranche autrement, un amendement suivra.
- **Q8 (C-v016-2-3)** — *(ajoutée à la validation P1, F12 ; soumise à Guy, réponse attendue)* Devise : **CHF seule**,
  comme l'import ; si le compte sans relevé électronique est en devise étrangère, la saisie ne lui sert pas. Les comptes
  en devise relèvent de **#233**. Recommandation retenue en attendant : CHF seule. Si Guy tranche autrement, un
  amendement suivra.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-30 — Fiche créée (Opus 5.5, autonomie), registre C-v016-2-1 à 14, prompt de validation P1
  (`v016-2-validate-prompt-p1.md`). Prochaine étape : questions Q1–Q6 à Guy, puis validation P1 (Sonnet, lentilles R
  et F).
- 2026-10-30 — **Amendement (C-v016-2-15)** : la release devient la **0.17.0** (décision du Project Lead), première
  candidate `v0.17.0-rc1`. La migration relève `min_required` à `0.17.0` ; les dix crates passent à `0.17.0` dans le
  même commit (P2-bis) ; le tri des occurrences de `0.16.1` est prescrit (AC-2) ; le CHANGELOG `[0.16.1]` devient
  `[0.17.0]` et le § *Passer à la 0.16.1* devient *Passer à la 0.17.0*, posés par le développeur (AC-29) ; § *Ce que la
  release doit contenir*, M16, Q1 et la limite de la rc1 mis à jour. Occurrences de `0.16.1` laissées à dessein :
  l'historique (`c5101b07`, candidate `v0.16.1-rc1`, recette de la rc1, première rédaction de l'origine), les versions
  sources d'un remplacement prescrit, et `0.16.1` comme version antérieure refusée.
- 2026-10-30 — **Contrainte du Project Lead (C-v016-2-16)** : les données de l'installation `v0.16.1-rc1` du NAS se
  gardent. Ajoutés : AC-2-bis (montée depuis une base 0.16.x peuplée, imports CAMT et CSV, transactions et rapprochements
  relus à l'identique, en base et par le code ; la migration ne porte que le relèvement), AC-3-bis (une sauvegarde
  `.keshbackup` 0.16.x s'importe dans la 0.17.0, aucune table ajoutée), M17 et M18 ; AC-29 et les notes de la candidate
  disent la sauvegarde complète d'avant la mise à jour ; point 5-bis de la release : rien de jetable sur la base du NAS.
- 2026-10-30 — **Validation P1** (Sonnet, lentilles R et F, contexte frais ; rapports
  `kesh-gate-logs/v0162-validate-p1-{R,F}.md`) : **0 CRITICAL, 0 HIGH, 12 MEDIUM, 17 LOW** (R : 5 MEDIUM, 10 LOW ;
  F : 7 MEDIUM, 7 LOW ; recoupements R3 = F4, R10 ≈ F5, R15 ≈ F12). Prémisses vérifiées au code avant chaque remède
  (lignes lues : `audit_label_registry.rs:105-116`, `:525-545` ; `audit_route_registry.rs:909`, `:915`, `:939`, `:944`,
  `:1324` ; `admin-manual.tex:2966`, `:2270`, `:2483`, `:2489`, `:82` ; `post_restore.rs:1214-1220` ; `balance.ts:23-35` ;
  `routes/bank_accounts.rs:176-187` ; `bank_accounts_repository.rs:2252`, `:2481` ; `user-manual.tex:49`, `:813-814`,
  `:2046-2052`, `:2097`, `:2126`, `:2216`, `:1925`, `:1958`, `:2029`, `:2462`, `:3265` ; `docs/ci.md:160-169` ;
  `kesh-style.sty:72-74` ; `textes_coherents.rs:1305-1350` ; `git tag -l 'v0.16*'`). Traitement :
  - **MEDIUM** — R1 appliqué (AC-15 : `ImportAuditAction` défini dans `bank_imports.rs`, entrée `SITES_INDIRECTS`) ;
    R2 appliqué (AC-15 : compteurs du registre des routes, phrase `admin-manual.tex:2966` ; fichiers touchés) ; R3 = F4
    appliqué (AC-21 : options de `isValidAmount`, solde vide, virgule) ; R4 appliqué (AC-4 r. 5 : `"50.000"`) ; R5 :
    pas de découpage (décision de l'orchestrateur), § *Dérogation règle de splitting* écrite (six modules) ; F1
    appliqué (AC-27 : inventaire complété, `:803` corrigé en `:813` ; `:2363` écarté après lecture, il ne nomme pas le
    fichier) ; F2 appliqué (AC-29 points 6 et 7) ; F3 appliqué (§ *Ce que la release doit contenir* en A / B / C ;
    macros et URL à 0.16.1 jusqu'au tag final, sur décision de l'orchestrateur, vérifié à `docs/ci.md` et à G19 ;
    `website/issues.html` ajouté) ; F5 (≈ R10) appliqué (AC-20 : correspondance des rangs) ; F7 appliqué (ensembles
    fermés de `field` et `reason`, table de libellés) ; F12 (≈ R15) : choix gardés, Q7 et Q8 ajoutées avec leur
    recommandation (décision de l'orchestrateur, questions soumises à Guy).
  - **LOW, tous appliqués** (remède mécanique) — R6 (compteur P7 21 → 22), R7 (255 caractères, non octets), R8 (encodage
    par longueur + test d'injectivité), R9 (trois cas « saisi puis importé »), R10 (avec F5), R11 (AC-26 : dates,
    compte propre à l'exécution), R12 (noms JSON d'AC-18 ; `statement_id` et champs non nuls semés en AC-2-bis ; AC-14
    au niveau dépôt), R13 (M19 à M22), R14 (limite du relevé voisin), R15 (avec F12), F6 (liste fermée des causes
    d'inactivité, cases), F8 (gardes i18n nommées), F9 (inventaire des chaînes de suppression et du détail), F10
    (captures), F11 (`api-external.md` `:243`, `:866`, `:878`, `sourceFormat`), F13 (voie normale, annulation d'abord),
    F14 (FAQ ; `README.md:49` à AC-29 ; brochure à la release).
  - **LOW non pris** : aucun.
  Choix nouveaux au registre : C-v016-2-17. Prochaine étape : validation P2 (`v016-2-validate-prompt-p2.md`, Opus,
  passe complète, lentilles R et F).
