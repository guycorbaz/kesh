-- Story 18-1a : Comptes TVA dans le plan comptable + configuration (Issue #180).
--
-- Fondation de la comptabilisation TVA (story-zéro Epic 18). Aucune écriture
-- comptable n'est générée ici — la migration pose uniquement :
--
--   1. Trois colonnes de configuration sur `company_invoice_settings` pour
--      désigner les comptes TVA par défaut (TVA due / récupérable / décompte),
--      chacune nullable avec FK `ON DELETE RESTRICT` (pattern `fk_cis_*` de
--      `20260417000001_invoice_validation.sql:45-47`).
--
--   2. Pour chaque company qui a DÉJÀ un plan comptable (au moins un compte), les
--      deux nouveaux comptes s'ils ne sont pas déjà présents :
--        - `1171` Impôt préalable (TVA récupérable sur achats), Asset, parent `10`.
--        - `2206` Décompte TVA (solde net dû à l'AFC), Liability, parent `20`.
--      Distincts de `1170`/`2201` (= impôt anticipé / Verrechnungssteuer, qui
--      restent INTACTS — sémantique de withholding 35 %).
--
--      GARDE « company sans plan » (`EXISTS (… accounts …)`) : on ne backfille que
--      les companies ayant au moins un compte. Une company SANS plan (typiquement
--      un stub bootstrap `auth/bootstrap.rs:insert_stub_company`, 0 compte) recevra
--      `1171`/`2206` plus tard via `bulk_create_from_chart` à l'onboarding
--      (`routes/onboarding.rs:set_accounting_language`), le chart JSON les incluant.
--      Injecter ici 2 comptes orphelins dans une company sans plan casserait le
--      garde de seed onboarding `if existing == 0` (2 comptes ≠ 0 → seed du plan
--      COMPLET sauté → company privée de 1000/2000/3000/…).
--      NB : on teste la présence du plan, PAS le flag `is_stub` : pendant
--      l'onboarding le plan est seedé (étape `set_accounting_language`) AVANT que
--      `is_stub` repasse à FALSE (étape `set_coordinates`). Une company
--      mid-onboarding (plan présent, `is_stub` encore TRUE) DOIT donc être
--      backfillée — ce que `is_stub = FALSE` aurait raté (régression évitée).
--
-- Choix de la langue du libellé : la table `companies` porte la colonne
-- `accounting_language` (CHAR(2) FR|DE|IT|EN), qui est EXACTEMENT la locale que
-- `bulk_create_from_chart` reçoit pour nommer les comptes au seed
-- (`kesh-seed/src/lib.rs:136` : `company.accounting_language.as_str().to_lowercase()`).
-- On réplique cette logique en SQL pur via un `CASE` sur `accounting_language`,
-- avec FR comme fallback défensif (les libellés restent éditables via le CRUD
-- du plan comptable de toute façon).
--
-- Résolution `parent_id` : sous-requête corrélée `number='10'|'20'` par company
-- (comme `bulk_create_from_chart`). Si le plan custom de la company n'a pas le
-- compte parent, la sous-requête renvoie NULL → compte créé orphelin
-- (`parent_id` nullable, toléré).
--
-- Idempotence : garantie par `WHERE NOT EXISTS (… number='1171'|'2206')` +
-- l'unicité `uq_accounts_company_number`. Un re-jeu ne crée pas de doublon et
-- ne touche aucun compte existant (pas d'UPDATE de `1170`/`2200`/`2201`).
--
-- Non-breaking (`ADD COLUMN` nullable + INSERT idempotent, ignorés par les
-- anciens binaires) → PAS de bump `kesh_version_min_required`. Ligne ajoutée à
-- `docs/migrations-idempotence-audit.md` (politique P5).

ALTER TABLE company_invoice_settings
    ADD COLUMN default_vat_payable_account_id BIGINT NULL,
    ADD COLUMN default_vat_recoverable_account_id BIGINT NULL,
    ADD COLUMN default_vat_decompte_account_id BIGINT NULL,
    ADD CONSTRAINT fk_cis_vat_payable
        FOREIGN KEY (default_vat_payable_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT fk_cis_vat_recoverable
        FOREIGN KEY (default_vat_recoverable_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT fk_cis_vat_decompte
        FOREIGN KEY (default_vat_decompte_account_id) REFERENCES accounts(id) ON DELETE RESTRICT;

-- Backfill data : compte 1171 Impôt préalable (Asset, parent 10) par company.
INSERT INTO accounts (company_id, number, name, account_type, parent_id, active, version)
SELECT
    c.id,
    '1171',
    CASE c.accounting_language
        WHEN 'DE' THEN 'Vorsteuer'
        WHEN 'IT' THEN 'Imposta precedente'
        WHEN 'EN' THEN 'Input VAT'
        ELSE 'Impôt préalable'
    END,
    'Asset',
    (SELECT p.id FROM accounts p WHERE p.company_id = c.id AND p.number = '10'),
    TRUE,
    1
FROM companies c
WHERE EXISTS (SELECT 1 FROM accounts a2 WHERE a2.company_id = c.id)
  AND NOT EXISTS (
    SELECT 1 FROM accounts a WHERE a.company_id = c.id AND a.number = '1171'
);

-- Backfill data : compte 2206 Décompte TVA (Liability, parent 20) par company.
INSERT INTO accounts (company_id, number, name, account_type, parent_id, active, version)
SELECT
    c.id,
    '2206',
    CASE c.accounting_language
        WHEN 'DE' THEN 'MWST-Abrechnung'
        WHEN 'IT' THEN 'Rendiconto IVA'
        WHEN 'EN' THEN 'VAT settlement'
        ELSE 'Décompte TVA'
    END,
    'Liability',
    (SELECT p.id FROM accounts p WHERE p.company_id = c.id AND p.number = '20'),
    TRUE,
    1
FROM companies c
WHERE EXISTS (SELECT 1 FROM accounts a2 WHERE a2.company_id = c.id)
  AND NOT EXISTS (
    SELECT 1 FROM accounts a WHERE a.company_id = c.id AND a.number = '2206'
);
