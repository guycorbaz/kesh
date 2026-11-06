-- Story 33-1a (0.17.5, #926) — le numéro interne des factures fournisseur.
--
-- Le seul numéro d'une facture fournisseur était celui du fournisseur : facultatif,
-- et parfois identique d'un fournisseur à l'autre. Kesh attribue désormais, à la
-- validation, un numéro unique et lisible, pour classer et retrouver le papier.
--
-- Schéma : le numéro (`supplier_invoices.internal_number`), le compteur
-- (`fiscal_years.supplier_invoice_next_number`, une colonne et non une table de
-- séquence : une table neuve ferait refuser toute sauvegarde 0.17.4 au contrôle de
-- couverture, C-e331-1), le format réglable
-- (`company_invoice_settings.supplier_internal_number_format`).
--
-- Données : les factures déjà validées, payées ou annulées reçoivent un numéro, par
-- société et par année de début d'exercice, dans l'ordre
-- `fy.start_date, invoice_date, id` ; deux exercices commençant la même année se
-- suivent (C-e331-8). L'exercice est lu sur l'écriture d'achat, jamais sur la date
-- (C-e331-6). Ni `version` ni `updated_at` ne bougent (C-e331-12), et aucune ligne de
-- `journal_entries` n'est touchée : les libellés passés restent. `GREATEST(3, …)` :
-- `LPAD(1000, 3, '0')` tronquerait à `100`.
--
-- L'`UNIQUE` vient en dernier, une fois les numéros posés.
--
-- Politique (CLAUDE.md § Migration breaking policy) : NON BREAKING — `ADD COLUMN`
-- nullable ou avec défaut, `ADD UNIQUE` ; pas de bump de `kesh_version_min_required`
-- (C-e331-14). Comme toute migration, une 0.17.4 refuse de démarrer sur une base qui
-- la porte : le retour passe par la sauvegarde d'avant la mise à jour.
--
-- Idempotence : tracked-by-sqlx (erreur 1060 au rejeu du DDL).
-- Triage P7 : classe B, sentinelle `supplier_invoices.internal_number`, extrait
-- verbatim dans `src/post_restore/20261106000001_supplier_invoice_internal_number.sql`.

ALTER TABLE supplier_invoices
    ADD COLUMN internal_number VARCHAR(64) NULL AFTER supplier_invoice_number;

ALTER TABLE fiscal_years
    ADD COLUMN supplier_invoice_next_number BIGINT NOT NULL DEFAULT 1,
    ADD CONSTRAINT chk_fiscal_years_supplier_next_positive CHECK (supplier_invoice_next_number >= 1);

ALTER TABLE company_invoice_settings
    ADD COLUMN supplier_internal_number_format VARCHAR(64) NOT NULL DEFAULT 'FF-{YEAR}-{SEQ:03}',
    ADD CONSTRAINT chk_cis_supplier_format_nonempty CHECK (CHAR_LENGTH(TRIM(supplier_internal_number_format)) > 0);

UPDATE supplier_invoices si
  JOIN (
    SELECT s.id, YEAR(f.start_date) AS annee,
           ROW_NUMBER() OVER (PARTITION BY s.company_id, YEAR(f.start_date)
                              ORDER BY f.start_date, s.invoice_date, s.id) AS rang
      FROM supplier_invoices s
      JOIN journal_entries je ON je.id = s.purchase_journal_entry_id AND je.company_id = s.company_id
      JOIN fiscal_years f ON f.id = je.fiscal_year_id AND f.company_id = s.company_id
     WHERE s.status <> 'draft'
  ) n ON n.id = si.id
   SET si.internal_number = CONCAT('FF-', n.annee, '-', LPAD(n.rang, GREATEST(3, CHAR_LENGTH(n.rang)), '0')),
       si.updated_at = si.updated_at
 WHERE si.internal_number IS NULL;

UPDATE fiscal_years fy
  JOIN (
    SELECT r.fiscal_year_id, MAX(r.rang) AS dernier
      FROM (
        SELECT f.id AS fiscal_year_id,
               ROW_NUMBER() OVER (PARTITION BY s.company_id, YEAR(f.start_date)
                                  ORDER BY f.start_date, s.invoice_date, s.id) AS rang
          FROM supplier_invoices s
          JOIN journal_entries je ON je.id = s.purchase_journal_entry_id AND je.company_id = s.company_id
          JOIN fiscal_years f ON f.id = je.fiscal_year_id AND f.company_id = s.company_id
         WHERE s.status <> 'draft'
      ) r
     GROUP BY r.fiscal_year_id
  ) c ON c.fiscal_year_id = fy.id
   SET fy.supplier_invoice_next_number = GREATEST(fy.supplier_invoice_next_number, c.dernier + 1),
       fy.updated_at = fy.updated_at;

ALTER TABLE supplier_invoices
    ADD CONSTRAINT uq_supplier_invoices_internal_number UNIQUE (company_id, internal_number);
