#!/usr/bin/env bash
# ci-mariadb-fork.test.sh — éprouve `scripts/ci-mariadb-fork.sh` sans le vrai `docker`
# (C-31-2-3, variante B), puis garde l'égalité de ses valeurs avec le `services:` de
# `.github/workflows/ci.yml`. Lancé par le job `backend` de la CI ; lançable seul.
#
# Surcharge, pour la sonde de mutation seulement : KESH_CI_MARIADB_SCRIPT (le script éprouvé).

set -euo pipefail

SCRIPTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="${KESH_CI_MARIADB_SCRIPT:-$SCRIPTS_DIR/ci-mariadb-fork.sh}"
CI_YML="$SCRIPTS_DIR/../.github/workflows/ci.yml"

WORK=$(mktemp -d "${TMPDIR:-/tmp}/ci-mariadb-fork-test.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

ECHECS=0
fail() { echo "    ✗ $*"; ECHECS=$((ECHECS + 1)); }
ok()   { echo "    ✓ $*"; }

# Factice : consigne ses appels ; `pull` refuse une référence autant de fois que le dit
# le fichier `refus` (lignes « <référence> <n> »), avec le message d'ECR au run
# 45525662619 ; `inspect` rend tour à tour les états du fichier `etats`.
cat > "$WORK/docker" <<'FACTICE'
#!/usr/bin/env bash
ici="$(dirname "$0")"
echo "docker $*" >> "$ici/appels.log"
case "$1" in
  pull)
    ref="${!#}"
    n=$(awk -v r="$ref" '$1 == r { print $2 }' "$ici/refus")
    if [ "${n:-0}" -gt 0 ]; then
      awk -v r="$ref" '$1 == r { $2 = $2 - 1 } { print }' "$ici/refus" > "$ici/refus.t"; mv "$ici/refus.t" "$ici/refus"
      echo "Error response from daemon: toomanyrequests: Data limit exceeded" >&2; exit 1
    fi ;;
  run) exit "$(cat "$ici/code-run")" ;;
  inspect)
    etat=$(head -1 "$ici/etats"); [ -n "$etat" ] || exit 1
    tail -n +2 "$ici/etats" > "$ici/etats.t" || true; mv "$ici/etats.t" "$ici/etats"
    echo "$etat" ;;
  logs) echo "journal factice" ;;
esac
FACTICE
chmod +x "$WORK/docker"
# Factice de `sleep`, premier dans le PATH : consigne les délais de reprise, sans attendre.
# Les pauses de l'attente de santé valent 0 (KESH_CI_MARIADB_PAUSE=0) et ne sont pas notées.
mkdir -p "$WORK/bin"
cat > "$WORK/bin/sleep" <<'FACTICE'
#!/usr/bin/env bash
[ "$1" = 0 ] || echo "$1" >> "$(dirname "$0")/../delais.log"
FACTICE
chmod +x "$WORK/bin/sleep"

ECR=public.ecr.aws/docker/library/mariadb:10.11
GCR=mirror.gcr.io/library/mariadb:10.11
HUB=docker.io/library/mariadb:10.11

REFUS=""  # lignes « <référence> <n> », lues par le prochain `lancer`
lancer() { # <code-run> <états…>
  : > "$WORK/appels.log"; : > "$WORK/delais.log"; echo "$1" > "$WORK/code-run"; shift
  printf '%s\n' "$@" > "$WORK/etats"
  printf '%s\n' "$REFUS" > "$WORK/refus"
  PATH="$WORK/bin:$PATH" DOCKER="$WORK/docker" KESH_CI_MARIADB_ESSAIS=5 KESH_CI_MARIADB_PAUSE=0 \
    bash "$SCRIPT" > "$WORK/sortie" 2>&1
}
tirages()   { grep '^docker pull ' "$WORK/appels.log" | awk '{ print $NF }' | tr '\n' ' ' | sed 's/ $//'; }
image_run() { grep '^docker run ' "$WORK/appels.log" | awk '{ print $NF }'; }
attendre_egal() { # <libellé> <obtenu> <attendu>
  [ "$2" = "$3" ] && ok "$1" || fail "$1 : « $2 », attendu « $3 »"
}

echo "  ci-mariadb-fork.sh : prête"
REFUS=""
if lancer 0 starting starting healthy; then ok "code 0"; else fail "code non nul"; fi
grep -q 'essai 3' "$WORK/sortie" && ok "prête au troisième essai" || fail "$(cat "$WORK/sortie")"
attendre_egal "un seul tirage, du miroir ECR" "$(tirages)" "$ECR"
grep -qF "Image tirée de $ECR" "$WORK/sortie" && ok "le miroir qui a servi est nommé" || fail "miroir non nommé : $(cat "$WORK/sortie")"
run=$(grep '^docker run ' "$WORK/appels.log" || true)
for attendu in "--pull=never" "--name mariadb" "-p 3306:3306" "-e MARIADB_ROOT_PASSWORD=kesh_root" "-e MARIADB_DATABASE=kesh" \
  "-e MARIADB_USER=kesh" "-e MARIADB_PASSWORD=kesh_dev" "--health-cmd=healthcheck.sh --connect --innodb_initialized" \
  "--health-interval=10s" "--health-timeout=5s" "--health-retries=10" "$ECR"; do
  case "$run" in *"$attendu"*) ok "run : $attendu" ;; *) fail "run sans « $attendu » : $run" ;; esac
