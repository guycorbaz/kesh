-- Story 25-6-b (#387) — le PDF d'une facture est FIGÉ.
--
-- Le PDF d'une facture était régénéré à chaque demande : une facture
-- réimprimée dans cinq ans n'aurait pas été celle remise au client. Il est
-- désormais figé au premier rendu après validation (téléchargement ou envoi),
-- dans la langue du client, et conservé sous `KESH_DOCUMENTS_DIR` sous le nom
-- de son empreinte SHA-256. Ces quatre colonnes le désignent.
--
-- Arbitrage du Project Lead (2026-10-03) : des COLONNES sur `invoices`, pas une
-- table — une table neuve rendrait toutes les sauvegardes existantes non
-- importables (`post_restore.rs`, inventaire des tables), une colonne nullable
-- non (`check_schema_compat` n'exige que les colonnes NOT NULL sans défaut).
--
-- Contrainte TOUT-OU-RIEN : les quatre colonnes sont nulles (facture non
-- figée), ou toutes renseignées, l'empreinte faisant 64 caractères et la langue
-- étant l'une des quatre de l'instance.
--
-- DDL seul, AUCUNE donnée écrite : les factures existantes sont figées à leur
-- prochain rendu (triage P7 sans objet). Non-breaking (`ADD COLUMN` nullable,
-- ignoré par les anciens binaires, qui listent leurs colonnes) → PAS de bump
-- `kesh_version_min_required`. Ligne ajoutée à
-- `docs/migrations-idempotence-audit.md` (politique P5).

ALTER TABLE invoices
    ADD COLUMN pdf_storage_path VARCHAR(512) NULL,
    ADD COLUMN pdf_sha256 CHAR(64) NULL,
    ADD COLUMN pdf_frozen_at DATETIME(3) NULL,
    ADD COLUMN pdf_language CHAR(2) NULL,
    ADD CONSTRAINT chk_invoices_frozen_pdf CHECK (
        (pdf_storage_path IS NULL AND pdf_sha256 IS NULL AND pdf_frozen_at IS NULL AND pdf_language IS NULL)
        OR (pdf_storage_path IS NOT NULL AND pdf_sha256 IS NOT NULL AND pdf_frozen_at IS NOT NULL
            AND pdf_language IS NOT NULL
            AND CHAR_LENGTH(pdf_sha256) = 64
            AND BINARY pdf_language IN (BINARY 'FR', BINARY 'DE', BINARY 'IT', BINARY 'EN'))
    );
