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
-- Ce fichier est la source UNIQUE du seed : `.github/workflows/ci.yml` le
-- consomme au lieu d'en porter une copie. Une divergence entre les deux ne
-- rougissait nulle part — le mode d'échec que cette story combat par ailleurs.
-- *(Duplication relevée par deux lentilles en passe 1 de revue de code.)*
--
-- Contenu aligné sur `seed_accounting_company` (crates/kesh-db/src/test_fixtures.rs),
-- pour satisfaire les 154 tests de `kesh-db` qui travaillent sur la base
-- partagée et attendent :
--   - 1 société (SELECT id FROM companies LIMIT 1)
--   - ≥ 1 utilisateur Admin
--   - 1 exercice ouvert couvrant la date du jour
--   - ≥ 2 comptes actifs (journal_entries::tests::two_accounts)
--
-- L'empreinte Argon2id est celle de `admin/admin123`, pré-calculée dans
-- test_fixtures.rs — jamais un mot de passe en clair ici. (Ce seed ne crée PAS
-- l'utilisateur `changeme` du helper Rust : aucun test de la base partagée ne
-- l'exige. Le commentaire d'origine annonçait les deux — corrigé en passe 1 de
-- revue.)
--
-- ⚠️ IDEMPOTENT ET TRANSACTIONNEL, parce qu'il est fait pour être REJOUÉ et
-- qu'on ne sait pas toujours s'il l'a déjà été. La version d'origine insérait
-- sans garde : un second passage créait une deuxième société puis échouait sur
-- l'unicité de `users`, laissant une société ORPHELINE — sans admin, sans
-- exercice, sans comptes — que le `SELECT … LIMIT 1` des tests pouvait
-- atteindre. C'est l'état à demi-seedé le plus difficile à diagnostiquer.
-- *(Relevé par deux lentilles en passe 1 de revue.)*

START TRANSACTION;

INSERT INTO companies (name, address, org_type, accounting_language, instance_language)
SELECT 'CI Seed Company', 'Test Address 1\n1000 Lausanne', 'Independant', 'FR', 'FR'
WHERE NOT EXISTS (SELECT 1 FROM companies);

SET @company_id := (SELECT id FROM companies ORDER BY id LIMIT 1);

INSERT INTO users (username, password_hash, role, active, company_id)
SELECT 'admin',
       '$argon2id$v=19$m=19456,t=2,p=1$wDaFUbAJuozHKhQshibCHw$T/DeYTKABHDpW7JM5MoiQciUad5Eb81Cfvh0aUvi2Z4',
       'Admin', TRUE, @company_id
WHERE NOT EXISTS (SELECT 1 FROM users WHERE username = 'admin');

INSERT INTO fiscal_years (company_id, name, start_date, end_date, status)
SELECT @company_id, 'Exercice CI 2020-2030', '2020-01-01', '2030-12-31', 'Open'
WHERE NOT EXISTS (SELECT 1 FROM fiscal_years WHERE company_id = @company_id);

INSERT INTO accounts (company_id, number, name, account_type)
SELECT * FROM (
    SELECT @company_id AS c, '1000' AS n, 'Caisse CI'  AS l, 'Asset'     AS t UNION ALL
    SELECT @company_id,      '1100',      'Banque CI',      'Asset'             UNION ALL
    SELECT @company_id,      '2000',      'Capital CI',     'Liability'         UNION ALL
    SELECT @company_id,      '3000',      'Ventes CI',      'Revenue'           UNION ALL
    SELECT @company_id,      '4000',      'Charges CI',     'Expense'
) AS seed
WHERE NOT EXISTS (
    SELECT 1 FROM accounts a WHERE a.company_id = seed.c AND a.number = seed.n
);

COMMIT;
