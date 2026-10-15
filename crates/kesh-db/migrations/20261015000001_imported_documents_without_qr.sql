-- BREAKING (P1/P3) : relève kesh_version_min_required à 0.14.0 (dernière
-- instruction). Un binaire 0.13.x lit les sept colonnes QR de
-- `imported_supplier_invoices` dans des types non optionnels : une seule ligne
-- sans QR ferait échouer son décodage, donc la liste « à compléter » et
-- l'export CSV entiers. Le bump refuse ce binaire au démarrage et à l'import
-- d'une sauvegarde, en le nommant. Les dix crates passent à 0.14.0 dans la
-- même PR (P2-bis).
--
-- Story 30-2a (#685) — tout document PDF ou image entre dans la liste
-- « Documents à traiter », avec ou sans QR-code.
--
-- 1. Les sept colonnes que remplit le QR deviennent facultatives (MODIFY
--    COLUMN … NULL, définition complète réécrite : type d'origine, sans
--    DEFAULT). Les cinq autres colonnes du QR l'étaient déjà.
-- 2. Contrainte « tout ou rien » : les sept sont NULL ensemble (document sans
--    QR) ou renseignées ensemble (document avec QR). « Le document porte un
--    QR » se lit `creditor_iban IS NOT NULL`.
-- 3. L'index `idx_imported_supplier_invoice` devient une contrainte d'unicité
--    `uq_imported_supplier_invoice` : une facture fournisseur n'a qu'un
--    document. Plusieurs NULL restent permis. Elle ne peut pas échouer en 1062
--    sur une base peuplée : `mark_completed`, seul écrivain du lien, n'est
--    appelé que par la complétion, avec la facture qu'elle vient de créer
--    (`repositories/imported_supplier_invoices.rs`), et la réactivation d'un
--    document écarté remet le lien à NULL. L'unicité est ajoutée AVANT la
--    suppression de l'index : l'ancien index porte la clé étrangère
--    `fk_imported_si_supplier_invoice`, et MariaDB refuse de supprimer le
--    dernier index d'une clé étrangère (erreur 1553 — précédents
--    20260714000002 et 20260814000001).
--
-- Aucune écriture de données applicatives : les lignes existantes portent
-- toutes un QR et satisfont la contrainte telles quelles. Le seul statement
-- d'écriture est le bump _kesh_version, table système jamais restaurée, d'où
-- l'entrée EXEMPT_MIGRATIONS (P7). Ligne ajoutée à
-- docs/migrations-idempotence-audit.md (P5).
ALTER TABLE imported_supplier_invoices
    MODIFY COLUMN creditor_iban VARCHAR(34) NULL,
    MODIFY COLUMN is_qr_iban BOOLEAN NULL,
    MODIFY COLUMN creditor_address_type CHAR(1) NULL,
    MODIFY COLUMN creditor_name VARCHAR(70) NULL,
    MODIFY COLUMN creditor_country CHAR(2) NULL,
    MODIFY COLUMN reference_type VARCHAR(8) NULL,
    MODIFY COLUMN currency VARCHAR(3) NULL,
    ADD CONSTRAINT chk_imported_si_qr_all_or_nothing CHECK (
        (creditor_iban IS NULL) = (is_qr_iban IS NULL)
        AND (creditor_iban IS NULL) = (creditor_address_type IS NULL)
        AND (creditor_iban IS NULL) = (creditor_name IS NULL)
        AND (creditor_iban IS NULL) = (creditor_country IS NULL)
        AND (creditor_iban IS NULL) = (reference_type IS NULL)
        AND (creditor_iban IS NULL) = (currency IS NULL)
    ),
    ADD CONSTRAINT uq_imported_supplier_invoice UNIQUE (supplier_invoice_id),
    DROP INDEX idx_imported_supplier_invoice;

UPDATE _kesh_version SET kesh_version_min_required = '0.14.0' WHERE id = 1;
