# Audit d'idempotence des migrations SQL `kesh-db`

Audit créé par **Story 10-2** (cf. `_bmad-output/implementation-artifacts/10-2-migrations-idempotence-downgrade-protection.md` AC #1-3).

## Contexte

Le tracking `_sqlx_migrations` garantit déjà qu'une migration appliquée ne sera **pas** ré-exécutée par `MIGRATOR.run()`. Cet audit documente, pour chaque fichier `.sql` historique, le **comportement attendu en cas de re-exécution manuelle hors sqlx** (par exemple si `_sqlx_migrations` est corrompue, perdue lors d'un restore partiel, ou si un opérateur exécute directement `mariadb < migration.sql` — cas du step CI `Apply migrations to kesh DB` dans `.github/workflows/ci.yml`).

Cet audit est **purement informationnel** : les fichiers `.sql` historiques ne sont **pas** modifiés par Story 10-2 (toute modification, même un commentaire, casserait le checksum SHA-384 sqlx et provoquerait `MigrateError::VersionMismatch` sur toute DB déjà migrée, cf. `sqlx-core-0.8.6/src/migrate/migrator.rs:175-176`).

## Verdicts

- **`yes`** — re-exécution serait no-op (usage `CREATE TABLE IF NOT EXISTS`, `ALTER TABLE … IF [NOT] EXISTS`, `CREATE INDEX IF NOT EXISTS`, `DROP INDEX IF EXISTS`).
- **`tracked-by-sqlx`** — l'idempotence est garantie uniquement par le tracking `_sqlx_migrations`. Re-exécution manuelle hors sqlx échouerait avec un code d'erreur MariaDB précisé en justification (1050 table exists, 1060 duplicate column, 1061 duplicate key, 1091 index not found, etc.).
- **`no`** — re-exécution échouerait toujours (non utilisé sur le repo actuel).

## Table d'audit (27 migrations — 26 historiques + 1 nouvelle Story 10-2)

