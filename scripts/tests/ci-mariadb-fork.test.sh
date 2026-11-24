#!/usr/bin/env bash
# ci-mariadb-fork.test.sh — éprouve `scripts/ci-mariadb-fork.sh` sans le vrai `docker`
# (C-31-2-3, variante B), puis garde l'égalité de ses valeurs avec le `services:` de
# `.github/workflows/ci.yml`. Lancé par le job `backend` de la CI ; lançable seul.

set -euo pipefail

SCRIPTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="$SCRIPTS_DIR/ci-mariadb-fork.sh"
CI_YML="$SCRIPTS_DIR/../.github/workflows/ci.yml"

WORK=$(mktemp -d "${TMPDIR:-/tmp}/ci-mariadb-fork-test.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

ECHECS=0
fail() { echo "    ✗ $*"; ECHECS=$((ECHECS + 1)); }
ok()   { echo "    ✓ $*"; }

# Factice : consigne ses appels ; `inspect` rend tour à tour les états du fichier `etats`.
cat > "$WORK/docker" <<'FACTICE'
#!/usr/bin/env bash
ici="$(dirname "$0")"
echo "docker $*" >> "$ici/appels.log"
case "$1" in
  run) exit "$(cat "$ici/code-run")" ;;
  inspect)
    etat=$(head -1 "$ici/etats"); [ -n "$etat" ] || exit 1
    tail -n +2 "$ici/etats" > "$ici/etats.t" || true; mv "$ici/etats.t" "$ici/etats"
    echo "$etat" ;;
  logs) echo "journal factice" ;;
esac
FACTICE
chmod +x "$WORK/docker"

lancer() { # <code-run> <états…>
  : > "$WORK/appels.log"; echo "$1" > "$WORK/code-run"; shift
  printf '%s\n' "$@" > "$WORK/etats"
  DOCKER="$WORK/docker" KESH_CI_MARIADB_ESSAIS=5 KESH_CI_MARIADB_PAUSE=0 bash "$SCRIPT" > "$WORK/sortie" 2>&1
}

echo "  ci-mariadb-fork.sh : prête"
if lancer 0 starting starting healthy; then ok "code 0"; else fail "code non nul"; fi
grep -q 'essai 3' "$WORK/sortie" && ok "prête au troisième essai" || fail "$(cat "$WORK/sortie")"
run=$(grep '^docker run ' "$WORK/appels.log" || true)
for attendu in "--name mariadb" "-p 3306:3306" "-e MARIADB_ROOT_PASSWORD=kesh_root" "-e MARIADB_DATABASE=kesh" \
  "-e MARIADB_USER=kesh" "-e MARIADB_PASSWORD=kesh_dev" "--health-cmd=healthcheck.sh --connect --innodb_initialized" \
  "--health-interval=10s" "--health-timeout=5s" "--health-retries=10" "public.ecr.aws/docker/library/mariadb:10.11"; do
  case "$run" in *"$attendu"*) ok "run : $attendu" ;; *) fail "run sans « $attendu » : $run" ;; esac
done
case "$run" in *"docker.io"*|*" mariadb:10.11"*) fail "image de Docker Hub : $run" ;; *) ok "aucune image de Docker Hub" ;; esac

echo "  ci-mariadb-fork.sh : refus (la sonde négative)"
for cas in "unhealthy" "exited" "starting starting starting starting starting" ""; do
  # shellcheck disable=SC2086
  if lancer 0 $cas; then fail "« ${cas:-inspect en échec} » : code 0"; else ok "« ${cas:-inspect en échec} » : code non nul"; fi
  grep -q '^docker logs ' "$WORK/appels.log" && ok "journal du conteneur rendu" || fail "journal absent"
done
if lancer 1 healthy; then fail "docker run en échec : code 0"; else ok "docker run en échec : code non nul"; fi
grep -q '^docker inspect' "$WORK/appels.log" && fail "attente lancée après un run en échec" || ok "pas d'attente"

echo "  ci.yml : les valeurs du service sont celles du script"
grep -qF 'run: bash scripts/ci-mariadb-fork.sh' "$CI_YML" && ok "ci.yml lance le script" || fail "ci.yml ne lance pas le script"
for ligne in "MARIADB_ROOT_PASSWORD: kesh_root" "MARIADB_DATABASE: kesh" "MARIADB_USER: kesh" "MARIADB_PASSWORD: kesh_dev" \
  "- 3306:3306" '--health-cmd="healthcheck.sh --connect --innodb_initialized"' "--health-interval=10s" \
  "--health-timeout=5s" "--health-retries=10"; do
  n=$(grep -cF -- "$ligne" "$CI_YML" || true)
  [ "$n" -eq 2 ] && ok "ci.yml (deux services) : $ligne" || fail "ci.yml : « $ligne » $n fois, deux attendues"
done
n=$(grep -cF "'public.ecr.aws/docker/library/mariadb:10.11'" "$CI_YML" || true)
[ "$n" -eq 0 ] && ok "aucun service ne tire du miroir (le script le fait)" || fail "$n service(s) sur le miroir"
n=$(grep -cF "&& 'mariadb:10.11' || '' }}" "$CI_YML" || true)
[ "$n" -eq 2 ] && ok "les deux services se désactivent sur un fork" || fail "$n service(s) désactivable(s), deux attendus"

echo
if [ "$ECHECS" -eq 0 ]; then echo "✅ aucune assertion en échec"; else echo "❌ $ECHECS assertion(s) en échec"; exit 1; fi
