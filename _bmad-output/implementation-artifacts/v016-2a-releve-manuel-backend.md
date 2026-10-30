# Story v016-2a : saisir un relevé bancaire à la main — le backend

Status: ready-for-dev

*Sous-fiche issue du découpage de la Story v016-2 à la validation P2 (2026-10-30, décision de l'orchestrateur,
C-v016-2-18). Worktree `/home/gcorbaz/devel/kesh-0162`, branche `story/v016-2-releve-manuel` (partie de `c5101b07`), la 2a d'abord,
puis la 2b. Registre **commun** : `v016-2-choix-autonomes.md` (C-v016-2-1 à C-v016-2-19). Issue : **#816**, en `refs`
seulement ; la fermeture se porte sur la PR de release. Les numéros de ligne cités sont ceux de `c5101b07`.
**Contexte, décisions, contrat de l'API (corps, réponse de l'aperçu, codes d'erreur, ensembles fermés, ordre des
refus), limites, liste de la release et questions au Project Lead : fiche parente `v016-2-releve-manuel.md`** — ils
valent ici sans être recopiés. Les AC gardent leur numéro d'origine (table de correspondance de la parente).*

## Story

En tant que Guy, je veux que Kesh **accepte et contrôle un relevé saisi à la main** par son API — et que la 0.17.0 monte
sans perte une base 0.16.x peuplée —, afin que l'écran de la `v016-2b` n'ait plus qu'à le présenter.

## Périmètre

