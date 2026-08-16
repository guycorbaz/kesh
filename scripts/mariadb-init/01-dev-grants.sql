-- Amorçage de la MariaDB de DÉVELOPPEMENT — rejoué à CHAQUE démarrage.
--
-- Le conteneur dev monte /var/lib/mysql en tmpfs (Story 22-5, issue #251) :
-- les tables système repartent donc vierges à chaque restart, et TOUT droit
-- posé à la main disparaît avec elles. L'entrypoint MariaDB n'exécute ce
-- répertoire que lorsque le datadir est vide — ce qui, ici, est le cas à tous
-- les coups. C'est précisément ce qui rend ce fichier fiable.
--
-- Sans lui, `#[sqlx::test]` ne peut plus créer ses bases éphémères : le user
-- `kesh` créé par MARIADB_USER n'a de droits que sur la base `kesh`, et la
-- suite d'intégration entière échoue au premier CREATE DATABASE.
--
-- ⚠️ Réservé au DÉVELOPPEMENT. `docker-compose.yml` (production) ne monte pas
-- ce répertoire, et ne doit jamais le monter : des droits globaux y seraient
-- une faute de sécurité.

-- `#[sqlx::test]` crée et détruit une base par test (`_sqlx_test_database_*`),
-- plus sa base de suivi. Le README de kesh-db documente ce droit comme étape
-- d'installation manuelle ; sur tmpfs il doit être rejoué, donc automatisé.
GRANT ALL PRIVILEGES ON *.* TO 'kesh'@'%' WITH GRANT OPTION;
FLUSH PRIVILEGES;

-- Base de la suite Playwright. Le backend y applique ses migrations au
-- démarrage ; elle doit seulement exister.
CREATE DATABASE IF NOT EXISTS kesh_e2e
  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;