done
case "$run" in *"docker.io"*|*" mariadb:10.11"*) fail "image de Docker Hub : $run" ;; *) ok "aucune image de Docker Hub" ;; esac

echo "  ci-mariadb-fork.sh : reprise sur le même miroir"
REFUS="$ECR 1"
if lancer 0 healthy; then ok "code 0"; else fail "code non nul : $(cat "$WORK/sortie")"; fi
attendre_egal "ECR repris au second essai" "$(tirages)" "$ECR $ECR"
attendre_egal "run sur ECR" "$(image_run)" "$ECR"
attendre_egal "un délai avant la reprise" "$(tr '\n' ' ' < "$WORK/delais.log" | sed 's/ $//')" "5"
grep -qF "toomanyrequests" "$WORK/sortie" && ok "le refus du registre est rendu" || fail "refus du registre absent"

echo "  ci-mariadb-fork.sh : ECR refusé, le second miroir sert (le cas du run 45525662619)"
REFUS="$ECR 99"
if lancer 0 healthy; then ok "code 0"; else fail "code non nul : $(cat "$WORK/sortie")"; fi
attendre_egal "trois essais d'ECR, puis mirror.gcr.io" "$(tirages)" "$ECR $ECR $ECR $GCR"
attendre_egal "run sur mirror.gcr.io" "$(image_run)" "$GCR"
grep -qF "Image tirée de $GCR" "$WORK/sortie" && ok "mirror.gcr.io nommé" || fail "miroir non nommé : $(cat "$WORK/sortie")"
attendre_egal "délais croissants entre les essais d'un miroir, aucun après le dernier" \
  "$(tr '\n' ' ' < "$WORK/delais.log" | sed 's/ $//')" "5 10"

echo "  ci-mariadb-fork.sh : ECR et mirror.gcr.io refusés, Docker Hub sert"
REFUS="$ECR 99
$GCR 99"
if lancer 0 healthy; then ok "code 0"; else fail "code non nul : $(cat "$WORK/sortie")"; fi
attendre_egal "l'ordre des miroirs" "$(tirages)" "$ECR $ECR $ECR $GCR $GCR $GCR $HUB"
attendre_egal "run sur Docker Hub" "$(image_run)" "$HUB"

echo "  ci-mariadb-fork.sh : délai borné"
REFUS="$ECR 99"
: > "$WORK/appels.log"; : > "$WORK/delais.log"; echo 0 > "$WORK/code-run"; echo healthy > "$WORK/etats"
printf '%s\n' "$REFUS" > "$WORK/refus"
if PATH="$WORK/bin:$PATH" DOCKER="$WORK/docker" KESH_CI_MARIADB_ESSAIS=5 KESH_CI_MARIADB_PAUSE=0 \
  KESH_CI_MARIADB_TIRAGES=5 KESH_CI_MARIADB_REPRISE=3 KESH_CI_MARIADB_REPRISE_MAX=8 \
  bash "$SCRIPT" > "$WORK/sortie" 2>&1; then ok "code 0"; else fail "code non nul : $(cat "$WORK/sortie")"; fi
attendre_egal "délais 3, 6, puis bornés à 8" "$(tr '\n' ' ' < "$WORK/delais.log" | sed 's/ $//')" "3 6 8 8"

echo "  ci-mariadb-fork.sh : tous les miroirs refusés (la sonde négative)"
REFUS="$ECR 99
$GCR 99
$HUB 99"
if lancer 0 healthy; then fail "tous refusés : code 0"; else ok "tous refusés : code non nul"; fi
attendre_egal "chaque miroir essayé trois fois, dans l'ordre" "$(tirages)" \
  "$ECR $ECR $ECR $GCR $GCR $GCR $HUB $HUB $HUB"
grep -q '^docker run ' "$WORK/appels.log" && fail "docker run lancé sans image" || ok "pas de docker run"
grep -q "aucun miroir n'a servi l'image" "$WORK/sortie" && ok "message d'échec clair" || fail "message absent : $(cat "$WORK/sortie")"

echo "  ci-mariadb-fork.sh : refus du conteneur (la sonde négative)"
REFUS=""
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
# La chaîne du script, dans la version du service (`mariadb:10.11`, compté ci-dessus) :
# un miroir passé à une autre version, ou retiré, rougit ici.
for ref in "$ECR" "$GCR" "$HUB"; do
  grep -qxF "  $ref" "$SCRIPT" && ok "chaîne du script : $ref" || fail "chaîne du script sans « $ref »"
done

echo
if [ "$ECHECS" -eq 0 ]; then echo "✅ aucune assertion en échec"; else echo "❌ $ECHECS assertion(s) en échec"; exit 1; fi
