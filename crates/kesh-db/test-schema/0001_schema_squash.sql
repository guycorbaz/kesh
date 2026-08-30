-- SQUASH DU SCHÉMA DE TEST — Story 22-5 (#251). GÉNÉRÉ, NE PAS ÉDITER.
-- Régénérer : scripts/regen-test-schema.sh
-- Équivalent des 65 migrations de crates/kesh-db/migrations/,
-- rejouées en UN batch DDL par base éphémère de test.
--
-- Le garde-fou crates/kesh-db/tests/test_schema_guard.rs compare ce schéma
-- au vrai à chaque gate : une migration ajoutée sans régénération rougit.


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!50503 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `_kesh_version`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `_kesh_version` (
  `id` tinyint(3) unsigned NOT NULL DEFAULT 1,
  `kesh_version_min_required` varchar(40) NOT NULL,
  `kesh_version_last_applied` varchar(40) NOT NULL,
  `applied_at` datetime NOT NULL DEFAULT current_timestamp(),
  `last_boot_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `chk_kesh_version_single_row` CHECK (`id` = 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `accounts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `accounts` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `number` varchar(10) NOT NULL,
  `name` varchar(255) NOT NULL,
  `account_type` varchar(20) NOT NULL COMMENT 'Asset|Liability|Revenue|Expense',
  `parent_id` bigint(20) DEFAULT NULL,
  `active` tinyint(1) NOT NULL DEFAULT 1,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `role` varchar(32) DEFAULT NULL COMMENT 'Rôle métier explicite (Story 14-3a) — NULL = aucun',
  `postable` tinyint(1) NOT NULL DEFAULT 1 COMMENT 'FALSE = compte titre/regroupement ou compte de résultat calculé',
  `singleton_role` varchar(32) GENERATED ALWAYS AS (case when `active` <> 0 and `role` in ('Receivable','DefaultRevenue','Payable','VatRecoverable','VatPayable','VatSettlement','RetainedEarnings','CurrentYearResult') then `role` else NULL end) VIRTUAL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_accounts_company_number` (`company_id`,`number`),
  UNIQUE KEY `uq_accounts_company_singleton_role` (`company_id`,`singleton_role`),
  KEY `fk_accounts_parent` (`parent_id`),
  CONSTRAINT `fk_accounts_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_accounts_parent` FOREIGN KEY (`parent_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `chk_accounts_type` CHECK (cast(`account_type` as char charset binary) in (cast('Asset' as char charset binary),cast('Liability' as char charset binary),cast('Revenue' as char charset binary),cast('Expense' as char charset binary))),
  CONSTRAINT `chk_accounts_number_nonempty` CHECK (char_length(trim(`number`)) > 0),
  CONSTRAINT `chk_accounts_name_nonempty` CHECK (char_length(trim(`name`)) > 0),
  CONSTRAINT `chk_accounts_role` CHECK (`role` is null or cast(`role` as char charset binary) in (cast('Receivable' as char charset binary),cast('DefaultRevenue' as char charset binary),cast('Payable' as char charset binary),cast('VatRecoverable' as char charset binary),cast('VatPayable' as char charset binary),cast('VatSettlement' as char charset binary),cast('EquityCapital' as char charset binary),cast('EquityOther' as char charset binary),cast('RetainedEarnings' as char charset binary),cast('CurrentYearResult' as char charset binary)))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `api_keys`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `api_keys` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `created_by_user_id` bigint(20) NOT NULL COMMENT 'Créateur de la clé — le PAT authentifie en son nom (DC2)',
  `name` varchar(255) NOT NULL COMMENT 'Libellé human-readable de l''intégration',
  `key_hash` char(64) NOT NULL COMMENT 'SHA-256(token) hex — jamais le secret en clair (DC1)',
  `scope` varchar(16) NOT NULL COMMENT 'read | read-write (DC3)',
  `expires_at` datetime(3) DEFAULT NULL COMMENT 'Expiration optionnelle ; NULL = permanente jusqu''à révocation',
  `last_used_at` datetime(3) DEFAULT NULL COMMENT 'Best-effort, mis à jour au lookup auth (eventual consistency)',
  `revoked_at` datetime(3) DEFAULT NULL COMMENT 'Soft-delete — révocation immédiate (find_active exclut)',
  `version` int(11) NOT NULL DEFAULT 1 COMMENT 'Optimistic lock pour la révocation',
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_api_keys_key_hash` (`key_hash`),
  KEY `fk_api_keys_created_by` (`created_by_user_id`),
  KEY `idx_api_keys_company` (`company_id`),
  KEY `idx_api_keys_created` (`created_at` DESC),
  CONSTRAINT `fk_api_keys_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_api_keys_created_by` FOREIGN KEY (`created_by_user_id`) REFERENCES `users` (`id`),
  CONSTRAINT `chk_api_keys_name_nonempty` CHECK (char_length(trim(`name`)) > 0),
  CONSTRAINT `chk_api_keys_scope` CHECK (`scope` in ('read','read-write'))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `audit_log`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `audit_log` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `user_id` bigint(20) NOT NULL,
  `action` varchar(64) NOT NULL COMMENT 'ex: journal_entry.updated, journal_entry.deleted',
  `entity_type` varchar(32) NOT NULL COMMENT 'ex: journal_entry',
  `entity_id` bigint(20) NOT NULL COMMENT 'Pointeur logique (PAS une FK) — survit aux DELETE',
  `details_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL COMMENT 'Snapshot before/after ou autre contexte' CHECK (json_valid(`details_json`)),
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `actor_type` enum('user','api_key') NOT NULL DEFAULT 'user' COMMENT 'Story 17-2a — user (UI/JWT) ou api_key (PAT)',
  `actor_api_key_id` bigint(20) DEFAULT NULL COMMENT 'Story 17-2a — id clé API si actor_type=api_key (pointeur logique, pas de FK)',
  PRIMARY KEY (`id`),
  KEY `idx_audit_log_entity` (`entity_type`,`entity_id`),
  KEY `idx_audit_log_user_date` (`user_id`,`created_at` DESC),
  CONSTRAINT `fk_audit_log_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`),
  CONSTRAINT `chk_audit_log_action_nonempty` CHECK (char_length(trim(`action`)) > 0),
  CONSTRAINT `chk_audit_log_entity_type_nonempty` CHECK (char_length(trim(`entity_type`)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `bank_accounts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `bank_accounts` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `bank_name` varchar(255) NOT NULL,
  `iban` varchar(34) NOT NULL COMMENT 'IBAN normalisé sans espaces',
  `qr_iban` varchar(34) DEFAULT NULL COMMENT 'QR-IBAN optionnel (QR-IID 30000-31999)',
  `journal_account_id` bigint(20) DEFAULT NULL,
  `is_primary` tinyint(1) NOT NULL DEFAULT 0,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `archived` tinyint(1) NOT NULL DEFAULT 0,
  PRIMARY KEY (`id`),
  KEY `fk_bank_accounts_company` (`company_id`),
  KEY `idx_bank_accounts_journal_account` (`journal_account_id`),
  CONSTRAINT `fk_bank_accounts_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `chk_bank_accounts_bank_name_nonempty` CHECK (char_length(trim(`bank_name`)) > 0),
  CONSTRAINT `chk_bank_accounts_iban_nonempty` CHECK (char_length(trim(`iban`)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `bank_imports`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `bank_imports` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `bank_account_id` bigint(20) NOT NULL,
  `filename` varchar(255) NOT NULL,
  `file_hash` char(64) NOT NULL,
  `source_format` varchar(32) NOT NULL,
  `statement_id` varchar(255) DEFAULT NULL,
  `period_from` date NOT NULL,
  `period_to` date NOT NULL,
  `opening_balance` decimal(18,2) DEFAULT NULL,
  `closing_balance` decimal(18,2) DEFAULT NULL,
  `transaction_count` int(11) NOT NULL DEFAULT 0,
  `imported_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `imported_by_user_id` bigint(20) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `fk_bank_imports_bank_account` (`bank_account_id`),
  KEY `fk_bank_imports_user` (`imported_by_user_id`),
  KEY `idx_bank_imports_company_account_imported` (`company_id`,`bank_account_id`,`imported_at`),
  KEY `idx_bank_imports_company_hash` (`company_id`,`file_hash`),
  CONSTRAINT `fk_bank_imports_bank_account` FOREIGN KEY (`bank_account_id`) REFERENCES `bank_accounts` (`id`),
  CONSTRAINT `fk_bank_imports_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_bank_imports_user` FOREIGN KEY (`imported_by_user_id`) REFERENCES `users` (`id`),
  CONSTRAINT `chk_bank_imports_period` CHECK (`period_to` >= `period_from`),
  CONSTRAINT `chk_bank_imports_tx_count` CHECK (`transaction_count` >= 0),
  CONSTRAINT `chk_bank_imports_hash_len` CHECK (char_length(`file_hash`) = 64)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `bank_profiles`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `bank_profiles` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `bank_name` varchar(100) NOT NULL,
  `filename_pattern` varchar(200) DEFAULT NULL,
  `column_mapping` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL CHECK (json_valid(`column_mapping`)),
  `date_format` varchar(50) NOT NULL,
  `decimal_separator` char(1) NOT NULL,
  `field_separator` char(1) NOT NULL,
  `encoding` varchar(20) DEFAULT NULL,
  `header_row_count` tinyint(3) unsigned NOT NULL DEFAULT 1,
  `created_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
  `updated_at` datetime(6) NOT NULL DEFAULT current_timestamp(6) ON UPDATE current_timestamp(6),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_bank_profiles_company_name` (`company_id`,`bank_name`),
  KEY `idx_bank_profiles_company` (`company_id`),
  KEY `idx_bank_profiles_company_pattern` (`company_id`,`filename_pattern`),
  CONSTRAINT `fk_bank_profiles_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chk_bank_profiles_field_separator` CHECK (`field_separator` in (',',';','	')),
  CONSTRAINT `chk_bank_profiles_decimal_separator` CHECK (`decimal_separator` in ('.',',')),
  CONSTRAINT `chk_bank_profiles_separators_distinct` CHECK (`field_separator` <> `decimal_separator`),
  CONSTRAINT `chk_bank_profiles_header_row_count` CHECK (`header_row_count` <= 5),
  CONSTRAINT `chk_bank_profiles_bank_name_len` CHECK (char_length(`bank_name`) between 1 and 100),
  CONSTRAINT `chk_bank_profiles_column_mapping_valid` CHECK (json_valid(`column_mapping`)),
  CONSTRAINT `chk_bank_profiles_date_format_len` CHECK (char_length(`date_format`) between 1 and 50),
  CONSTRAINT `chk_bank_profiles_filename_pattern_len` CHECK (`filename_pattern` is null or char_length(`filename_pattern`) between 1 and 200)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `bank_transactions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `bank_transactions` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `import_id` bigint(20) NOT NULL,
  `bank_account_id` bigint(20) NOT NULL,
  `booking_date` date NOT NULL,
  `value_date` date DEFAULT NULL,
  `amount` decimal(18,2) NOT NULL,
  `currency` char(3) NOT NULL,
  `reference` varchar(255) DEFAULT NULL,
  `details` text NOT NULL,
  `end_to_end_id` varchar(255) DEFAULT NULL,
  `transaction_id` varchar(255) DEFAULT NULL,
  `counterparty_iban` varchar(34) DEFAULT NULL,
  `counterparty_name` varchar(255) DEFAULT NULL,
  `status` varchar(16) NOT NULL DEFAULT 'pending',
  `matched_entry_id` bigint(20) DEFAULT NULL,
  `auto_match_rejected_at` datetime(3) DEFAULT NULL,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  KEY `fk_bank_transactions_bank_account` (`bank_account_id`),
  KEY `fk_bank_transactions_matched_entry` (`matched_entry_id`),
  KEY `idx_bank_transactions_company_account_date` (`company_id`,`bank_account_id`,`booking_date`),
  KEY `idx_bank_transactions_import` (`import_id`),
  KEY `idx_bank_transactions_pending` (`company_id`,`bank_account_id`,`status`,`booking_date`),
  CONSTRAINT `fk_bank_transactions_bank_account` FOREIGN KEY (`bank_account_id`) REFERENCES `bank_accounts` (`id`),
  CONSTRAINT `fk_bank_transactions_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_bank_transactions_import` FOREIGN KEY (`import_id`) REFERENCES `bank_imports` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_bank_transactions_matched_entry` FOREIGN KEY (`matched_entry_id`) REFERENCES `journal_entries` (`id`) ON DELETE SET NULL,
  CONSTRAINT `chk_bank_transactions_status` CHECK (`status` in ('pending','reconciled')),
  CONSTRAINT `chk_bank_transactions_currency_iso4217` CHECK (char_length(`currency`) = 3)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `companies`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `companies` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `name` varchar(255) NOT NULL,
  `address` text NOT NULL,
  `ide_number` varchar(15) DEFAULT NULL COMMENT 'Format: CHExxxxxxxxx (normalisé, sans séparateurs)',
  `org_type` varchar(20) NOT NULL COMMENT 'Independant|Association|Pme (ASCII par design, pas d''accent)',
  `accounting_language` char(2) NOT NULL COMMENT 'FR|DE|IT|EN — langue des libellés comptables',
  `instance_language` char(2) NOT NULL COMMENT 'FR|DE|IT|EN — langue de l''interface',
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `country` char(2) NOT NULL DEFAULT 'CH',
  `is_stub` tinyint(1) NOT NULL DEFAULT 0,
  `address_street` varchar(70) NOT NULL DEFAULT '',
  `address_building` varchar(16) NOT NULL DEFAULT '',
  `address_postal_code` varchar(16) NOT NULL DEFAULT '',
  `address_city` varchar(35) NOT NULL DEFAULT '',
  `address_country` char(2) NOT NULL DEFAULT 'CH',
  `first_name` varchar(70) DEFAULT NULL,
  `last_name` varchar(70) DEFAULT NULL,
  `email` varchar(320) DEFAULT NULL,
  `phone` varchar(50) DEFAULT NULL,
  `website` varchar(255) DEFAULT NULL,
  `books_locked_through` date DEFAULT NULL COMMENT 'Story 24-4c (#380) : borne INCLUSIVE du verrou de période. NULL = aucun verrou. Aucune écriture ne peut être créée avec une entry_date <= cette date.',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_companies_ide_number` (`ide_number`),
  CONSTRAINT `chk_companies_org_type` CHECK (cast(`org_type` as char charset binary) in (cast('Independant' as char charset binary),cast('Association' as char charset binary),cast('Pme' as char charset binary))),
  CONSTRAINT `chk_companies_accounting_language` CHECK (cast(`accounting_language` as char charset binary) in (cast('FR' as char charset binary),cast('DE' as char charset binary),cast('IT' as char charset binary),cast('EN' as char charset binary))),
  CONSTRAINT `chk_companies_instance_language` CHECK (cast(`instance_language` as char charset binary) in (cast('FR' as char charset binary),cast('DE' as char charset binary),cast('IT' as char charset binary),cast('EN' as char charset binary))),
  CONSTRAINT `chk_companies_name_nonempty` CHECK (char_length(trim(`name`)) > 0),
  CONSTRAINT `chk_companies_address_nonempty` CHECK (char_length(trim(`address`)) > 0),
  CONSTRAINT `chk_companies_ide_format` CHECK (`ide_number` is null or `ide_number` regexp '^CHE[0-9]{9}$'),
  CONSTRAINT `chk_companies_country` CHECK (`country` regexp '^[A-Z]{2}$')
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `company_dunning_settings`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `company_dunning_settings` (
  `company_id` bigint(20) NOT NULL,
  `grace_period_days` int(11) NOT NULL DEFAULT 5,
  `seeded_at` datetime(3) DEFAULT NULL,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`company_id`),
  CONSTRAINT `fk_cds_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `chk_cds_grace_nonneg` CHECK (`grace_period_days` >= 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `company_invoice_settings`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `company_invoice_settings` (
  `company_id` bigint(20) NOT NULL,
  `invoice_number_format` varchar(64) NOT NULL DEFAULT 'F-{YEAR}-{SEQ:04}',
  `default_receivable_account_id` bigint(20) DEFAULT NULL,
  `default_revenue_account_id` bigint(20) DEFAULT NULL,
  `default_sales_journal` varchar(10) NOT NULL DEFAULT 'Ventes',
  `journal_entry_description_template` varchar(128) NOT NULL DEFAULT '{YEAR}-{INVOICE_NUMBER}',
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `default_vat_payable_account_id` bigint(20) DEFAULT NULL,
  `default_vat_recoverable_account_id` bigint(20) DEFAULT NULL,
  `default_vat_decompte_account_id` bigint(20) DEFAULT NULL,
  `credit_note_number_format` varchar(64) NOT NULL DEFAULT 'AV-{YEAR}-{SEQ:04}',
  `default_payable_account_id` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`company_id`),
  KEY `fk_cis_receivable` (`default_receivable_account_id`),
  KEY `fk_cis_revenue` (`default_revenue_account_id`),
  KEY `idx_company_invoice_settings_created_at` (`created_at`),
  KEY `fk_cis_vat_payable` (`default_vat_payable_account_id`),
  KEY `fk_cis_vat_recoverable` (`default_vat_recoverable_account_id`),
  KEY `fk_cis_vat_decompte` (`default_vat_decompte_account_id`),
  KEY `fk_cis_payable_account` (`default_payable_account_id`),
  CONSTRAINT `fk_cis_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_cis_payable_account` FOREIGN KEY (`default_payable_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `fk_cis_receivable` FOREIGN KEY (`default_receivable_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `fk_cis_revenue` FOREIGN KEY (`default_revenue_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `fk_cis_vat_decompte` FOREIGN KEY (`default_vat_decompte_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `fk_cis_vat_payable` FOREIGN KEY (`default_vat_payable_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `fk_cis_vat_recoverable` FOREIGN KEY (`default_vat_recoverable_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `chk_cis_journal` CHECK (cast(`default_sales_journal` as char charset binary) in (cast('Achats' as char charset binary),cast('Ventes' as char charset binary),cast('Banque' as char charset binary),cast('Caisse' as char charset binary),cast('OD' as char charset binary))),
  CONSTRAINT `chk_cis_format_nonempty` CHECK (char_length(trim(`invoice_number_format`)) > 0),
  CONSTRAINT `chk_cis_je_template_nonempty` CHECK (char_length(trim(`journal_entry_description_template`)) > 0),
  CONSTRAINT `chk_cis_cn_format_nonempty` CHECK (char_length(trim(`credit_note_number_format`)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `contact_persons`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `contact_persons` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `contact_id` bigint(20) NOT NULL,
  `first_name` varchar(70) NOT NULL,
  `last_name` varchar(70) NOT NULL,
  `role` varchar(100) DEFAULT NULL,
  `email` varchar(320) DEFAULT NULL,
  `phone` varchar(50) DEFAULT NULL,
  `active` tinyint(1) NOT NULL DEFAULT 1,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  KEY `fk_contact_persons_company` (`company_id`),
  KEY `idx_contact_persons_contact` (`contact_id`,`active`),
  CONSTRAINT `fk_contact_persons_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_contact_persons_contact` FOREIGN KEY (`contact_id`) REFERENCES `contacts` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chk_contact_persons_names` CHECK (char_length(trim(`first_name`)) > 0 and char_length(trim(`last_name`)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `contacts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `contacts` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `contact_type` varchar(20) NOT NULL,
  `name` varchar(255) NOT NULL,
  `is_client` tinyint(1) NOT NULL DEFAULT 0,
  `is_supplier` tinyint(1) NOT NULL DEFAULT 0,
  `address` varchar(500) DEFAULT NULL,
  `email` varchar(320) DEFAULT NULL,
  `phone` varchar(50) DEFAULT NULL,
  `ide_number` varchar(12) DEFAULT NULL,
  `default_payment_terms` varchar(100) DEFAULT NULL,
  `active` tinyint(1) NOT NULL DEFAULT 1,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `country` char(2) NOT NULL DEFAULT 'CH',
  `address_street` varchar(70) DEFAULT NULL,
  `address_building` varchar(16) DEFAULT NULL,
  `address_postal_code` varchar(16) DEFAULT NULL,
  `address_city` varchar(35) DEFAULT NULL,
  `address_country` char(2) DEFAULT NULL,
  `first_name` varchar(70) DEFAULT NULL,
  `last_name` varchar(70) DEFAULT NULL,
  `language` char(2) DEFAULT NULL,
  `salutation` varchar(10) NOT NULL DEFAULT 'Neutre',
  `default_payment_terms_days` int(11) DEFAULT NULL,
  `client_number` varchar(50) DEFAULT NULL COMMENT 'Numéro de client attribué par l''émetteur (Story 16-3b) — NULL = non renseigné',
  `client_number_canonical` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL COMMENT 'Forme canonique de client_number (kesh_core::text::canonical_key, Story 22-1) — colonne de comparaison, jamais affichée',
  `client_number_uniq` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin GENERATED ALWAYS AS (case when `active` then `client_number_canonical` else NULL end) VIRTUAL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_contacts_company_ide` (`company_id`,`ide_number`),
  UNIQUE KEY `uq_contacts_company_client_number` (`company_id`,`client_number_uniq`),
  KEY `idx_contacts_company_active` (`company_id`,`active`),
  KEY `idx_contacts_company_name` (`company_id`,`name`),
  FULLTEXT KEY `ft_contacts_name` (`name`),
  CONSTRAINT `fk_contacts_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `chk_contacts_name_not_empty` CHECK (char_length(trim(`name`)) > 0),
  CONSTRAINT `chk_contacts_type` CHECK (cast(`contact_type` as char charset binary) in (cast('Personne' as char charset binary),cast('Entreprise' as char charset binary))),
  CONSTRAINT `chk_contacts_country` CHECK (`country` regexp '^[A-Z]{2}$'),
  CONSTRAINT `chk_contacts_language` CHECK (`language` is null or cast(`language` as char charset binary) in (cast('FR' as char charset binary),cast('DE' as char charset binary),cast('IT' as char charset binary),cast('EN' as char charset binary))),
  CONSTRAINT `chk_contacts_salutation` CHECK (`salutation` in ('Monsieur','Madame','Neutre')),
  CONSTRAINT `chk_contacts_payment_terms_days` CHECK (`default_payment_terms_days` is null or `default_payment_terms_days` >= 0 and `default_payment_terms_days` <= 365)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `credit_note_lines`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `credit_note_lines` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `credit_note_id` bigint(20) NOT NULL,
  `position` int(11) NOT NULL,
  `description` varchar(1000) NOT NULL,
  `quantity` decimal(19,4) NOT NULL,
  `unit_price` decimal(19,4) NOT NULL,
  `vat_rate` decimal(5,2) NOT NULL,
  `line_total` decimal(19,4) NOT NULL,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `revenue_account_id` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_credit_note_lines_position` (`credit_note_id`,`position`),
  KEY `idx_credit_note_lines_credit_note` (`credit_note_id`),
  KEY `idx_credit_note_lines_revenue_account` (`revenue_account_id`),
  CONSTRAINT `fk_credit_note_lines_credit_note` FOREIGN KEY (`credit_note_id`) REFERENCES `credit_notes` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_credit_note_lines_revenue_account` FOREIGN KEY (`revenue_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `chk_credit_note_lines_quantity_positive` CHECK (`quantity` > 0),
  CONSTRAINT `chk_credit_note_lines_unit_price_non_negative` CHECK (`unit_price` >= 0),
  CONSTRAINT `chk_credit_note_lines_vat_rate_range` CHECK (`vat_rate` >= 0 and `vat_rate` <= 100),
  CONSTRAINT `chk_credit_note_lines_description_not_empty` CHECK (char_length(trim(`description`)) > 0),
  CONSTRAINT `chk_credit_note_lines_line_total_non_negative` CHECK (`line_total` >= 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `credit_note_number_sequences`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `credit_note_number_sequences` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `fiscal_year_id` bigint(20) NOT NULL,
  `next_number` bigint(20) NOT NULL DEFAULT 1,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_cns_company_fy` (`company_id`,`fiscal_year_id`),
  KEY `fk_cns_fiscal_year` (`fiscal_year_id`),
  CONSTRAINT `fk_cns_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_cns_fiscal_year` FOREIGN KEY (`fiscal_year_id`) REFERENCES `fiscal_years` (`id`),
  CONSTRAINT `chk_cns_next_positive` CHECK (`next_number` >= 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `credit_notes`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `credit_notes` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `contact_id` bigint(20) NOT NULL,
  `invoice_id` bigint(20) NOT NULL,
  `credit_note_number` varchar(64) DEFAULT NULL,
  `status` varchar(16) NOT NULL,
  `date` date NOT NULL,
  `total_amount` decimal(19,4) NOT NULL DEFAULT 0.0000,
  `journal_entry_id` bigint(20) DEFAULT NULL,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_credit_notes_invoice` (`invoice_id`),
  UNIQUE KEY `uq_credit_notes_number` (`company_id`,`credit_note_number`),
  KEY `fk_credit_notes_journal_entry` (`journal_entry_id`),
  KEY `idx_credit_notes_company_status` (`company_id`,`status`),
  KEY `idx_credit_notes_company_date` (`company_id`,`date`),
  KEY `idx_credit_notes_contact` (`contact_id`),
  CONSTRAINT `fk_credit_notes_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_credit_notes_contact` FOREIGN KEY (`contact_id`) REFERENCES `contacts` (`id`),
  CONSTRAINT `fk_credit_notes_invoice` FOREIGN KEY (`invoice_id`) REFERENCES `invoices` (`id`),
  CONSTRAINT `fk_credit_notes_journal_entry` FOREIGN KEY (`journal_entry_id`) REFERENCES `journal_entries` (`id`),
  CONSTRAINT `chk_credit_notes_status` CHECK (`status` in ('draft','issued','cancelled')),
  CONSTRAINT `chk_credit_notes_total_non_negative` CHECK (`total_amount` >= 0),
  CONSTRAINT `chk_credit_notes_issued_has_je` CHECK (`status` <> 'issued' or `credit_note_number` is not null and `journal_entry_id` is not null)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `dunning_levels`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `dunning_levels` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `level_number` smallint(6) NOT NULL,
  `delay_days` int(11) NOT NULL,
  `fee_amount` decimal(7,2) NOT NULL,
  `version` int(11) NOT NULL DEFAULT 0,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_dunning_levels_company_level` (`company_id`,`level_number`),
  KEY `idx_dunning_levels_company` (`company_id`),
  CONSTRAINT `fk_dunning_levels_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `chk_dunning_levels_fee_range` CHECK (`fee_amount` >= 0 and `fee_amount` <= 10000),
  CONSTRAINT `chk_dunning_levels_delay_nonneg` CHECK (`delay_days` >= 0),
  CONSTRAINT `chk_dunning_levels_level_positive` CHECK (`level_number` >= 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `email_templates`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `email_templates` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `template_type` varchar(50) NOT NULL,
  `language` char(2) NOT NULL,
  `subject` text NOT NULL,
  `body` text NOT NULL,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(6) NOT NULL DEFAULT current_timestamp(6),
  `updated_at` datetime(6) NOT NULL DEFAULT current_timestamp(6) ON UPDATE current_timestamp(6),
  `level_number` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_email_templates_company_type_language_level` (`company_id`,`template_type`,`language`,`level_number`),
  CONSTRAINT `fk_email_templates_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chk_email_templates_language` CHECK (cast(`language` as char charset binary) in (cast('FR' as char charset binary),cast('DE' as char charset binary),cast('IT' as char charset binary),cast('EN' as char charset binary))),
  CONSTRAINT `chk_email_templates_subject_body_nonempty` CHECK (char_length(trim(`subject`)) > 0 and char_length(trim(`body`)) > 0),
  CONSTRAINT `chk_email_templates_template_type` CHECK (`template_type` in ('invoice_send','invoice_reminder'))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `fiscal_years`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `fiscal_years` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `name` varchar(50) NOT NULL COMMENT 'ex: "Exercice 2026"',
  `start_date` date NOT NULL,
  `end_date` date NOT NULL,
  `status` varchar(10) NOT NULL DEFAULT 'Open' COMMENT 'Open|Closed',
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_fiscal_years_company_name` (`company_id`,`name`),
  UNIQUE KEY `uq_fiscal_years_company_start_date` (`company_id`,`start_date`),
  CONSTRAINT `fk_fiscal_years_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `chk_fiscal_years_dates` CHECK (`end_date` > `start_date`),
  CONSTRAINT `chk_fiscal_years_status` CHECK (cast(`status` as char charset binary) in (cast('Open' as char charset binary),cast('Closed' as char charset binary))),
  CONSTRAINT `chk_fiscal_years_name_nonempty` CHECK (char_length(trim(`name`)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `imported_supplier_invoices`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `imported_supplier_invoices` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `status` varchar(16) NOT NULL DEFAULT 'to_complete',
  `supplier_invoice_id` bigint(20) DEFAULT NULL,
  `file_hash` char(64) NOT NULL,
  `storage_path` varchar(512) NOT NULL,
  `original_filename` varchar(255) NOT NULL,
  `mime_type` varchar(100) NOT NULL,
  `byte_size` bigint(20) NOT NULL,
  `creditor_iban` varchar(34) NOT NULL,
  `is_qr_iban` tinyint(1) NOT NULL,
  `creditor_address_type` char(1) NOT NULL,
  `creditor_name` varchar(70) NOT NULL,
  `creditor_line1` varchar(70) DEFAULT NULL,
  `creditor_line2` varchar(70) DEFAULT NULL,
  `creditor_postal_code` varchar(16) DEFAULT NULL,
  `creditor_town` varchar(35) DEFAULT NULL,
  `creditor_country` char(2) NOT NULL,
  `reference_type` varchar(8) NOT NULL,
  `reference_value` varchar(40) DEFAULT NULL,
  `amount` decimal(19,4) DEFAULT NULL,
  `currency` varchar(3) NOT NULL,
  `unstructured_message` varchar(140) DEFAULT NULL,
  `billing_information` varchar(140) DEFAULT NULL,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_imported_company_hash` (`company_id`,`file_hash`),
  KEY `idx_imported_company_status` (`company_id`,`status`),
  KEY `idx_imported_supplier_invoice` (`supplier_invoice_id`),
  CONSTRAINT `fk_imported_si_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_imported_si_supplier_invoice` FOREIGN KEY (`supplier_invoice_id`) REFERENCES `supplier_invoices` (`id`) ON DELETE SET NULL,
  CONSTRAINT `chk_imported_si_status` CHECK (`status` in ('to_complete','completed','discarded')),
  CONSTRAINT `chk_imported_si_address_type` CHECK (`creditor_address_type` in ('K','S')),
  CONSTRAINT `chk_imported_si_reference_type` CHECK (`reference_type` in ('QRR','SCOR','NON')),
  CONSTRAINT `chk_imported_si_byte_size_positive` CHECK (`byte_size` > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `invoice_lines`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `invoice_lines` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `invoice_id` bigint(20) NOT NULL,
  `position` int(11) NOT NULL,
  `description` varchar(1000) NOT NULL,
  `quantity` decimal(19,4) NOT NULL,
  `unit_price` decimal(19,4) NOT NULL,
  `vat_rate` decimal(5,2) NOT NULL,
  `line_total` decimal(19,4) NOT NULL,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `revenue_account_id` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_invoice_lines_position` (`invoice_id`,`position`),
  KEY `idx_invoice_lines_invoice` (`invoice_id`),
  KEY `idx_invoice_lines_revenue_account` (`revenue_account_id`),
  CONSTRAINT `fk_invoice_lines_invoice` FOREIGN KEY (`invoice_id`) REFERENCES `invoices` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_invoice_lines_revenue_account` FOREIGN KEY (`revenue_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `chk_invoice_lines_quantity_positive` CHECK (`quantity` > 0),
  CONSTRAINT `chk_invoice_lines_unit_price_non_negative` CHECK (`unit_price` >= 0),
  CONSTRAINT `chk_invoice_lines_vat_rate_range` CHECK (`vat_rate` >= 0 and `vat_rate` <= 100),
  CONSTRAINT `chk_invoice_lines_description_not_empty` CHECK (char_length(trim(`description`)) > 0),
  CONSTRAINT `chk_invoice_lines_line_total_non_negative` CHECK (`line_total` >= 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `invoice_number_sequences`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `invoice_number_sequences` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `fiscal_year_id` bigint(20) NOT NULL,
  `next_number` bigint(20) NOT NULL DEFAULT 1,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_ins_company_fy` (`company_id`,`fiscal_year_id`),
  KEY `fk_ins_fiscal_year` (`fiscal_year_id`),
  CONSTRAINT `fk_ins_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_ins_fiscal_year` FOREIGN KEY (`fiscal_year_id`) REFERENCES `fiscal_years` (`id`),
  CONSTRAINT `chk_ins_next_positive` CHECK (`next_number` >= 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `invoice_reminders`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `invoice_reminders` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `invoice_id` bigint(20) NOT NULL,
  `level_number` smallint(6) NOT NULL,
  `fee_amount` decimal(7,2) NOT NULL,
  `sent_at` datetime(6) NOT NULL,
  `channel` varchar(16) NOT NULL,
  `sent_to` varchar(320) DEFAULT NULL,
  `subject` text NOT NULL,
  `body` text NOT NULL,
  `note` text DEFAULT NULL,
  `actor_user_id` bigint(20) DEFAULT NULL,
  `cancelled_at` datetime(6) DEFAULT NULL,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  PRIMARY KEY (`id`),
  KEY `idx_invoice_reminders_company_invoice` (`company_id`,`invoice_id`),
  KEY `idx_invoice_reminders_invoice` (`invoice_id`),
  CONSTRAINT `fk_invoice_reminders_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_invoice_reminders_invoice` FOREIGN KEY (`invoice_id`) REFERENCES `invoices` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chk_invoice_reminders_level_positive` CHECK (`level_number` >= 1),
  CONSTRAINT `chk_invoice_reminders_fee_range` CHECK (`fee_amount` >= 0 and `fee_amount` <= 10000),
  CONSTRAINT `chk_invoice_reminders_channel` CHECK (`channel` in ('email','manual'))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `invoice_settlements`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `invoice_settlements` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `invoice_id` bigint(20) NOT NULL,
  `journal_entry_id` bigint(20) NOT NULL,
  `amount` decimal(19,4) NOT NULL,
  `settled_on` date NOT NULL,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `settlement_type` varchar(20) NOT NULL DEFAULT 'bank_transfer',
  `settlement_bank_account_id` bigint(20) DEFAULT NULL,
  `settlement_account_id` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_invoice_settlements_entry` (`journal_entry_id`),
  KEY `idx_invoice_settlements_company_invoice` (`company_id`,`invoice_id`),
  KEY `idx_invoice_settlements_invoice` (`invoice_id`),
  KEY `fk_invoice_settlements_settlement_bank` (`settlement_bank_account_id`),
  KEY `fk_invoice_settlements_settlement_account` (`settlement_account_id`),
  CONSTRAINT `fk_invoice_settlements_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_invoice_settlements_entry` FOREIGN KEY (`journal_entry_id`) REFERENCES `journal_entries` (`id`),
  CONSTRAINT `fk_invoice_settlements_invoice` FOREIGN KEY (`invoice_id`) REFERENCES `invoices` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_invoice_settlements_settlement_account` FOREIGN KEY (`settlement_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `fk_invoice_settlements_settlement_bank` FOREIGN KEY (`settlement_bank_account_id`) REFERENCES `bank_accounts` (`id`),
  CONSTRAINT `chk_invoice_settlements_amount_positive` CHECK (`amount` > 0),
  CONSTRAINT `chk_invoice_settlements_type` CHECK (`settlement_type` in ('bank_transfer','internal_account')),
  CONSTRAINT `chk_invoice_settlements_counterparty` CHECK (`settlement_type` = 'bank_transfer' and `settlement_bank_account_id` is not null and `settlement_account_id` is null or `settlement_type` = 'internal_account' and `settlement_account_id` is not null and `settlement_bank_account_id` is null)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `invoices`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `invoices` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `contact_id` bigint(20) NOT NULL,
  `invoice_number` varchar(64) DEFAULT NULL,
  `status` varchar(16) NOT NULL DEFAULT 'draft',
  `date` date NOT NULL,
  `due_date` date DEFAULT NULL,
  `payment_terms` varchar(255) DEFAULT NULL,
  `total_amount` decimal(19,4) NOT NULL DEFAULT 0.0000,
  `journal_entry_id` bigint(20) DEFAULT NULL,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `paid_at` datetime(3) DEFAULT NULL,
  `project_id` bigint(20) DEFAULT NULL,
  `emailed_at` datetime(6) DEFAULT NULL,
  `emailed_to` varchar(320) DEFAULT NULL,
  `dunning_paused_at` datetime(6) DEFAULT NULL,
  `dunning_paused_note` varchar(500) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_invoices_number` (`company_id`,`invoice_number`),
  KEY `idx_invoices_company_status` (`company_id`,`status`),
  KEY `idx_invoices_company_date` (`company_id`,`date`),
  KEY `idx_invoices_contact` (`contact_id`),
  KEY `fk_invoices_journal_entry` (`journal_entry_id`),
  KEY `idx_invoices_payment_status` (`company_id`,`status`,`paid_at`),
  KEY `idx_invoices_due_date` (`company_id`,`status`,`due_date`),
  KEY `idx_invoices_company_validated_unpaid_date` (`company_id`,`status`,`paid_at`,`date`),
  KEY `idx_invoices_project` (`project_id`),
  CONSTRAINT `fk_invoices_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_invoices_contact` FOREIGN KEY (`contact_id`) REFERENCES `contacts` (`id`),
  CONSTRAINT `fk_invoices_journal_entry` FOREIGN KEY (`journal_entry_id`) REFERENCES `journal_entries` (`id`),
  CONSTRAINT `fk_invoices_project` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`),
  CONSTRAINT `chk_invoices_status` CHECK (`status` in ('draft','validated','cancelled')),
  CONSTRAINT `chk_invoices_total_non_negative` CHECK (`total_amount` >= 0),
  CONSTRAINT `chk_invoices_validated_has_je` CHECK (`status` <> 'validated' or `journal_entry_id` is not null),
  CONSTRAINT `chk_invoices_paid_at_validated` CHECK (`paid_at` is null or `status` in ('validated','cancelled')),
  CONSTRAINT `chk_invoices_paid_at_after_date` CHECK (`paid_at` is null or cast(`paid_at` as date) >= `date` - interval 1 day)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `journal_entries`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `journal_entries` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `fiscal_year_id` bigint(20) NOT NULL,
  `entry_number` bigint(20) NOT NULL COMMENT 'Séquentiel par (company_id, fiscal_year_id), jamais de trou. BIGINT pour instances multi-décennies.',
  `entry_date` date NOT NULL,
  `journal` varchar(10) NOT NULL COMMENT 'Achats|Ventes|Banque|Caisse|OD',
  `description` varchar(500) NOT NULL,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `reverses_entry_id` bigint(20) DEFAULT NULL COMMENT 'Écriture que celle-ci contre-passe. NULL = écriture ordinaire.',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_journal_entries_number` (`company_id`,`fiscal_year_id`,`entry_number`),
  UNIQUE KEY `uq_journal_entries_reverses` (`reverses_entry_id`),
  KEY `idx_journal_entries_company_date` (`company_id`,`entry_date` DESC),
  KEY `idx_journal_entries_fiscal_year` (`fiscal_year_id`),
  FULLTEXT KEY `ft_journal_entries_description` (`description`),
  CONSTRAINT `fk_journal_entries_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_journal_entries_fiscal_year` FOREIGN KEY (`fiscal_year_id`) REFERENCES `fiscal_years` (`id`),
  CONSTRAINT `fk_journal_entries_reverses` FOREIGN KEY (`reverses_entry_id`) REFERENCES `journal_entries` (`id`),
  CONSTRAINT `chk_journal_entries_journal` CHECK (cast(`journal` as char charset binary) in (cast('Achats' as char charset binary),cast('Ventes' as char charset binary),cast('Banque' as char charset binary),cast('Caisse' as char charset binary),cast('OD' as char charset binary))),
  CONSTRAINT `chk_journal_entries_description_nonempty` CHECK (char_length(trim(`description`)) > 0),
  CONSTRAINT `chk_journal_entries_entry_number_positive` CHECK (`entry_number` > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `journal_entry_lines`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `journal_entry_lines` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `entry_id` bigint(20) NOT NULL,
  `account_id` bigint(20) NOT NULL,
  `line_order` int(11) NOT NULL COMMENT 'Position dans l''écriture (1, 2, 3...)',
  `debit` decimal(19,4) NOT NULL DEFAULT 0.0000,
  `credit` decimal(19,4) NOT NULL DEFAULT 0.0000,
  `project_id` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_jel_entry_order` (`entry_id`,`line_order`),
  KEY `idx_jel_entry` (`entry_id`),
  KEY `idx_jel_account` (`account_id`),
  KEY `idx_jel_project` (`project_id`),
  CONSTRAINT `fk_jel_account` FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `fk_jel_entry` FOREIGN KEY (`entry_id`) REFERENCES `journal_entries` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_jel_project` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`),
  CONSTRAINT `chk_jel_debit_credit_exclusive` CHECK (`debit` = 0 and `credit` > 0 or `debit` > 0 and `credit` = 0),
  CONSTRAINT `chk_jel_debit_nonneg` CHECK (`debit` >= 0),
  CONSTRAINT `chk_jel_credit_nonneg` CHECK (`credit` >= 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `onboarding_state`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `onboarding_state` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `singleton` tinyint(1) NOT NULL DEFAULT 1 COMMENT 'Sentinelle UNIQUE — garantit une seule row',
  `step_completed` int(11) NOT NULL DEFAULT 0 COMMENT '0=pas commencé, 1=langue choisie, 2=mode choisi, 3=chemin choisi (démo ou prod), 4-10 réservés Chemin B (story 2-3)',
  `is_demo` tinyint(1) NOT NULL DEFAULT 0,
  `ui_mode` varchar(10) DEFAULT NULL COMMENT 'guided|expert — NULL tant que pas choisi',
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_onboarding_singleton` (`singleton`),
  CONSTRAINT `chk_onboarding_step` CHECK (`step_completed` between 0 and 10),
  CONSTRAINT `chk_onboarding_ui_mode` CHECK (`ui_mode` is null or cast(`ui_mode` as char charset binary) in (cast('guided' as char charset binary),cast('expert' as char charset binary))),
  CONSTRAINT `chk_onboarding_singleton` CHECK (`singleton` = 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `password_reset_tokens`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `password_reset_tokens` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `user_id` bigint(20) NOT NULL,
  `token_hash` char(64) NOT NULL,
  `expires_at` datetime(3) NOT NULL,
  `used_at` datetime(3) DEFAULT NULL,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_prt_token_hash` (`token_hash`),
  KEY `idx_prt_user` (`user_id`),
  KEY `idx_prt_expires` (`expires_at`),
  CONSTRAINT `fk_prt_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `payment_batch_items`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `payment_batch_items` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `payment_batch_id` bigint(20) NOT NULL,
  `supplier_invoice_id` bigint(20) NOT NULL,
  `position` int(11) NOT NULL,
  `end_to_end_id` varchar(35) NOT NULL,
  `amount` decimal(19,4) NOT NULL,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_payment_batch_items_batch_invoice` (`payment_batch_id`,`supplier_invoice_id`),
  UNIQUE KEY `uq_payment_batch_items_position` (`payment_batch_id`,`position`),
  KEY `idx_payment_batch_items_invoice` (`supplier_invoice_id`),
  CONSTRAINT `fk_payment_batch_items_batch` FOREIGN KEY (`payment_batch_id`) REFERENCES `payment_batches` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_payment_batch_items_supplier_invoice` FOREIGN KEY (`supplier_invoice_id`) REFERENCES `supplier_invoices` (`id`),
  CONSTRAINT `chk_payment_batch_items_amount_positive` CHECK (`amount` > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `payment_batches`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `payment_batches` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `bank_account_id` bigint(20) NOT NULL,
  `status` varchar(16) NOT NULL DEFAULT 'generated',
  `requested_execution_date` date NOT NULL,
  `total_amount` decimal(19,4) NOT NULL DEFAULT 0.0000,
  `msg_id` varchar(35) NOT NULL,
  `payment_info_id` varchar(35) NOT NULL,
  `confirmed_at` datetime(3) DEFAULT NULL,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  KEY `fk_payment_batches_bank_account` (`bank_account_id`),
  KEY `idx_payment_batches_company_status` (`company_id`,`status`),
  KEY `idx_payment_batches_company_date` (`company_id`,`created_at`),
  CONSTRAINT `fk_payment_batches_bank_account` FOREIGN KEY (`bank_account_id`) REFERENCES `bank_accounts` (`id`),
  CONSTRAINT `fk_payment_batches_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `chk_payment_batches_status` CHECK (`status` in ('generated','confirmed','cancelled')),
  CONSTRAINT `chk_payment_batches_total_non_negative` CHECK (`total_amount` >= 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `products`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `products` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `name` varchar(255) NOT NULL,
  `description` varchar(1000) DEFAULT NULL,
  `unit_price` decimal(19,4) NOT NULL,
  `vat_rate` decimal(5,2) NOT NULL,
  `active` tinyint(1) NOT NULL DEFAULT 1,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `default_revenue_account_id` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_products_company_name` (`company_id`,`name`),
  KEY `idx_products_company_active` (`company_id`,`active`),
  KEY `idx_products_default_revenue_account` (`default_revenue_account_id`),
  FULLTEXT KEY `ft_products_name` (`name`),
  FULLTEXT KEY `ft_products_description` (`description`),
  CONSTRAINT `fk_products_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_products_default_revenue_account` FOREIGN KEY (`default_revenue_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `chk_products_name_not_empty` CHECK (char_length(trim(`name`)) > 0),
  CONSTRAINT `chk_products_price_non_negative` CHECK (`unit_price` >= 0),
  CONSTRAINT `chk_products_price_upper_bound` CHECK (`unit_price` <= 1000000000),
  CONSTRAINT `chk_products_vat_rate_range` CHECK (`vat_rate` >= 0 and `vat_rate` <= 100)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `projects`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `projects` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `parent_id` bigint(20) DEFAULT NULL,
  `code` varchar(32) NOT NULL,
  `name` varchar(150) NOT NULL,
  `description` text DEFAULT NULL,
  `archived` tinyint(1) NOT NULL DEFAULT 0,
  `start_date` date DEFAULT NULL,
  `end_date` date DEFAULT NULL,
  `version` int(11) NOT NULL DEFAULT 0,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_projects_company_code` (`company_id`,`code`),
  KEY `fk_projects_parent` (`parent_id`),
  KEY `idx_projects_company_parent` (`company_id`,`parent_id`),
  KEY `idx_projects_company_archived` (`company_id`,`archived`),
  CONSTRAINT `fk_projects_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_projects_parent` FOREIGN KEY (`parent_id`) REFERENCES `projects` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `reconciliation_rules`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `reconciliation_rules` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `label` varchar(120) NOT NULL,
  `match_type` enum('counterparty_contains','counterparty_exact','reference_contains','iban_exact') NOT NULL,
  `match_value` varchar(255) NOT NULL,
  `counterparty_account_id` bigint(20) NOT NULL,
  `priority` int(11) NOT NULL DEFAULT 100,
  `active` tinyint(1) NOT NULL DEFAULT 1,
  `applied_count` bigint(20) NOT NULL DEFAULT 0,
  `last_applied_at` datetime(3) DEFAULT NULL,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `active_uniq` varchar(255) GENERATED ALWAYS AS (if(`active`,`match_value`,NULL)) VIRTUAL,
  `default_project_id` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_reconciliation_rules_match_active` (`company_id`,`match_type`,`active_uniq`),
  KEY `fk_reconciliation_rules_account` (`counterparty_account_id`),
  KEY `idx_reconciliation_rules_company_active_priority` (`company_id`,`active`,`priority`,`id`),
  KEY `idx_reconciliation_rules_default_project` (`default_project_id`),
  CONSTRAINT `fk_reconciliation_rules_account` FOREIGN KEY (`counterparty_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `fk_reconciliation_rules_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_reconciliation_rules_default_project` FOREIGN KEY (`default_project_id`) REFERENCES `projects` (`id`),
  CONSTRAINT `chk_reconciliation_rules_label_non_empty` CHECK (char_length(trim(`label`)) > 0),
  CONSTRAINT `chk_reconciliation_rules_match_value_non_empty` CHECK (char_length(trim(`match_value`)) > 0),
  CONSTRAINT `chk_reconciliation_rules_priority_range` CHECK (`priority` between 1 and 1000),
  CONSTRAINT `chk_reconciliation_rules_applied_count_positive` CHECK (`applied_count` >= 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `refresh_tokens`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `refresh_tokens` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `user_id` bigint(20) NOT NULL,
  `token` char(36) NOT NULL COMMENT 'UUID v4 opaque',
  `expires_at` datetime(3) NOT NULL,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `revoked_at` datetime(3) DEFAULT NULL,
  `revoked_reason` varchar(32) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_refresh_tokens_token` (`token`),
  KEY `idx_refresh_tokens_user_id` (`user_id`),
  KEY `idx_refresh_tokens_expires_at` (`expires_at`),
  CONSTRAINT `fk_refresh_tokens_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chk_refresh_tokens_token_format` CHECK (`token` regexp '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'),
  CONSTRAINT `chk_refresh_tokens_revoked_reason` CHECK (`revoked_reason` in ('logout','rotation','password_change','admin_disable','theft_detected'))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `supplier_invoice_lines`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `supplier_invoice_lines` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `supplier_invoice_id` bigint(20) NOT NULL,
  `position` int(11) NOT NULL,
  `description` varchar(1000) NOT NULL,
  `quantity` decimal(19,4) NOT NULL,
  `unit_price` decimal(19,4) NOT NULL,
  `vat_rate` decimal(5,2) NOT NULL,
  `line_total` decimal(19,4) NOT NULL,
  `expense_account_id` bigint(20) NOT NULL,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_supplier_invoice_lines_position` (`supplier_invoice_id`,`position`),
  KEY `fk_supplier_invoice_lines_expense_account` (`expense_account_id`),
  KEY `idx_supplier_invoice_lines_invoice` (`supplier_invoice_id`),
  CONSTRAINT `fk_supplier_invoice_lines_expense_account` FOREIGN KEY (`expense_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `fk_supplier_invoice_lines_invoice` FOREIGN KEY (`supplier_invoice_id`) REFERENCES `supplier_invoices` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chk_supplier_invoice_lines_quantity_positive` CHECK (`quantity` > 0),
  CONSTRAINT `chk_supplier_invoice_lines_unit_price_positive` CHECK (`unit_price` > 0),
  CONSTRAINT `chk_supplier_invoice_lines_vat_rate_range` CHECK (`vat_rate` >= 0 and `vat_rate` <= 100),
  CONSTRAINT `chk_supplier_invoice_lines_line_total_positive` CHECK (`line_total` > 0),
  CONSTRAINT `chk_supplier_invoice_lines_description_not_empty` CHECK (char_length(trim(`description`)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `supplier_invoices`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `supplier_invoices` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `contact_id` bigint(20) NOT NULL,
  `supplier_invoice_number` varchar(64) DEFAULT NULL,
  `status` varchar(16) NOT NULL DEFAULT 'open',
  `invoice_date` date NOT NULL,
  `due_date` date DEFAULT NULL,
  `total_amount` decimal(19,4) NOT NULL DEFAULT 0.0000,
  `creditor_iban` varchar(34) DEFAULT NULL,
  `creditor_qr_iban` varchar(34) DEFAULT NULL,
  `payment_reference` varchar(64) DEFAULT NULL,
  `expected_payment_amount` decimal(19,4) DEFAULT NULL,
  `purchase_journal_entry_id` bigint(20) NOT NULL,
  `settlement_type` varchar(20) DEFAULT NULL,
  `settlement_bank_account_id` bigint(20) DEFAULT NULL,
  `settlement_account_id` bigint(20) DEFAULT NULL,
  `settlement_journal_entry_id` bigint(20) DEFAULT NULL,
  `paid_at` datetime(3) DEFAULT NULL,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `project_id` bigint(20) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `fk_supplier_invoices_purchase_je` (`purchase_journal_entry_id`),
  KEY `fk_supplier_invoices_settlement_bank` (`settlement_bank_account_id`),
  KEY `fk_supplier_invoices_settlement_account` (`settlement_account_id`),
  KEY `fk_supplier_invoices_settlement_je` (`settlement_journal_entry_id`),
  KEY `idx_supplier_invoices_company_status` (`company_id`,`status`),
  KEY `idx_supplier_invoices_company_date` (`company_id`,`invoice_date`),
  KEY `idx_supplier_invoices_contact` (`contact_id`),
  KEY `idx_supplier_invoices_project` (`project_id`),
  CONSTRAINT `fk_supplier_invoices_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `fk_supplier_invoices_contact` FOREIGN KEY (`contact_id`) REFERENCES `contacts` (`id`),
  CONSTRAINT `fk_supplier_invoices_project` FOREIGN KEY (`project_id`) REFERENCES `projects` (`id`),
  CONSTRAINT `fk_supplier_invoices_purchase_je` FOREIGN KEY (`purchase_journal_entry_id`) REFERENCES `journal_entries` (`id`),
  CONSTRAINT `fk_supplier_invoices_settlement_account` FOREIGN KEY (`settlement_account_id`) REFERENCES `accounts` (`id`),
  CONSTRAINT `fk_supplier_invoices_settlement_bank` FOREIGN KEY (`settlement_bank_account_id`) REFERENCES `bank_accounts` (`id`),
  CONSTRAINT `fk_supplier_invoices_settlement_je` FOREIGN KEY (`settlement_journal_entry_id`) REFERENCES `journal_entries` (`id`),
  CONSTRAINT `chk_supplier_invoices_status` CHECK (`status` in ('open','paid','cancelled')),
  CONSTRAINT `chk_supplier_invoices_settlement_type` CHECK (`settlement_type` is null or `settlement_type` in ('bank_transfer','internal_account')),
  CONSTRAINT `chk_supplier_invoices_total_non_negative` CHECK (`total_amount` >= 0),
  CONSTRAINT `chk_supplier_invoices_paid_has_settlement` CHECK (`status` <> 'paid' or `settlement_journal_entry_id` is not null and `paid_at` is not null and `settlement_type` is not null)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `users`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `users` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `username` varchar(64) NOT NULL,
  `password_hash` varchar(512) NOT NULL COMMENT 'Argon2id — format PHC string (jusqu''à 512 chars pour supporter les paramètres custom)',
  `role` varchar(20) NOT NULL COMMENT 'Admin|Comptable|Consultation',
  `active` tinyint(1) NOT NULL DEFAULT 1,
  `version` int(11) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `company_id` bigint(20) NOT NULL,
  `email` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_users_username` (`username`),
  KEY `idx_users_company_id` (`company_id`),
  KEY `idx_users_email` (`email`),
  CONSTRAINT `fk_users_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chk_users_role` CHECK (cast(`role` as char charset binary) in (cast('Admin' as char charset binary),cast('Comptable' as char charset binary),cast('Consultation' as char charset binary))),
  CONSTRAINT `chk_users_username_nonempty` CHECK (char_length(trim(`username`)) > 0),
  CONSTRAINT `chk_users_password_hash_len` CHECK (octet_length(`password_hash`) >= 20)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `vat_rates`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `vat_rates` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `company_id` bigint(20) NOT NULL,
  `label` varchar(64) NOT NULL,
  `rate` decimal(5,2) NOT NULL,
  `valid_from` date NOT NULL,
  `valid_to` date DEFAULT NULL,
  `active` tinyint(1) NOT NULL DEFAULT 1,
  `created_at` datetime(3) NOT NULL DEFAULT current_timestamp(3),
  `updated_at` datetime(3) NOT NULL DEFAULT current_timestamp(3) ON UPDATE current_timestamp(3),
  `version` int(11) NOT NULL DEFAULT 0,
  `category` varchar(32) NOT NULL DEFAULT 'custom',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_vat_rates_company_rate_valid_from` (`company_id`,`rate`,`valid_from`),
  KEY `idx_vat_rates_company_active` (`company_id`,`active`),
  KEY `idx_vat_rates_company_category_active` (`company_id`,`category`,`active`),
  CONSTRAINT `fk_vat_rates_company` FOREIGN KEY (`company_id`) REFERENCES `companies` (`id`),
  CONSTRAINT `chk_vat_rates_rate_range` CHECK (`rate` >= 0 and `rate` <= 100),
  CONSTRAINT `chk_vat_rates_label_not_empty` CHECK (char_length(trim(`label`)) > 0),
  CONSTRAINT `chk_vat_rates_dates` CHECK (`valid_to` is null or `valid_to` > `valid_from`),
  CONSTRAINT `chk_vat_rates_category_not_empty` CHECK (char_length(trim(`category`)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;


-- Ligne d'installation, RELEVÉE dans la base migrée (jamais codée en dur :
-- chaque bump P2 de kesh_version_min_required la déplace).
INSERT INTO `_kesh_version` (`id`, `kesh_version_min_required`, `kesh_version_last_applied`)
VALUES (1, '0.10.0', '0.1.0');
