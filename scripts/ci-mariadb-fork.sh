#!/usr/bin/env bash
# ci-mariadb-fork.sh — démarre MariaDB par `docker run` pour la CI d'une PR de fork
# (C-31-2-3, variante B). Sur `main` et les branches du dépôt, MariaDB reste un
# `services:` tiré de Docker Hub avec `credentials` ; sur un fork, ces `credentials`
# vaudraient `''`, que le modèle du job peut refuser (« Unexpected value '' »)
# avant toute étape : le service y est donc désactivé (image vide) et remplacé
# par ce conteneur, tiré sans authentification.
#
# Le tirage suit une chaîne de miroirs, dans cet ordre, chacun repris quelques fois
# avec un délai croissant mais borné : le miroir public ECR, `mirror.gcr.io`, puis
# Docker Hub en anonyme. Les trois servaient la même image au relevé du 2026-11-25
# (même empreinte d'index, docs/ci.md) ; l'étiquette `10.11` est mouvante, et un
# miroir peut servir un temps un correctif plus ancien que Docker Hub. La reprise vient du run 45525662619 (PR #1560) :
# ECR y a refusé le tirage anonyme (« toomanyrequests: Data limit exceeded »), un
# quota intermittent, alors que le job `backend` avait obtenu l'image peu avant.
#
# Même image (10.11), mêmes variables, même port et même contrôle de santé que le
# `services:` des jobs `backend` et `e2e-smoke` de `.github/workflows/ci.yml` —
# à tenir égaux à la main, la garde de `scripts/tests/ci-mariadb-fork.test.sh`
# compare les valeurs écrites ici à celles de `ci.yml`.
#
# Rend 0 quand le conteneur est « healthy » ; 1 si aucun miroir ne sert l'image, ou si
# le conteneur devient « unhealthy », s'arrête ou n'est pas prêt dans le délai (journal
# du conteneur sur la sortie d'erreur) ; le code de `docker` si `docker run` échoue.
#
# Surcharges, pour les tests seulement : DOCKER (commande), KESH_CI_MARIADB_PORT,
# KESH_CI_MARIADB_NOM, KESH_CI_MARIADB_ESSAIS, KESH_CI_MARIADB_PAUSE,
# KESH_CI_MARIADB_TIRAGES (essais par miroir), KESH_CI_MARIADB_REPRISE (premier délai
# de reprise, en secondes ; doublé à chaque essai, borné par KESH_CI_MARIADB_REPRISE_MAX).

set -euo pipefail

DOCKER=${DOCKER:-docker}
MIROIRS=(
  public.ecr.aws/docker/library/mariadb:10.11
  mirror.gcr.io/library/mariadb:10.11
  docker.io/library/mariadb:10.11
)
PORT=${KESH_CI_MARIADB_PORT:-3306}
NOM=${KESH_CI_MARIADB_NOM:-mariadb}
ESSAIS=${KESH_CI_MARIADB_ESSAIS:-90}
PAUSE=${KESH_CI_MARIADB_PAUSE:-2}
TIRAGES=${KESH_CI_MARIADB_TIRAGES:-3}
REPRISE=${KESH_CI_MARIADB_REPRISE:-5}
REPRISE_MAX=${KESH_CI_MARIADB_REPRISE_MAX:-20}

# Tire l'image du premier miroir qui la sert ; la référence obtenue va dans IMAGE.
IMAGE=
for miroir in "${MIROIRS[@]}"; do
  delai=$REPRISE
  for ((t = 1; t <= TIRAGES; t++)); do
    if $DOCKER pull --quiet "$miroir" >/dev/null; then
      IMAGE=$miroir
      break 2
    fi
    echo "Tirage refusé : $miroir (essai $t sur $TIRAGES)" >&2
    if ((t < TIRAGES)); then
      sleep "$delai"
      delai=$((delai * 2 > REPRISE_MAX ? REPRISE_MAX : delai * 2))
    fi
  done
done
if [ -z "$IMAGE" ]; then
  echo "MariaDB introuvable : aucun miroir n'a servi l'image (${MIROIRS[*]})." >&2
  exit 1
fi
echo "Image tirée de $IMAGE"

# `--pull=never` : l'image vient d'être tirée ; `docker run` ne retente pas le registre.
$DOCKER run -d --pull=never --name "$NOM" -p "$PORT:3306" \
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
