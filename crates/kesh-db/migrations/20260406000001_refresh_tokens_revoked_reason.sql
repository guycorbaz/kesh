-- Story 1.6 : ajout du champ revoked_reason pour distinguer
-- les types de révocation (logout, rotation, password_change, etc.)
-- Les tokens pré-migration auront revoked_reason = NULL → traités comme logout.

ALTER TABLE refresh_tokens
    ADD COLUMN revoked_reason VARCHAR(32) NULL AFTER revoked_at,
    ADD CONSTRAINT chk_refresh_tokens_revoked_reason
        CHECK (revoked_reason IN ('logout', 'rotation', 'password_change', 'admin_disable', 'theft_detected'));
