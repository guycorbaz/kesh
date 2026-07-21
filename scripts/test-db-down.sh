#!/usr/bin/env bash
# Arrête et supprime la MariaDB de TEST (libère la RAM du tmpfs).
set -euo pipefail
cd "$(dirname "$0")/.."
docker compose -f docker-compose.test.yml down
echo ">>> MariaDB test arrêtée (RAM libérée)."
