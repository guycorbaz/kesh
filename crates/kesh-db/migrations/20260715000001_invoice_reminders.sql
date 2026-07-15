-- Migration Story 21-5a — données & éligibilité des rappels débiteurs.
-- Issue #231 (Epic 21 « Échéances & relances débiteurs »).
--
-- NON-BREAKING (CREATE TABLE + ADD COLUMN nullable — un binaire antérieur les
-- ignore) → PAS de bump `kesh_version_min_required` (politique P2/P3 CLAUDE.md).
--
-- `invoice_reminders` : historique APPEND-ONLY des rappels d'une facture. Chaque
--   ligne snapshote `level_number` + `fee_amount` + `subject` + `body` (preuve de
--   ce qui a été réclamé — acte pré-contentieux, item 11 epic). `channel` = 'email'
--   (envoi Kesh, 21-5b) ou 'manual' (rappel papier enregistré, item 12). `sent_to`
--   = destinataire e-mail réel (NULL si manuel). `cancelled_at` = annulation SOFT
--   d'un envoi accidentel (Admin, item 11) — la ligne reste (append-only) mais est
--   exclue du MAX(level_number) qui détermine le niveau courant.
--   FK `invoice_id ON DELETE CASCADE` : aligné sur la suppression définitive #219
--   (comme `invoice_lines`, seul autre enfant CASCADE de `invoices`) — les rappels
--   disparaissent avec la facture ; l'audit_log SANS FK reste la trace résiduelle
--   (`invoice.reminder_sent`/`invoice.reminder_cancelled`), comme `invoice.emailed`.
--   `actor_user_id` = pointeur logique (PAS de FK — survit à la désactivation d'un
--   user, l'audit_log porte la trace authentifiée de l'acteur).
--   PAS de FK vers `dunning_levels` : `fee_amount`/`level_number` sont des SNAPSHOTS
--   découplés (le hard-delete + renumérotation de `dunning_levels` en 21-3 est sûr
--   précisément grâce à ces snapshots) — cohérent append-only.
--
-- `invoices.dunning_paused_at` + `dunning_paused_note` : suspension par facture
--   (item 10). Une facture suspendue sort de la liste « à rappeler » mais RESTE dans
--   la balance âgée / l'échéancier (invariant anti-dissimulation). Pattern calqué sur
--   `emailed_at`/`emailed_to` (20260709000002). Nullable → non-breaking.
--
-- Idempotence (docs/migrations-idempotence-audit.md) : tracked-by-sqlx — pas de
-- IF NOT EXISTS (convention majoritaire) ; re-exécution hors sqlx échouerait
-- erreur 1050/1060. Non-breaking → pas de bump.

CREATE TABLE invoice_reminders (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    invoice_id BIGINT NOT NULL,
    level_number SMALLINT NOT NULL,
    fee_amount DECIMAL(7,2) NOT NULL,
    sent_at DATETIME(6) NOT NULL,
    channel VARCHAR(16) NOT NULL,
    sent_to VARCHAR(320) NULL,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    note TEXT NULL,
    actor_user_id BIGINT NULL,
    cancelled_at DATETIME(6) NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_invoice_reminders_invoice
        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
    CONSTRAINT fk_invoice_reminders_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT chk_invoice_reminders_level_positive CHECK (level_number >= 1),
    CONSTRAINT chk_invoice_reminders_fee_range CHECK (fee_amount >= 0 AND fee_amount <= 10000),
    CONSTRAINT chk_invoice_reminders_channel CHECK (channel IN ('email', 'manual')),
    INDEX idx_invoice_reminders_company_invoice (company_id, invoice_id),
    INDEX idx_invoice_reminders_invoice (invoice_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Suspension des rappels par facture (item 10). Pattern `emailed_at`/`emailed_to`.
ALTER TABLE invoices
    ADD COLUMN dunning_paused_at DATETIME(6) NULL,
    ADD COLUMN dunning_paused_note VARCHAR(500) NULL;
