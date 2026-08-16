-- Seed de la base de dev partagée `kesh` — à rejouer après chaque redémarrage
-- du conteneur MariaDB de dev, dont le datadir est en tmpfs (Story 22-5, #251).
--
-- ⚠️ NE PAS placer ce fichier dans `scripts/mariadb-init/` : l'entrypoint
-- l'exécuterait AVANT les migrations, sur des tables qui n'existent pas encore.
-- L'ordre est : `sqlx migrate run` d'abord, ce seed ensuite.
--
--   export DATABASE_URL='mysql://kesh:kesh_dev@127.0.0.1:3306/kesh'
--   sqlx migrate run --source crates/kesh-db/migrations
--   docker exec -i kesh-mariadb-dev mariadb -uroot -pkesh_dev_root kesh \
--     < scripts/seed-dev-db.sql
--
-- Contenu aligné sur l'étape « Seed CI fixtures » de `.github/workflows/ci.yml`
-- et sur `seed_accounting_company` (crates/kesh-db/src/test_fixtures.rs), pour
-- satisfaire les 154 tests `kesh-db::repositories::*` qui attendent :
--   - 1 société (SELECT id FROM companies LIMIT 1)
--   - ≥ 1 utilisateur Admin
--   - 1 exercice ouvert couvrant la date du jour
--   - ≥ 2 comptes actifs (journal_entries::tests::two_accounts)
--
-- Les empreintes Argon2id sont celles de `admin/admin123` et `changeme/changeme`
-- pré-calculées dans test_fixtures.rs — jamais un mot de passe en clair ici.

INSERT INTO companies (name, address, org_type, accounting_language, instance_language)
VALUES ('CI Seed Company', 'Test Address 1\n1000 Lausanne', 'Independant', 'FR', 'FR');
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
VALUES (@company_id, 'Exercice CI 2020-2030', '2020-01-01', '2030-12-31', 'Open');

INSERT INTO accounts (company_id, number, name, account_type) VALUES
  (@company_id, '1000', 'Caisse CI', 'Asset'),
  (@company_id, '1100', 'Banque CI', 'Asset'),
  (@company_id, '2000', 'Capital CI', 'Liability'),
  (@company_id, '3000', 'Ventes CI', 'Revenue'),
  (@company_id, '4000', 'Charges CI', 'Expense');
