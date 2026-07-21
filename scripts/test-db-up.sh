#!/usr/bin/env bash
# Démarre la MariaDB de TEST en RAM (docker-compose.test.yml, port 3307) puis
# applique migrations + seed à la base `kesh` — pour que la suite complète
# passe (y compris les ~84 tests kesh-db legacy qui se connectent directement
# à DATABASE_URL et attendent une base migrée + seedée, cf. ci.yml).
#
# Les tests `#[sqlx::test]` (bases éphémères `_sqlx_test_*`) n'ont PAS besoin de
# cette prépa — ils se migrent seuls — mais ça ne les gêne pas.
#
# Usage :
#   scripts/test-db-up.sh
#   DATABASE_URL="mysql://kesh:kesh_dev@127.0.0.1:3307/kesh" scripts/test-fast.sh --no-lint
#   scripts/test-db-down.sh          # ou : docker compose -f docker-compose.test.yml down
set -euo pipefail
cd "$(dirname "$0")/.."

COMPOSE="docker compose -f docker-compose.test.yml"
CONT=kesh-mariadb-test

echo ">>> Démarrage MariaDB test (tmpfs, port 3307)…"
$COMPOSE up -d

echo -n ">>> Attente healthy"
for _ in $(seq 1 40); do
  if [ "$(docker inspect -f '{{.State.Health.Status}}' "$CONT" 2>/dev/null)" = "healthy" ]; then
    echo " ✓"
    break
  fi
  echo -n "."
  sleep 2
done

echo ">>> Application des 51 migrations à la base kesh…"
for m in crates/kesh-db/migrations/*.sql; do
  docker exec -i "$CONT" mariadb -uroot -pkesh_dev_root kesh < "$m"
done

echo ">>> Seed (company + admin + fiscal_year + accounts)…"
docker exec -i "$CONT" mariadb -uroot -pkesh_dev_root kesh < scripts/test-db-seed.sql

echo ">>> Prête : DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh"
