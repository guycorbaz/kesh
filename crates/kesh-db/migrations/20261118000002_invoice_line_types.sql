-- Story 49-1a (#1242) — le TYPE d'une ligne de facture client et d'avoir.
--
-- Une ligne est désormais `priced` (chiffrée : la seule forme d'avant), `comment`
-- (texte seul) ou `heading` (titre de section). Une ligne non chiffrée porte des
-- valeurs NEUTRES (quantité, prix, taux et total à 0, compte NULL) : elle ne
-- contribue à aucune somme, et les colonnes de montant restent NOT NULL, de sorte
-- qu'un binaire 0.19.0 lit encore toute ligne écrite par la 0.20.0.
--
-- (a) NON BREAKING au sens de P1 (`CLAUDE.md` § Migration breaking policy) :
--     « sans relever `kesh_version_min_required` et sans corrompre le schéma ».
--     Une colonne ajoutée à DEFAULT, des CHECK ajoutés et un CHECK retiré : un
--     binaire antérieur liste ses colonnes (il ignore `line_type`) et
--     `check_schema_compat` n'exige d'une sauvegarde que les colonnes NOT NULL
--     sans défaut. `kesh_version_min_required` reste 0.17.6.
-- (b) `DROP CONSTRAINT` n'est pas une opération de P3 (DROP TABLE, DROP COLUMN,
--     RENAME, MODIFY / CHANGE COLUMN) : pas de bump. Précédent :
--     `20261001000004_invoice_settlements_write_off.sql`.
-- (c) LE SQL QUI DÉFAIT CETTE MIGRATION (P1), sur une base où aucune ligne n'est
--     d'un autre type que `priced` — à vérifier d'abord, chacune des deux
--     requêtes doit rendre 0 :
--       SELECT COUNT(*) FROM invoice_lines WHERE line_type <> 'priced';
--       SELECT COUNT(*) FROM credit_note_lines WHERE line_type <> 'priced';
--     puis, dans cet ordre (les contraintes d'abord : la contrainte neutre est
--     multi-colonnes et référence `line_type`) :
--       ALTER TABLE invoice_lines DROP CONSTRAINT chk_invoice_lines_unpriced_neutral, DROP CONSTRAINT chk_invoice_lines_priced_quantity, DROP CONSTRAINT chk_invoice_lines_line_type;
--       ALTER TABLE invoice_lines DROP COLUMN line_type, ADD CONSTRAINT chk_invoice_lines_quantity_positive CHECK (quantity > 0);
--       ALTER TABLE credit_note_lines DROP CONSTRAINT chk_credit_note_lines_unpriced_neutral, DROP CONSTRAINT chk_credit_note_lines_priced_quantity, DROP CONSTRAINT chk_credit_note_lines_line_type;
--       ALTER TABLE credit_note_lines DROP COLUMN line_type, ADD CONSTRAINT chk_credit_note_lines_quantity_positive CHECK (quantity > 0);
--       DELETE FROM _sqlx_migrations WHERE version = 20261118000002;
--     Le retour ordinaire reste la sauvegarde d'avant la mise à jour.
-- (d) Règle de typage des lignes existantes : TOUTES `priced`, par le DEFAULT de
--     la colonne, sans aucun UPDATE. L'origine d'une ligne (choisie dans le
--     catalogue ou saisie à la main) n'est pas établissable : aucune colonne
--     d'article n'a jamais existé, l'écran recopie le nom, le prix et le compte
--     de l'article sur une ligne ordinaire. Deviner un type ferait un faux dans
--     une facture déjà remise. Le DEFAULT est aussi ce que reçoit, à l'import,
--     une sauvegarde 0.19.0 (sans la colonne) : même valeur, montée en place ou
--     importée. DDL seul : aucun triage P7.
-- (e) Pas de type « sous-total » : son sens (HT ou TTC, ventilation par taux,
--     calculé au rendu ou figé) reste ouvert (#1330) ; une valeur dans le CHECK
--     le figerait en ligne stockée à valeurs neutres. L'ajouter plus tard ne
--     demande qu'un DROP / ADD CONSTRAINT de `chk_*_line_type` et de
--     `chk_*_unpriced_neutral`, non breaking.
--
-- Idempotence (P5) : un seul ALTER TABLE par table, formes IF [NOT] EXISTS ; le
-- DDL MariaDB n'étant pas transactionnel, un échec entre les deux ALTER laisse
-- la première table migrée et la migration non inscrite ; le rejeu est alors
-- sans effet sur elle (`docs/migrations-idempotence-audit.md`, verdict `yes`).
-- Les contraintes inchangées : unit_price >= 0, plage de vat_rate,
-- line_total >= 0, description non vide, unicité de la position.

ALTER TABLE invoice_lines
    ADD COLUMN IF NOT EXISTS line_type VARCHAR(16) NOT NULL DEFAULT 'priced',
    DROP CONSTRAINT IF EXISTS chk_invoice_lines_quantity_positive,
    ADD CONSTRAINT IF NOT EXISTS chk_invoice_lines_line_type
        CHECK (line_type IN ('priced', 'comment', 'heading')),
    ADD CONSTRAINT IF NOT EXISTS chk_invoice_lines_priced_quantity
        CHECK (line_type <> 'priced' OR quantity > 0),
    ADD CONSTRAINT IF NOT EXISTS chk_invoice_lines_unpriced_neutral
        CHECK (line_type = 'priced' OR (quantity = 0 AND unit_price = 0
            AND vat_rate = 0 AND line_total = 0 AND revenue_account_id IS NULL));

ALTER TABLE credit_note_lines
    ADD COLUMN IF NOT EXISTS line_type VARCHAR(16) NOT NULL DEFAULT 'priced',
    DROP CONSTRAINT IF EXISTS chk_credit_note_lines_quantity_positive,
    ADD CONSTRAINT IF NOT EXISTS chk_credit_note_lines_line_type
        CHECK (line_type IN ('priced', 'comment', 'heading')),
    ADD CONSTRAINT IF NOT EXISTS chk_credit_note_lines_priced_quantity
        CHECK (line_type <> 'priced' OR quantity > 0),
    ADD CONSTRAINT IF NOT EXISTS chk_credit_note_lines_unpriced_neutral
        CHECK (line_type = 'priced' OR (quantity = 0 AND unit_price = 0
            AND vat_rate = 0 AND line_total = 0 AND revenue_account_id IS NULL));
