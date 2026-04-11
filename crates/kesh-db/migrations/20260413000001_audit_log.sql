-- Story 3.3 : Journal d'audit minimal
-- Enregistre les actions utilisateurs sur les données comptables (FR88).
--
-- Scope initial (3.3) : journal_entry.updated, journal_entry.deleted.
-- Story 3.5 étendra avec journal_entry.created + UI de consultation.
--
-- Conformité CO art. 957-964 : l'audit doit survivre au DELETE des
-- entités auditées → pas de FK vers journal_entries.id, entity_id est
-- un pointeur logique. FK vers users.id avec ON DELETE RESTRICT
-- (conservation 10 ans obligatoire — un utilisateur référencé dans
-- l'audit ne peut pas être supprimé).

CREATE TABLE audit_log (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    action VARCHAR(64) NOT NULL COMMENT 'ex: journal_entry.updated, journal_entry.deleted',
    entity_type VARCHAR(32) NOT NULL COMMENT 'ex: journal_entry',
    entity_id BIGINT NOT NULL COMMENT 'Pointeur logique (PAS une FK) — survit aux DELETE',
    details_json JSON NULL COMMENT 'Snapshot before/after ou autre contexte',
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_audit_log_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT chk_audit_log_action_nonempty CHECK (CHAR_LENGTH(TRIM(action)) > 0),
    CONSTRAINT chk_audit_log_entity_type_nonempty CHECK (CHAR_LENGTH(TRIM(entity_type)) > 0),
    INDEX idx_audit_log_entity (entity_type, entity_id),
    INDEX idx_audit_log_user_date (user_id, created_at DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
