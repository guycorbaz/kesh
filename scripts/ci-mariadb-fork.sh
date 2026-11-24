#!/usr/bin/env bash
# ci-mariadb-fork.sh — démarre MariaDB par `docker run` pour la CI d'une PR de fork
# (C-31-2-3, variante B). Sur `main` et les branches du dépôt, MariaDB reste un
# `services:` tiré de Docker Hub avec `credentials` ; sur un fork, ces `credentials`
# vaudraient `''`, que le modèle du job peut refuser (« Unexpected value '' »)
# avant toute étape : le service y est donc désactivé (image vide) et remplacé
# par ce conteneur, tiré sans authentification du miroir public ECR.
#
# Même image (10.11), mêmes variables, même port et même contrôle de santé que le
# `services:` des jobs `backend` et `e2e-smoke` de `.github/workflows/ci.yml` —
# à tenir égaux à la main, la garde de `scripts/tests/ci-mariadb-fork.test.sh`
# compare les valeurs écrites ici à celles de `ci.yml`.
#
# Rend 0 quand le conteneur est « healthy », 1 s'il devient « unhealthy », s'arrête
# ou n'est pas prêt dans le délai (journal du conteneur sur la sortie d'erreur).
#
# Surcharges, pour les tests seulement : DOCKER (commande), KESH_CI_MARIADB_PORT,
# KESH_CI_MARIADB_NOM, KESH_CI_MARIADB_ESSAIS, KESH_CI_MARIADB_PAUSE.

set -euo pipefail

DOCKER=${DOCKER:-docker}
IMAGE=public.ecr.aws/docker/library/mariadb:10.11
PORT=${KESH_CI_MARIADB_PORT:-3306}
NOM=${KESH_CI_MARIADB_NOM:-mariadb}
ESSAIS=${KESH_CI_MARIADB_ESSAIS:-90}
PAUSE=${KESH_CI_MARIADB_PAUSE:-2}

$DOCKER run -d --name "$NOM" -p "$PORT:3306" \
  -e MARIADB_ROOT_PASSWORD=kesh_root \
  -e MARIADB_DATABASE=kesh \
  -e MARIADB_USER=kesh \
  -e MARIADB_PASSWORD=kesh_dev \
  --health-cmd="healthcheck.sh --connect --innodb_initialized" \
  --health-interval=10s \
  --health-timeout=5s \
  --health-retries=10 \
  "$IMAGE" >/dev/null

for ((i = 1; i <= ESSAIS; i++)); do
  etat=$($DOCKER inspect -f '{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}' "$NOM" 2>/dev/null || echo absent)
  case "$etat" in
    healthy)
      echo "MariaDB prête ($IMAGE, port $PORT, essai $i)"
      exit 0 ;;
    unhealthy|exited|dead|absent)
      break ;;
  esac
  sleep "$PAUSE"
done

echo "MariaDB non prête (état : ${etat:-inconnu}) ; journal du conteneur :" >&2
$DOCKER logs "$NOM" >&2 || true
exit 1