`kesh-core` (validation, empreinte), `kesh-db` (variante `Manual`, la migration, le bump `0.17.0` des dix crates par
P2-bis, deux lectures, la montée sans perte AC-2-bis, la sauvegarde 0.16.x AC-3-bis), `kesh-api` (routes, erreurs et
leurs messages aux quatre catalogues, audit et son libellé, registres), `docs/api-external.md`, les tests Rust, et les
textes liés à la migration et à la version : CHANGELOG `[0.17.0]` (hors l'entrée *Ajouté*), manuel d'administration
(§ *Passer à la 0.17.0*, `:2270`, `:2966`, les `:3220`, `:3238`, `:3349`, `:3351` ; sa ligne `:82` relève de la 2b, avec
l'inventaire de l'AC-27),
les `0.16.1` du tableau des propriétaires, `README.md:49` et `:235`, `user-manual.tex:3265`, et les PDF régénérés au
commit de leur `.tex`. **Livrable seul** : l'API se teste de bout en bout en HTTP (AC-10 à AC-18) sans écran.

## Critères d'acceptation

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
  passent par `env!("CARGO_PKG_VERSION")`). Les occurrences des manuels, du README, du site et des macros suivent le **tableau des
  propriétaires** de l'AC-29 point 6 (validation P2, F2). Le tri, occurrence par
  occurrence, au Dev Agent Record. P5 : une ligne à `docs/migrations-idempotence-audit.md` (verdict, justification) et les
  **deux** sites du total recomptés (`ls crates/kesh-db/migrations/*.sql | wc -l` → 87 ;
  `grep -c '^| \`20' docs/migrations-idempotence-audit.md` → 87), les trois compteurs de partition recomptés depuis le
  tableau. P6 : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` (46 occurrences à `c5101b07`), chaque site
  inspecté, sa conclusion au Dev Agent Record. P7 : entrée `EXEMPT_MIGRATIONS` (`crates/kesh-db/src/post_restore.rs`,
  `ExemptionBasis::Durable`, justification « UPDATE sur _kesh_version (bump kesh_version_min_required), table système
  jamais restaurée. La migration ne porte aucune autre instruction. »), et le compteur figé de
  `post_restore.rs:1217-1219` (`assert_eq!(EXEMPT_MIGRATIONS.len(), 21, …)` ; la ligne d'historique se pose au-dessus,
  avec celles des l. 1214-1216 — citation corrigée à la validation P3, F2) passe à **22**, avec sa ligne
  d'historique (« 21 → 22 (Story v016-2) »). Squash régénéré par
  `scripts/regen-test-schema.sh` (sa ligne `_kesh_version` passe à `'0.17.0'`), `test_schema_guard` vert. La migration s'inscrit au registre des checksums
  `crates/kesh-db/migrations.sha384` (`<version> <sha384>`, le geste qui la déclare publiée,
  `test_schema_guard.rs:690` ; validation P2, R7). Les tests qui
  figent le dernier bump (`migrations_fresh_install.rs:230-247`, `migrations_upgrade_path.rs:537-569`, et ce que
  `grep -rn "0\.14\.0" crates --include=*.rs` rend d'autre) passent à `0.17.0`, avec leur commentaire, **et leur sémantique se relit**, non la seule valeur (validation
  P2, R10) : `migrations_upgrade_path.rs:568-570` vérifie `BinaryAhead` avec un binaire `0.15.0`, qui deviendrait un
  refus — son binaire passe au-delà de `0.17.0` (`0.18.0`) ; un test
  `check_downgrade_protection(&pool, "0.16.1")` → `DowngradeRefused` (la dernière version antérieure, candidate
  comprise : la `v0.16.1-rc1` porte la version Cargo `0.16.1`). **Sites nommés à la validation P3 (R4)** :
  `migrations_upgrade_path.rs:122-125` (`assert_eq!(total, 86, …)` → 87, avec sa ligne d'historique) **et** `:213`
  (`total - 52` → `total - 53`, du même pas, la frontière 34 inchangée — le piège que rappelle le message d'assertion
  d'`apply_migrations_up_to`, `:42-50`, *et non « l'en-tête du fichier » : corrigé à la validation P4, P2*) ; **toutes les
  copies de ces valeurs** se trouvent par `grep -nE '\b(52|53|86|87)\b|total - 5[23]'
  crates/kesh-db/tests/migrations_upgrade_path.rs` (à `c5101b07` : `:44`, `:46`, `:63-64`, `:121`, `:124-125`, `:128`,
  `:136`, `:143`, `:146`, `:206`, `:213`, `:216`), chaque occurrence triée au Dev Agent Record, avec la ligne
  d'historique « Story v016-2a : 52 → 53, frontière inchangée (87 - 53 = 34) » sous `:206` ; `:541` et `:545` (version
  **du binaire** `"0.14.0"`, cas `Aligned`), `:557`, `:573` (`db_min` `"0.14.0"`) et `:656` (`assert_eq!(min_required, "0.14.0")`, dans un test qui
  rejoue toutes les migrations) ; le nouveau `bank_statements_manual_e2e.rs` monte le squash
  (`#[sqlx::test(migrations = "../kesh-db/test-schema")]`), sans quoi `test_schema_guard.rs`
  (`every_sqlx_test_attribute_is_accounted_for`) rougit.
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
     comparaison avant/après, `version` et `updated_at` compris pour `bank_transactions` — `bank_imports` n'a ni l'un ni
     l'autre (validation P3, R7) : la migration ne touche aucune ligne) ;
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
  manifeste comme l'écrirait une 0.16.x — praticable : le manifeste n'est pas signé (`manifest.rs:33-47`), seules les
  tables portent un `sha256` ; patron `recrire_table`, `unzip`, `rezip_tout` (`admin_full_import_e2e.rs:3684`) ;
  (b) sinon, une fixture `.keshbackup` **synthétique** produite par un générateur `#[ignore]` joué dans un worktree au
  tag `v0.16.1-rc1`, patron `generate_fixture_0_14_0` (`admin_full_export_e2e.rs:536`, Story 31-1a), versée sous
  `crates/kesh-api/tests/fixtures/` avec sa recette en commentaire — jamais l'installation du NAS, jamais de données
  réelles (patrons cités à la validation P2, R12). Le sens inverse reste refusé :
  une sauvegarde de la 0.17.0 dans un binaire `0.16.1` → `409` (`ImportVersionIncompatible`), testé par
  `check_import_version_compat("0.17.0", "0.16.1")`.
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
     plus 255 **caractères** — toutes les longueurs se comptent **après `trim`** (valeur enregistrée), précision de la
     validation P3 (2b, R6 b) (`chars().count()`, comme `routes/api_keys.rs:130` : en MariaDB, `VARCHAR(255)` compte des
     caractères — validation P1, R7) ;
  5. montant `> 0`, au plus deux décimales **après `normalize()`** (`"50.00"` et `"50.000"` admis, `"50.0100"` ≡
     `"50.01"` admis, `"50.001"` refusé — `"50.000"` est le cas qui distingue la règle de sa mutation M4), au plus
     treize chiffres entiers ; soldes : mêmes bornes, signe libre ;
  6. **équilibre exact** : `opening + Σ crédits − Σ débits == closing`, sans tolérance (`"0.01"` d'écart refusé).
  L'erreur porte le rang de ligne (1-based) et le champ en cause. `validate_balance` et `from_imported` ne changent pas.
  **Ensembles fermés épinglés côté Rust** (décision de l'orchestrateur à la validation P3, 2b R1 et 2a F7) : deux
  énumérations de `kesh-core`, `ManualStatementReason` (onze variantes) et `ManualStatementField` (treize), chacune avec
  `code() -> &'static str` et une constante `ALL` ; l'erreur de validation les porte, et la route sérialise `code()`. Un
  test Rust **épingle** les deux ensembles sur les chaînes **littérales** de la parente (§ *Ensembles fermés*) : `ALL`
  mappé par `code()` égale, dans les deux sens, la liste écrite dans le test. La 2b ajoutera le test croisé qui lit sa
  table TypeScript et la compare à `ManualStatementReason::ALL`. *(Validation P4, 2a P8 = 2b P3 et 2b P2 :)* `ALL` est
  `pub const`, **hors** `cfg(test)`, atteignable depuis `kesh_core` par un chemin public (un test d'intégration de
  `kesh-api` le lit) ; et un test lie `ALL` à l'**énumération** : une fonction fait un `match` **exhaustif sans joker**
  sur chaque variante et vérifie que `ALL` la contient — le compilateur force alors la mise à jour du test pour toute
  variante neuve (mutation M23-bis). Même dispositif pour `ManualStatementField`.
- **AC-5** — L'empreinte canonique est **stable** (même entrée → mêmes octets, quel que soit l'ordre d'émission des
  champs JSON côté client, `"50"` ≡ `"50.00"`) et **sensible** à chacun des champs qu'elle porte : un test par champ
  (compte, bornes, numéro, chaque solde, et pour une ligne : date, valeur, libellé, contrepartie, référence, montant,
  sens), plus l'ordre des lignes, plus **l'injectivité** : `details = "a|b", reference = "c"` et `details = "a",
  reference = "b|c"` (et toute paire qui ne diffère que par la frontière entre deux champs voisins) donnent des octets
  différents.
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
  Le test `both_import_calls_map_the_archived_account` (`bank_account_closed_inventory.rs:628-634`, « deux appels de
  l'aide attendus ») ne lit que `routes/bank_imports.rs` : il est **étendu** à `bank_statements_manual.rs` (un appel de
  `create_import_or_roll_back(tx,` suivi de `.await?`), hors `REGISTRE` et `FENETRE` (validation P2, R14).
- **AC-9** — `POST /api/v1/bank-imports/manual/preview` et `POST /api/v1/bank-imports/manual`, montés **aux routes
  Comptable+** à côté de l'import (`lib.rs:792-806`), corps JSON (§ *Le corps de requête*), dans un module nouveau
  `crates/kesh-api/src/routes/bank_statements_manual.rs`. Ils réutilisent, rendues `pub(crate)` si besoin :
  `create_import_or_roll_back`, `import_create_error`, `compute_sha256_hex`, `duplicate_file_payload`,
  `import_to_response`, `insert_canonical_audit_log` (l'action en paramètre), `detect_duplicate_lines` /
  `dedup_key_scalar`. *(Précisé à la validation P3, R5 :)* la partie à partager est la construction des clés des
  transactions **existantes** (`routes/bank_imports.rs:1763-1778`, `existing.iter().map(|t| (t.id, dedup_key_scalar(…)))`) :
  elle s'extrait en une fonction appelée par `detect_duplicate_lines_for_imported` **et** par la saisie, qui passe ses
  `BankTransactionDraft` (issus de `manual_statement_to_drafts`) à `detect_duplicate_lines` sans conversion. Aucune
  requête SQL écrite dans le module.
  **L'extracteur du corps** (décision de l'orchestrateur à la validation P3, R1) : la macro
  `impl_validation_extractor` de `routes/bank_accounts.rs:196-226` (locale au fichier) s'**extrait** dans un module
  partagé de `kesh-api` (p. ex. `src/extractors.rs`), réutilisé par `bank_accounts.rs` — comportement **inchangé**, tests
  existants verts — et par la route de saisie ; aucune copie. Une seule règle : tout rejet du corps, `Content-Type`
  absent et dépassement de la limite compris, rend `400 VALIDATION_ERROR` (bras `MissingJsonContentType` et `_` de la
  macro). Avec `KESH_BANK_IMPORT_MAX_MB` réglé à son minimum (1, `config.rs:877-882`), un relevé de 500 lignes pleines
  (≈ 2,1 Mo) est refusé ainsi : limite assumée, écrite à `docs/api-external.md`.
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
  Un test envoie `"direction": "out"` et un corps sans `lines` et constate `BANK_STATEMENT_INVALID` avec le `field` et
  le `reason` attendus (validation P2, R3). *(Ajouté à la validation P3, R1 et R2 :)* un test par cas de rejet du corps,
  chacun `400 VALIDATION_ERROR` **sans** `field` ni `reason` et sans écriture : corps sans `Content-Type`, corps au-delà
  de la limite, JSON illisible, un champ d'un autre type JSON que chaîne (`"amount": 50`), `lines` qui n'est pas un
  tableau, `bankAccountId` absent ou non entier — la limite de « toute faute de forme rend `BANK_STATEMENT_INVALID` » :
  elle ne vaut que pour une chaîne absente, vide ou mal formée. Les messages serveur des codes nouveaux (`errors.rs`, `t("…")`) ont leurs
  clés aux **quatre** catalogues, parité verte. *(Frontière écrite à la validation P4, 2b P1 :)* les deux messages
  serveur qui disent « importée » sont **à la 2a** — `bank-accounts-errors-has-transactions` (repli
  `errors.rs:2714-2721`) et `bank-accounts-errors-has-pending-transactions` (repli `errors.rs:2776-2780`) : repli **et**
  valeur aux quatre catalogues (`fr-CH` `:1433`, `:1439` ; `de-CH`, `it-CH`, `en-CH` `:1369`, `:1375`) récrits pour
  valoir aussi d'un relevé saisi, et le test qui épingle le texte, `crates/kesh-api/tests/bank_accounts_e2e.rs:2485`,
  suivi. Le Vitest qui le recopie (`bank-accounts-page.test.ts:502`) est à la 2b.
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
  littéral d'action. **Et** (validation P2, R1) : inscrire `bank_imports.rs` fait lire au tamis du sens 2
  (`retenir_si_code`, `:648-…`) le littéral `"upload.xml"` (`bank_imports.rs:473`, `:603`, nom par défaut d'un envoi
  multipart) ; il entre à `PAS_DES_CODES` (constante à `:585`, commentaire `:575-584`) avec ce motif, et « huit » devient « neuf » *(liste complétée à la validation P4, P3 :)*
  aux `:20`, `:379` (« les huit vrais ») et `:579`, à l'assertion `SITES_INDIRECTS.len()` (`:919-921`, 8 → 9) et au motif
  de son message (`:925`, `six|sept|huit` gagne `neuf`) — recomptés par `grep -niE '\b(huit|neuf)\b'`. Gate ciblé : `binary(audit_label_registry)`.
  **Compteurs figés du registre des routes** (R2), à recompter, non à ajuster de confiance :
  `audit_route_registry.rs:909` (`LIB_ROUTES.len()`, 135 → 137), `:915` (`traced`, 129 → 130), `:939` (`no_matter`,
  2 → 3), `:944` (138 → 140), `:1324` (`assert_eq!(tout.len(), 138)` → 140), `:1312` (`sans_ecriture`, 110 → 112, et son message « 109 routes de
  lib.rs » → 111 ; validation P2, R8), et les commentaires d'en-tête (l. 27-33) ; et (validation P3, F6) `:910`
  (« l'inventaire porte sur 135 routes ») et les énumérations des messages de `traced` (`:916-937`) et de
  `sans_ecriture` (`:1313`), complétées de « la saisie manuelle d'un relevé » ; et (validation P4, P4) le message de
  `no_matter` (`:940`, « deux routes mutantes qui ne mutent rien » → « trois », l'aperçu de la saisie nommé).
  Le test `le_manuel_dit_le_compte_du_registre` (`:955`) lit `docs/manual/fr/admin-manual.tex:2966` : la phrase devient
  « 130 des 137 routes à verbe mutant », « trois autres routes qui n'écrivent rien parce qu'elles ne modifient rien »,
  « 130 + 4 + 3 = 137 » (vérifier au `.tex` la forme exacte que le test exige).
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
- **AC-28** — `docs/api-external.md` : les deux routes (tableau des ressources, `:223`), le corps, les codes nouveaux
  (tableau des codes d'erreur), la phrase « *(Depuis la v0.17.0.)* » ; et (validation P1, F11) : la liste des écritures
  refusées sur un compte clos (`:243`, « aperçu et création d'un import… ») gagne la saisie, `:866` qui y renvoie se
  relit ; la phrase du rejeu (`:878`, « L'import d'un relevé (`POST /bank-imports`) n'est pas rejoué ») couvre aussi la
  saisie ; les valeurs de `sourceFormat` (`CAMT053_V04`, `CAMT053_V08`, `CSV`, `MANUAL`) et la forme du `filename` d'un
  relevé saisi sont documentées à côté des routes de lecture ; et (validation P3, F5, R1) : la **réponse de l'aperçu**
  (sommes et quatre avertissements), les **ensembles fermés** de `field` et `reason`, l'**ordre des refus**, la limite
  de corps et la règle « tout rejet du corps, `Content-Type` absent et dépassement compris : `400 VALIDATION_ERROR` » ;
  aux `:87`, `:826`, `:853`, une phrase : un relevé saisi par une clé `read-write` ne se supprime pas par cette clé
  (la suppression reste réservée à l'interface).
- **AC-29** — **CHANGELOG et § de mise à jour, passés à la 0.17.0** (C-v016-2-15), dans le commit de la migration ou
  le suivant :
  1. l'entrée `## [0.16.1] — 2026-10-29` devient `## [0.17.0] — Non publié` (forme que `scripts/prepare-release.sh`
     date à la release — la lire, ne pas l'exécuter) ;
  2. *(point déplacé au découpage, validation P2 : AC-29-bis de la `v016-2b`)* ;
  3. son en-tête se récrit : la phrase « Il n'ajoute **aucune migration** : la 0.16.0 redémarre sur une base passée par
     la 0.16.1, et une sauvegarde de l'une s'importe dans l'autre — le retour à la 0.16.0 ne demande aucune sauvegarde
     particulière […] » est **supprimée** et remplacée par : la 0.17.0 ajoute **une migration**
     (`20261030000001_bank_imports_manual_source`) qui relève `kesh_version_min_required` à **0.17.0** — la 0.16.x ne
     démarre plus sur une base passée par la 0.17.0, une sauvegarde de la 0.17.0 ne s'importe plus dans une 0.16.x, et
     le retour passe par la sauvegarde d'avant la mise à jour (base, `.keshbackup`, dossier des documents) ;
  4. une **note sur la candidate `0.16.1-rc1`** : publiée en pré-version le 2026-10-29 (`gcorbaz/kesh:0.16.1-rc1`,
     `:latest` non déplacé), non poursuivie ; son contenu (#811, #812) est livré par la 0.17.0 ; une installation passée
     par la rc1 se met à jour vers la 0.17.0 comme depuis la 0.16.0 ;
  5. au manuel d'administration, `\subsubsection{Passer à la 0.16.1}\label{sec:maj-0-16-1}` (`admin-manual.tex:2481`,
     dont `:2486`, « La 0.16.1 n'ajoute aucune migration … » ; validation P2, R5)
     devient *Passer à la 0.17.0* (`sec:maj-0-17-0`, chaque `\ref` suivi : `grep -rn "maj-0-16-1" docs`), et son texte
     dit la migration et la sauvegarde d'avant la mise à jour comme le point 3, au `.tex` — **sauvegarde complète
     (base, `.keshbackup`, dossier des documents) avant la mise à jour** ; le retour à la `0.16.1-rc1` (ou à la 0.16.0)
     ne passe **que** par elle ; une sauvegarde `.keshbackup` faite en 0.16.x s'importe dans la 0.17.0 (AC-3-bis), non
     l'inverse. *(Première rédaction : « le PDF se régénère à la release ». Corrigé à la validation P2, F3 :)* les **deux**
     PDF FR touchés (administration, utilisateur) se régénèrent **au même commit** que leur `.tex` (*précisé à la validation P3, F3 :*
     `make admin` et `make user` dans `docs/manual/` — non `make fr`, qui reconstruit aussi `marketing-brochure.pdf` —,
     sous verrou et `mem-guard`), **sans toucher aux macros** — leur page de titre dit encore 0.16.1
     jusqu'au tag final, ce que les notes de la candidate disent ; le PDF du tag `v0.17.0-rc1` porte ainsi le
     § *Passer à la 0.17.0* et la sauvegarde d'avant mise à jour. *(Validation P3, F4 :)* `admin-manual.tex:2966` est
     touché par l'AC-15 (commit du registre) : le PDF d'administration se régénère à **chacun** des commits qui touchent
     son `.tex`, ou au dernier d'entre eux, avant tout push. **Contrôle du PDF aplati**
     (`pdftotext … - | tr '\n' ' ' | tr -s ' '`, l'apostrophe y est typographique : greper `’` ou un motif sans
     apostrophe) : il porte `Passer à la 0.17.0`, `130 des 137`, `130 + 4 + 3 = 137`, la forme corrigée de `:2270`, et
     ne porte plus `Passer à la 0.16.1` ni `129 des 135` — sorties copiées au Dev Agent Record.
  6. **tri des `0.16.1` du texte qui décrit le contenu de la version** (validation P1, F2) : au CHANGELOG `[0.17.0]`,
     la phrase d'ouverture (« Correctif de l'Epic 32 … » — *auteur précisé à la validation P3, F8 :* la 2a la récrit pour
     annoncer la migration, la sauvegarde d'avant mise à jour et le correctif MCP ; la 2b y ajoute la fonctionnalité,
     AC-29-bis), le renvoi
     « *Passer à la 0.16.1* » (l. 19) et « reste à confirmer avec Claude Code sur la 0.16.1 » (l. 15) ; au § renommé,
     `admin-manual.tex:2483` (« La 0.16.1 ne demande ni nouveau compose… ») et `:2489` (« la 0.16.1 n'en émettant
     plus ») ; `user-manual.tex:3265` (« à confirmer avec Claude Code sur la 0.16.1 ») ; `README.md:49` (« corrigée côté
     Kesh en 0.16.1 ») ; et, ajoutés à la validation P2 (R5 = F2), `admin-manual.tex:3220` (« Depuis la 0.16.1 » →
     0.17.0, « à confirmer … sur la 0.16.1 » → sur la 0.17.0), `:3238`, `:3349`, `:3351`. **Tableau des propriétaires
     de chaque occurrence de `0.16.1`** (geste de clôture : `grep -rn "0\.16\.1" docs README.md CHANGELOG.md website
     crates frontend/src scripts --exclude-dir=node_modules`, texte aplati pour les `.tex` et PDF, **chaque occurrence
     classée** au Dev Agent Record « historique », « release » ou « corrigée ») :

     | Occurrences | Propriétaire |
     |---|---|
     | les dix `Cargo.toml` (`version`), les neuf commentaires « depuis la 0.16.1 » de `crates/` | 2a (AC-2) |
     | CHANGELOG `[0.17.0]` (l. 11, 13, 15, 19), `admin-manual.tex:2481-2490` (la 0.16.0 commence à `:2491` : historique ; validation P3, F9), `:3220`, `:3238`, `:3349`, `:3351`, `user-manual.tex:3265`, `README.md:49`, `README.md:235` (feuille de route) | 2a (AC-29) |
     | mentions historiques de la candidate `0.16.1-rc1` (note du CHANGELOG, `docs/ci.md:163`, `:169`) | historique, gardées |
     | `kesh-style.sty:72` (`\keshVersion`), URL `raw.githubusercontent.com/…/v0.16.1` (manuels), `website/index.html:70` | release, tag final (B) |
  7. `admin-manual.tex:2270` (§ *Sauvegarde et restauration*) : « ce seuil n'est relevé que par les migrations qui
     changent le schéma de façon incompatible » devient faux (la migration de la 0.17.0 relève le seuil sans changer le
     schéma) — récrire *(forme corrigée à la validation P2, F6)* : « … par les migrations qui changent le schéma de façon
     incompatible, ou lorsqu'une version antérieure lirait mal, ou ne lirait plus, les données écrites par la
     nouvelle » ;
  8. `README.md:235`, ligne `v0.16.1 (correctif)` de la feuille de route : devient la ligne `v0.17.0` (correctif MCP,
     saisie manuelle, migration) — **dans ce commit**, comme le veut la règle de synchronisation du planning du README ;
     G19 ne la couvre pas (aucune URL `raw` au README ; validation P2, F7). *(Décision de l'orchestrateur à la
     validation P3, F1 :)* (a) la phrase « Aucune migration : le **retour à la 0.16.0** se fait en reprenant son image »
     est **retirée** au profit de « une migration relève la version minimale exigée ; le retour passe par la sauvegarde
     d'avant la mise à jour » ; (b) le statut de la ligne ne dit **pas** « ✅ Publiée » pendant la candidate (sigle choisi
     par l'orchestrateur, p. ex. « 🚧 Candidate ») ; (c) la bascule à « ✅ Publiée » se fait au tag final (liste B de la
     release, parente).
  Le développeur ne touche **ni** les macros `\keshVersion`…, **ni** les URL `raw.githubusercontent.com/…/v0.16.1`,
  **ni** le site : ils suivent le bump de `\keshVersion` (garde G19) au **tag final** (§ *Ce que la release doit
  contenir*, B).

## Tâches

1. **Étape 0** — recontrôler au code chaque ligne du § *À recontrôler* ci-dessous, ligne lue citée au Change Log ;
   bases `kesh_v0162` et `kesh_e2e_v0162` (`docs/agents-orchestration.md` § 4).
2. **Tests rouges d'abord** : `kesh-core` (AC-4, AC-5), dépôt (AC-6, AC-7), montée (AC-2-bis), e2e HTTP de création
   nominale (AC-10) et de déséquilibre (AC-11), commités rouges avec un message qui le dit.
3. `kesh-core` : variante `Manual` ; module `manual` (AC-4, AC-5).
4. `kesh-db` : variante `Manual` (AC-1) ; migration **et**, dans le même commit, les dix crates en `0.17.0` (P2-bis),
   `migrations.sha384`, audit, P6, P7, squash (AC-2) ; montée et sauvegarde 0.16.x (AC-2-bis, AC-3, AC-3-bis) ; deux
   lectures (AC-6, AC-7).
5. `kesh-api` : extracteur, erreurs, module de routes, montage, audit, registres (AC-8 à AC-15) ; tests HTTP (AC-10 à
   AC-18).
6. Textes : `docs/api-external.md` (AC-28) ; CHANGELOG et manuel d'administration, tri des `0.16.1`, README (AC-29) ;
   PDF régénérés au même commit que leur `.tex`.
7. Gates, mutations, Dev Agent Record.

## Tests et mutations prévus

Fichiers prévus : `crates/kesh-core/src/bank_imports.rs` (ou `bank_imports/manual.rs`) `mod tests` ;
`crates/kesh-db/src/repositories/bank_imports.rs` (tests `#[sqlx::test]`, squash) ; `crates/kesh-db/tests/migrations_*.rs`
(AC-2, AC-2-bis) ; `crates/kesh-db/tests/bank_accounts_repository.rs` (AC-14) ; `crates/kesh-api/tests/bank_statements_manual_e2e.rs`
(nouveau) ; `crates/kesh-api/tests/admin_full_import_e2e.rs` ou voisin (AC-3, AC-3-bis).

| # | Mutation (une à la fois, motif vérifié unique, restauration `git checkout -- f && touch f`) | Doit rougir |
|---|---|---|
| M1 | équilibre : `!=` remplacé par une tolérance `abs() > 0.01` | AC-4 r. 6 (écart d'un centime refusé) |
| M2 | sens : `debit` enregistré positif | AC-10 (montants signés en base) |
| M3 | règle 3 retirée (date hors période admise) | AC-4 r. 3, AC-11 |
| M4 | règle 5 : `normalize()` retiré avant le contrôle d'échelle | AC-4 r. 5 (`"50.000"` et `"50.0100"` admis ; *première rédaction « `"50.00"` », qui ne la tue pas — validation P3, R3*) |
| M5 | doublons de lignes : sans confirmation, filtrés au lieu de refusés | AC-11 (refus, rien d'écrit) |
| M6 | `confirmOpeningBalanceGap` ignoré (toujours vrai) | AC-11 (`BANK_STATEMENT_OPENING_BALANCE_GAP`) |
| M7 | `previous_statement_closing` : `period_to < ?` → `<= ?` | AC-6 (relevé qui finit le jour même exclu) |
| M8 | `previous_statement_closing` : `ORDER BY period_to ASC` | AC-6 (ordre) |
| M9 | empreinte canonique : le sens d'une ligne omis | AC-5 (sensibilité au sens) |
| M10 | `source_format` écrit `Csv` au lieu de `Manual` | AC-10 |
| M11 | action d'audit `bank_import.created` au lieu de `entered_manually` | AC-10, AC-15 |
| M12 | aperçu : la trace d'audit écrite | AC-12 |
| M16 | migration : `'0.17.0'` → `'0.16.1'` | AC-2 (test `DowngradeRefused` sur `0.16.1` et `min_required`) |
| M17 | migration : ajouter `UPDATE bank_imports SET statement_id = NULL;` avant le relèvement | AC-2-bis (relecture à l'identique ; inventaire des instructions) |
| M18 | `BankImportSourceFormat::from_str` : arme `"CSV"` retirée | AC-2-bis point 2, AC-3-bis (relecture applicative) |
| M20 | `POST /bank-imports/manual` monté hors du groupe Comptable+ | AC-13 (Consultation → `403`) |
| M21 | `confirmDuplicateFile` ignoré (toujours vrai) | AC-11 (`BANK_IMPORT_DUPLICATE_FILE`) |
| M23 | `ManualStatementReason::ALL` : une variante retirée de la constante | test d'épinglage (AC-4) |
| M24 | extracteur partagé : bras `MissingJsonContentType` rendu en `415` | AC-11 (`Content-Type` absent → `400`) *(« tests de `bank_accounts` » retiré à la validation P4, P5 : aucun n'atteint ce bras)* |
| M23-bis | une variante ajoutée à `ManualStatementReason`, absente de `ALL` | test d'exhaustivité par `match` sans joker (AC-4) — rouge à la compilation ou à l'assertion |
| M22 | « un échec de la trace n'écrit rien » : `tx.commit()` avant l'écriture de la trace | AC-10 : test qui fait échouer la trace par un **déclencheur** `BEFORE INSERT ON audit_log` restreint à l'action `bank_import.entered_manually` (`SIGNAL SQLSTATE '45000'`), patron `FAIL_TRIGGER` de `onboarding_audit_e2e.rs:871`, puis `COUNT(*)` de `bank_imports` et `bank_transactions` = 0. *(Première rédaction : « `user_id` inexistant » — faux à la validation P2, R2 : l'entête échoue d'abord sur `fk_bank_imports_user`, et la trace n'a plus de clé étrangère depuis la 25-1a, `audit_log.rs:78-93`.)* |

**M16 et M17 mutent un fichier de migration** (P8 ; validation P2, R13) : tests `#[sqlx::test]` seulement, **aucun
démarrage** contre une base persistante (`kesh_v0162`, `kesh_e2e_v0162`) tant que la mutation est en place ; base E2E
reconstruite au moindre doute. Elles font aussi rougir le registre `migrations.sha384` (R7).

Chaque mutation : le test qui rougit et la raison, au Dev Agent Record. Une mutation **survivante** est un défaut de test
à corriger, non à déclarer.

## Gates

`kesh-db` est touché : **gate complet même en cours de boucle**. Base de gate remise à zéro avant chaque gate complet. Au
**dernier commit de code** de la 2a : `scripts/test-fast.sh --ci` sous `flock "$GATE_LOGS/.gate.lock"`,
`cargo build --workspace --all-targets`, et la **suite E2E complète** selon `docs/testing.md` à la lettre — la 2a ne
change pas l'écran, mais la migration exige un démarrage réel contre une base **persistante** (`kesh_e2e_v0162`, P8) et
le bump de version touche tout le binaire ; échecs jugés fichier par fichier contre `docs/testing.md` § *Les échecs
attendus*. Ne déclarer que ce qui a tourné, chiffres recomptés.

## Fichiers touchés (prévus)

| Module | Fichiers |
|---|---|
| `kesh-core` | `src/bank_imports.rs` (et `src/bank_imports/manual.rs` si le module devient un dossier ; validation P3, R7) |
| `kesh-db` | `src/entities/bank_import.rs`, `src/repositories/bank_imports.rs`, `migrations/20261030000001_bank_imports_manual_source.sql`, `migrations.sha384`, `src/post_restore.rs`, `test-schema/0001_schema_squash.sql` (régénéré), `tests/migrations_fresh_install.rs`, `tests/migrations_upgrade_path.rs`, `tests/bank_accounts_repository.rs`, `docs/migrations-idempotence-audit.md` |
| `kesh-api` | `src/extractors.rs` (nouveau, la macro extraite de `routes/bank_accounts.rs`), `src/routes/bank_accounts.rs`, `src/routes/bank_statements_manual.rs` (nouveau), `src/routes/mod.rs`, `src/routes/bank_imports.rs`, `src/lib.rs`, `src/errors.rs`, `src/audit_labels.rs`, `tests/bank_statements_manual_e2e.rs` (nouveau), `tests/audit_route_registry.rs`, `tests/audit_label_registry.rs`, `tests/bank_account_closed_inventory.rs`, `tests/admin_full_import_e2e.rs`, `tests/bank_accounts_e2e.rs` (`:2485`) |
| crates (P2-bis) | les dix `crates/*/Cargo.toml`, `Cargo.lock` ; les neuf commentaires « depuis la 0.16.1 » |
| `kesh-i18n` | `locales/{fr,de,it,en}-CH/messages.ftl` (messages serveur, libellé d'audit, et les deux `bank-accounts-errors-has-*`) |
| docs | `docs/api-external.md` (`:87`, `:223`, `:243`, `:826`, `:853`, `:866`, `:878`), `CHANGELOG.md`, `README.md` (`:49`, `:235`), `docs/manual/fr/admin-manual.tex` (+ PDF), `docs/manual/fr/user-manual.tex:3265` (+ PDF) |

## À recontrôler à l'étape 0 du développement

- `routes/bank_imports.rs:83-98`, `:389-398`, `:1154-1180`, `:1259-1278`, `:1484-1576`, `:1698-1750`, `:978-984` — ce
  que la fiche leur fait dire.
- `bank_accounts.rs:1798-1804` (ordre du relevé de référence) et `closing_checks.rs:455-470` (critère « à solde »).
- `entities/bank_import.rs:53-60` — le refus des valeurs inconnues (fonde C-v016-2-2).
- `routes/admin.rs:217-223` — le refus d'une sauvegarde de `min_required` supérieur.
- La forme d'inscription à `SITES_INDIRECTS` (`tests/audit_label_registry.rs`) et au registre des routes.
- `routes/bank_accounts.rs:196-226` (la macro à extraire), `routes/bank_imports.rs:1756-1779` (clés existantes),
  `config.rs:877-882` (bornes de la limite), `migrations_upgrade_path.rs:122-125`, `:213`, `:541`, `:557`, `:573`, `:656`,
  `audit_label_registry.rs:585`, `:648`, `post_restore.rs:1214-1219` (ajoutés à la validation P3).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-30 — Sous-fiche créée au découpage de la Story v016-2 (validation P2, C-v016-2-18) ; AC repris de la fiche
  parente à leur état remédié de la P2. Prochaine étape : validation P3 (`v016-2a-validate-prompt-p3.md`).
- 2026-10-30 — **Validation P3** (Sonnet, lentilles R et F, contexte frais ; rapports
  `kesh-gate-logs/v0162a-validate-p3-{R,F}.md`) : **0 CRITICAL, 0 HIGH, 2 MEDIUM, 14 LOW** (R : 1 MEDIUM, 6 LOW ; F :
  1 MEDIUM, 8 LOW). Prémisses vérifiées au code (`routes/bank_accounts.rs:196`, `:230-255` ; `test_endpoints.rs:122-138` ;
  `migrations_upgrade_path.rs:122-125`, `:213` ; `audit_label_registry.rs:585` ; `textes_coherents.rs:46-48`). Traitement :
  - **MEDIUM** — R1 : décision de l'orchestrateur, une seule règle `400 VALIDATION_ERROR` pour tout rejet du corps, la macro
    extraite en module partagé, un test par cas (AC-9, AC-11, AC-28 ; parente corrigée) ; F1 : `README.md:235` — phrase
    « Aucune migration » retirée, statut non « Publiée » pendant la candidate, bascule au tag final (AC-29 point 8 ;
    parente, liste B).
  - **LOW appliqués** — R2 (limite de `BANK_STATEMENT_INVALID`, cas à l'AC-11), R3 (M4 : `"50.000"`), R4 (sites de
    `migrations_upgrade_path.rs`, attribut du squash), R5 (clés des transactions existantes extraites), R6 (registre :
    C-v016-2-14 et C-v016-2-17 annotés « levé par C-v016-2-18 »), R7 (citations), F2 (`:1217-1219`), F3 (`make admin`,
    `make user`), F4 (PDF à chaque commit qui touche son `.tex` ; contrôle du PDF aplati), F5 (`api-external.md` : aperçu,
    ensembles fermés, ordre des refus, suppression par clé), F6 (`:910` et messages), F7 (ensembles épinglés côté Rust,
    décision de l'orchestrateur avec 2b R1 : `ManualStatementReason`, `ManualStatementField`, M23), F8 (auteur de la
    phrase d'ouverture), F9 (`:2481-2490`). Ajout : M24 (extracteur).
  - **LOW non pris** — les deux autres copies de l'extracteur (`routes/test_endpoints.rs:122-138`,
    `routes/reconciliation.rs:928-…`) ne sont pas factorisées par cette story (périmètre minimal) : issue demandée.
  Prochaine étape : validation P4 ciblée (`v016-2a-validate-prompt-p4.md`, Opus).
- 2026-10-30 — **Validation P4** (ciblée, Opus ; rapport `kesh-gate-logs/v0162a-validate-p4.md`) : **0 CRITICAL, 0 HIGH,
  1 MEDIUM, 7 LOW**. Prémisses vérifiées au code (`migrations_upgrade_path.rs` par le grep des valeurs ;
  `audit_label_registry.rs:379`, `:919-925` ; `audit_route_registry.rs:940` ; `config.rs:877-882` ;
  `test_endpoints.rs:111`, `reconciliation.rs:914` ; `gh issue view 829` : OPEN). Traitement :
  - **MEDIUM** — P1 : décisions de l'orchestrateur à la P3 versées au registre (**C-v016-2-19**), borne portée à -19 en
    tête de la 2a et de la 2b, entrée P3 au Change Log de la parente.
  - **LOW appliqués** — P2 (copies de `52` et `86` par le grep des valeurs, `:42-50`, `:541`/`:545` binaire), P3
    (`:379`, `:919-921` 8 → 9, `:925`), P4 (`:940`), P5 (M24), P6 (parente `:169`, `:192` et registre annotés), P7
    (`config.rs:877-882` ; le « LOW non pris » de la P3 — « issue demandée » — est **tracé en #829**, ouverte, citée au
    point 10 de la liste C de la parente ; `test_endpoints.rs:111-…`, `reconciliation.rs:914-…`), P8 (exhaustivité de
    `ALL` par `match` sans joker, M23-bis). Ajout (2b P1, décision de l'orchestrateur) : les deux messages serveur
    `bank-accounts-errors-has-*` passent à la 2a (AC-11).
  - **LOW non pris** : aucun.
  Prochaine étape : validation P5 ciblée (`v016-2a-validate-prompt-p5.md`, Sonnet).
