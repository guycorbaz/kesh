-- Init de la MariaDB de TEST (docker-compose.test.yml).
-- Élargit les droits de l'utilisateur applicatif `kesh` à *.* pour que le
-- harness `#[sqlx::test]` puisse créer/détruire ses bases éphémères
-- `_sqlx_test_*` (isolation par test). Sans ce grant : ERROR 1044 Access denied.
-- Rejoué à chaque démarrage (datadir en tmpfs → init à chaque up).
GRANT ALL PRIVILEGES ON *.* TO 'kesh'@'%' WITH GRANT OPTION;
FLUSH PRIVILEGES;
