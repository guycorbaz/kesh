-- Seed minimal pour les tests kesh-db « legacy » (pré-Story-6-4) qui se
-- connectent DIRECTEMENT à DATABASE_URL (base `kesh`) au lieu d'utiliser
-- `#[sqlx::test]`. Ils attendent une base MIGRÉE + un jeu minimal :
--   - 1 company               (SELECT id FROM companies LIMIT 1)
--   - ≥1 admin user           (SELECT id FROM users WHERE role='Admin' LIMIT 1)
--   - 1 fiscal_year Open       couvrant les dates de test
--   - ≥2 accounts actifs       (journal_entries::tests::two_accounts)
--
-- ⚠️ Source de vérité alignée sur .github/workflows/ci.yml (step « Seed CI
-- fixtures ») + crates/kesh-db/src/test_fixtures.rs (hash argon2id admin/admin123).
-- Si une migration change le schéma de `companies`/`users`, mettre à jour les
-- DEUX (ce fichier ET ci.yml) — la CI le détectera sinon.
INSERT INTO companies (name, address, org_type, accounting_language, instance_language)
VALUES ('Test Seed Company', 'Test Address 1\n1000 Lausanne', 'Independant', 'FR', 'FR');
SET @company_id := LAST_INSERT_ID();

INSERT INTO users (username, password_hash, role, active, company_id)
VALUES (
  'admin',
  '$argon2id$v=19$m=19456,t=2,p=1$wDaFUbAJuozHKhQshibCHw$T/DeYTKABHDpW7JM5MoiQciUad5Eb81Cfvh0aUvi2Z4',
  'Admin',
  TRUE,
  @company_id
);

INSERT INTO fiscal_years (company_id, name, start_date, end_date, status)
VALUES (@company_id, 'Exercice test 2020-2030', '2020-01-01', '2030-12-31', 'Open');

INSERT INTO accounts (company_id, number, name, account_type) VALUES
  (@company_id, '1000', 'Caisse', 'Asset'),
  (@company_id, '1100', 'Banque', 'Asset'),
  (@company_id, '2000', 'Capital', 'Liability'),
  (@company_id, '3000', 'Ventes', 'Revenue'),
  (@company_id, '4000', 'Charges', 'Expense');
