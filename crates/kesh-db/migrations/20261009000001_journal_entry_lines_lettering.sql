-- BREAKING (P1) : relève kesh_version_min_required à 0.13.0 (dernière instruction).
-- Tout binaire publié qui passait le contrôle de version (v0.10.0 à v0.12.1) ignore le
-- lettrage : tous peuvent annuler une pièce dont l'écriture est lettrée sans dissoudre son
-- groupe, et les v0.10.0 à v0.11.1 jusqu'à modifier et supprimer des écritures manuelles.
-- Le bump les refuse au démarrage et à l'import d'une sauvegarde, en le nommant (les
-- v0.1.0 à v0.9.0 le sont déjà par min_required = 0.10.0).
--
-- Story 15-1a-i (#518) — la marque du lettrage, portée par la LIGNE (R1, C90).
--
-- Deux colonnes et non une table : une table neuve rendrait toutes les
-- sauvegardes antérieures inimportables (l'inventaire des tables d'un backup
-- doit être identique), alors qu'une colonne nullable passe le contrôle de
-- compatibilité de l'import. La clé du groupe est le plus petit `id` de ligne
-- du groupe : unique par construction, sans compteur.
--
-- ⛔ AUCUNE clé étrangère sur `lettering_key` : aucune table ne référence
-- `journal_entry_lines` (la modification d'écriture réinsère ses lignes).
-- L'intégrité est tenue par la primitive unique `letterings::create_group_in_tx`
-- / `dissolve_group_in_tx` et par un test d'invariant.
--
-- Le seul statement d'écriture est le bump `_kesh_version`, table système
-- jamais restaurée → EXEMPT_MIGRATIONS (P7). Ligne ajoutée à
-- `docs/migrations-idempotence-audit.md` (P5).
ALTER TABLE journal_entry_lines
    ADD COLUMN lettering_key BIGINT NULL
        COMMENT 'Clé du groupe de lettrage = plus petit id de ligne du groupe ; NULL = ligne ouverte',
    ADD COLUMN lettering_origin VARCHAR(10) NULL
        COMMENT 'Origine du groupe : document, reversal, manual',
    ADD CONSTRAINT chk_jel_lettering_origin
        CHECK (lettering_origin IS NULL OR lettering_origin IN ('document', 'reversal', 'manual')),
    ADD CONSTRAINT chk_jel_lettering_pair
        CHECK ((lettering_key IS NULL) = (lettering_origin IS NULL));

CREATE INDEX idx_jel_account_lettering ON journal_entry_lines (account_id, lettering_key);
CREATE INDEX idx_jel_lettering ON journal_entry_lines (lettering_key);

UPDATE _kesh_version SET kesh_version_min_required = '0.13.0' WHERE id = 1;
