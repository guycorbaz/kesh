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
-- ce répertoire, et ne doit jamais le monter.

-- Les droits sont bornés à DEUX ESPACES DE NOMS, et pas donnés sur `*.*` :
--
--   `_sqlx_test%` — les bases éphémères de `#[sqlx::test]` (une par test, plus
--                   sa base de suivi), celles du garde-fou de schéma
--                   (`_sqlx_test_guard_*`) et celles du script de régénération
--                   (`_sqlx_test_squashgen_*`). C'est ce préfixe que
--                   `scripts/regen-test-schema.sh` impose et contrôle.
--   `kesh%`       — la base de dev `kesh`, la base E2E `kesh_e2e` et les bases
--                   de gate `kesh_gate*`.
--
-- ⚠️ La première rédaction donnait `ALL PRIVILEGES ON *.* … WITH GRANT OPTION`.
-- C'était plus large que le besoin, et surtout cela rendait FAUX le
-- raisonnement écrit dans `scripts/regen-test-schema.sh`, qui justifie son
-- préfixe réservé par des droits restreints. Deux fichiers de la même story se
-- contredisaient. *(Relevé par deux lentilles en passe 1 de revue de code.)*
GRANT ALL PRIVILEGES ON `_sqlx_test%`.* TO 'kesh'@'%';
GRANT ALL PRIVILEGES ON `kesh%`.* TO 'kesh'@'%';
FLUSH PRIVILEGES;

-- Base de la suite Playwright. Le backend y applique ses migrations au
-- démarrage ; elle doit seulement exister.
--
-- Collation : `utf8mb4_general_ci`, qui est le défaut du serveur et donc celui
-- de la base `kesh`. Les tables, elles, portent chacune la leur (36 sur 38 en
-- `utf8mb4_unicode_ci`, posée par les migrations) : la collation de la BASE ne
-- gouverne que les tables futures sans `COLLATE` explicite. S'aligner sur les
-- tables ferait diverger `kesh_e2e` de `kesh`, ce qui serait pire.
CREATE DATABASE IF NOT EXISTS kesh_e2e
  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;