| Fichier | Idempotence | Justification |
|---|---|---|
| `20260404000001_initial_schema.sql` | tracked-by-sqlx | `CREATE TABLE companies/users/fiscal_years` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260405000001_auth_refresh_tokens.sql` | tracked-by-sqlx | `CREATE TABLE refresh_tokens` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260406000001_refresh_tokens_revoked_reason.sql` | tracked-by-sqlx | `ALTER TABLE refresh_tokens ADD COLUMN revoked_reason` + `ADD CONSTRAINT chk_refresh_tokens_revoked_reason` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1060 (colonne) puis 1061 (constraint). |
| `20260409000001_onboarding_state.sql` | tracked-by-sqlx | `CREATE TABLE onboarding_state` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260410000001_bank_accounts.sql` | tracked-by-sqlx | `CREATE TABLE bank_accounts` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260411000001_accounts.sql` | tracked-by-sqlx | `CREATE TABLE accounts` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260412000001_journal_entries.sql` | tracked-by-sqlx | `CREATE TABLE journal_entries` + `CREATE TABLE journal_entry_lines` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260413000001_audit_log.sql` | tracked-by-sqlx | `CREATE TABLE audit_log` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260414000001_contacts.sql` | tracked-by-sqlx | `CREATE TABLE contacts` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260415000001_products.sql` | tracked-by-sqlx | `CREATE TABLE products` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260416000001_invoices.sql` | tracked-by-sqlx | `CREATE TABLE invoices` + `CREATE TABLE invoice_lines` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260416000002_invoice_lines_line_total_check.sql` | tracked-by-sqlx | `ALTER TABLE invoice_lines ADD CONSTRAINT chk_invoice_lines_line_total_non_negative` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1061 (duplicate constraint). |
| `20260417000001_invoice_validation.sql` | tracked-by-sqlx | `CREATE TABLE invoice_number_sequences` + `CREATE TABLE company_invoice_settings` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260417000002_invoice_validated_journal_entry_check.sql` | tracked-by-sqlx | `ALTER TABLE invoices ADD CONSTRAINT chk_invoices_validated_has_journal_entry` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1061. |
| `20260418000001_country_code.sql` | yes | Tous les `ALTER TABLE companies/contacts ADD COLUMN IF NOT EXISTS country` + `ADD CONSTRAINT IF NOT EXISTS chk_*_country` utilisent les guards (MariaDB ≥ 10.3) — migration explicitement ré-entrante en cas de crash partiel (commentaire en-tête lignes 9-10). |
| `20260419000001_invoice_paid_at.sql` | yes | Tous les `ALTER TABLE invoices ADD COLUMN/ADD CONSTRAINT/CREATE INDEX IF NOT EXISTS` utilisent les guards — migration explicitement ré-entrante (commentaire en-tête lignes 16-17). |
| `20260419000002_users_company_id.sql` | tracked-by-sqlx | `ALTER TABLE users ADD COLUMN company_id` + `ALTER TABLE users MODIFY COLUMN company_id BIGINT NOT NULL` sans `IF NOT EXISTS` (sur les étapes ADD/MODIFY) ; re-exécution hors sqlx échouerait erreur 1060 (colonne déjà présente). |
| `20260419000003_company_invoice_settings.sql` | tracked-by-sqlx | `CREATE INDEX idx_company_invoice_settings_created_at` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1061 (index déjà présent). |
| `20260428000001_vat_rates.sql` | tracked-by-sqlx | `CREATE TABLE vat_rates` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260430000001_kf005_fulltext_indexes.sql` | tracked-by-sqlx | Plusieurs `ALTER TABLE … ADD FULLTEXT INDEX ft_*` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1061. |
| `20260504000001_bank_imports.sql` | tracked-by-sqlx | `CREATE TABLE bank_imports` + `CREATE TABLE bank_transactions` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260505000001_bank_profiles.sql` | tracked-by-sqlx | `CREATE TABLE bank_profiles` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260507000001_bank_imports_relax_hash_unique.sql` | yes | `DROP INDEX IF EXISTS uq_bank_imports_company_hash` + `CREATE INDEX IF NOT EXISTS idx_bank_imports_company_hash` (commentaire d'idempotence lignes 15-16 + instructions SQL lignes 17-18 = lignes 15-18 du fichier ; supporte la re-application ou les partial-state re-runs sans crasher sur erreur 1091 ou 1061). |
| `20260507100001_reconciliation_8_4.sql` | tracked-by-sqlx | `ALTER TABLE bank_transactions ADD COLUMN auto_match_rejected_at` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1060 sur le ADD COLUMN. Le `CREATE INDEX IF NOT EXISTS idx_invoices_company_validated_unpaid_date` qui suit est lui guarded mais l'ALTER est l'étape bloquante. |
| `20260507200001_bank_account_journal_link.sql` | tracked-by-sqlx | `ALTER TABLE bank_accounts ADD COLUMN journal_account_id` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1060 sur le ADD COLUMN. Le `CREATE INDEX IF NOT EXISTS idx_bank_accounts_journal_account` qui suit est lui guarded. |
| `20260513000001_reconciliation_rules.sql` | tracked-by-sqlx | `CREATE TABLE reconciliation_rules` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. |
| `20260522000001_kesh_version.sql` | tracked-by-sqlx | `CREATE TABLE _kesh_version` sans `IF NOT EXISTS` + `INSERT INTO _kesh_version (id, kesh_version_min_required, kesh_version_last_applied) VALUES (1, '0.1.0', '0.1.0')` sans `INSERT IGNORE` ni `ON DUPLICATE KEY UPDATE` ; re-exécution hors sqlx échouerait erreur 1050 sur le CREATE puis 1062 sur l'INSERT. Intentionnel : la nouvelle migration suit la convention historique majoritaire (non-guarded) plutôt que d'introduire une dissymétrie. |
| `20260528000001_companies_is_stub.sql` | tracked-by-sqlx | `ALTER TABLE companies ADD COLUMN is_stub BOOLEAN NOT NULL DEFAULT FALSE` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1060 (colonne déjà présente). Non-breaking (ADD COLUMN avec DEFAULT) → pas de bump `kesh_version_min_required` (Story v011-2 / epic H8). |
| `20260531000001_bank_accounts_archived.sql` | tracked-by-sqlx | `ALTER TABLE bank_accounts ADD COLUMN archived BOOLEAN NOT NULL DEFAULT FALSE` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1060 (colonne déjà présente). Non-breaking (ADD COLUMN avec DEFAULT) → pas de bump `kesh_version_min_required` (Story v014-1). Pas d'index ajouté (YAGNI, FINDING-8 Pass 3 Opus — table ~10 rows max par company). Note : invariant primary par company toujours applicatif-only v0.1 (pas de UNIQUE INDEX partiel sur `(company_id, is_primary=TRUE)`, L5 limitation Story v014-1). |
| `20260605000001_api_keys.sql` | tracked-by-sqlx | `CREATE TABLE api_keys` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. Non-breaking (nouvelle table) → pas de bump `kesh_version_min_required` (Story 17-2a, #100). |
| `20260605000002_audit_log_actor.sql` | tracked-by-sqlx | `ALTER TABLE audit_log ADD COLUMN actor_type ENUM(...) NOT NULL DEFAULT 'user'` + `ADD COLUMN actor_api_key_id BIGINT NULL` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1060 (colonne déjà présente). Non-breaking (ADD COLUMN avec DEFAULT / nullable) → pas de bump `kesh_version_min_required` (Story 17-2a, DC5). Pas de FK sur `actor_api_key_id` (pointeur logique — l'audit doit survivre 10 ans à une clé révoquée/supprimée, cohérent `entity_id` sans FK). |
| `20260610000001_users_email.sql` | yes | `ALTER TABLE users ADD COLUMN IF NOT EXISTS email` + `CREATE INDEX IF NOT EXISTS idx_users_email` utilisent les guards (MariaDB ≥ 10.3) — migration ré-entrante. Non-breaking (ADD COLUMN nullable) → pas de bump `kesh_version_min_required` (Story 17-4a, #122). |
| `20260610000002_password_reset_tokens.sql` | tracked-by-sqlx | `CREATE TABLE password_reset_tokens` sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1050. Non-breaking (nouvelle table) → pas de bump `kesh_version_min_required` (Story 17-4a, #122). FK `ON DELETE CASCADE` (DC11, tokens éphémères). |
| `20260613000001_vat_rates_crud.sql` | tracked-by-sqlx | `ALTER TABLE vat_rates ADD COLUMN version` + `ADD COLUMN category` (+ CHECK non-vide) sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1060. Le `UPDATE … CASE` de backfill est intrinsèquement idempotent (re-jeu sans effet : ELSE conserve). Non-breaking (ADD COLUMN avec DEFAULT) → pas de bump `kesh_version_min_required` (Story 11-1). Pas de `CHECK IN (liste fermée)` sur `category` (modèle extensible — nouvelles catégories officielles sans migration). |
| `20260614000001_vat_accounts_config.sql` | tracked-by-sqlx | `ALTER TABLE company_invoice_settings ADD COLUMN default_vat_payable_account_id / default_vat_recoverable_account_id / default_vat_decompte_account_id` (+ 3 FK `fk_cis_vat_*`) sans `IF NOT EXISTS` ; re-exécution hors sqlx échouerait erreur 1060 (colonne déjà présente). Les deux `INSERT … SELECT … WHERE NOT EXISTS` (comptes `1171`/`2206` par company) sont intrinsèquement idempotents (re-jeu sans doublon, garanti par `NOT EXISTS` + `uq_accounts_company_number`) et ne touchent aucun compte existant. Non-breaking (ADD COLUMN nullable + INSERT idempotent) → pas de bump `kesh_version_min_required` (Story 18-1a, #180). |

## Statistiques

- **Total** : 35 migrations (26 historiques + 1 Story 10-2 + 1 Story v011-2 + 1 Story v014-1 + 2 Story 17-2a + 2 Story 17-4a + 1 Story 11-1 + 1 Story 18-1a).
- **Idempotence `yes`** : 4 (`country_code`, `invoice_paid_at`, `bank_imports_relax_hash_unique`, `users_email`).
- **Idempotence `tracked-by-sqlx`** : 31 (toutes les autres).
- **Idempotence `no`** : 0.

## Maintenance future

Conformément à la **politique P5** de `CLAUDE.md` `## Migration breaking policy` (codifiée par Story 10-2 AC #22), toute PR introduisant un nouveau fichier `crates/kesh-db/migrations/*.sql` DOIT ajouter une ligne correspondante dans ce tableau. L'oubli est un finding **MEDIUM** en code review (`bmad-code-review`).
