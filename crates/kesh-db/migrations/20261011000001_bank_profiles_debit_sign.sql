-- BREAKING (P1) : relève kesh_version_min_required à 0.13.1 (dernière instruction).
-- Une 0.13.0 ignorerait la colonne debit_sign : elle lirait un profil réglé
-- « débits déjà négatifs » (UBS) comme un profil à débits sans signe, et
-- inverserait chaque débit en silence, chaque dépense devenant un encaissement.
-- C'est le P1 #620 rouvert par un simple retour en arrière. Le bump refuse ce
-- binaire au démarrage et à l'import d'une sauvegarde, en le nommant.
--
-- Story 26-1 (#620, #621) — importer le CSV d'UBS tel quel.
--
-- 1. debit_sign : comment la banque écrit la colonne Débit, en mode
--    débit/crédit séparés. positive = sans signe, Kesh les rend négatifs
--    (comportement d'avant). negative = déjà négatifs, Kesh les garde.
--    Colonne dédiée et non champ du JSON column_mapping, que la 0.13.0
--    désérialise en deny_unknown_fields. Les profils existants prennent
--    positive par le DEFAULT, c'est-à-dire exactement leur comportement
--    d'avant.
-- 2. header_row_count : plafond relevé de 5 à 20 (le relevé UBS demande 9).
--    Les valeurs existantes (au plus 5) restent valides.
--
-- Aucune écriture de données applicatives. Le seul statement d'écriture est
-- le bump _kesh_version, table système jamais restaurée, d'où l'entrée
-- EXEMPT_MIGRATIONS (P7). Ligne ajoutée à docs/migrations-idempotence-audit.md
-- (P5).
ALTER TABLE bank_profiles
    ADD COLUMN debit_sign VARCHAR(10) NOT NULL DEFAULT 'positive'
        COMMENT 'Colonne Débit en mode séparé. positive = sans signe, Kesh inverse. negative = déjà négatif, Kesh garde'
        AFTER header_row_count,
    ADD CONSTRAINT chk_bank_profiles_debit_sign
        CHECK (debit_sign IN ('positive', 'negative'));

ALTER TABLE bank_profiles DROP CONSTRAINT chk_bank_profiles_header_row_count;

ALTER TABLE bank_profiles ADD CONSTRAINT chk_bank_profiles_header_row_count
    CHECK (header_row_count <= 20);

UPDATE _kesh_version SET kesh_version_min_required = '0.13.1' WHERE id = 1;
