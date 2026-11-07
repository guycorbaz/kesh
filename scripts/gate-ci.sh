#!/usr/bin/env bash
# gate-ci.sh — rejoue la suite backend TELLE QUE LA CI LA LANCE (Story 33-10, #925).
#
# `scripts/test-fast.sh` lance `cargo nextest` : un processus par test. La CI lance
# `cargo test` : un processus par binaire, si bien qu'un test qui laisse l'état du
# processus sale (une variable d'environnement, par exemple) ne se voit qu'en CI.
#
# Usage : scripts/gate-ci.sh <base> [journal]
#   <base>     kesh_ci_<suffixe> (a-z, 0-9, _) : recréée à neuf. Jamais `kesh`,
#              `kesh_gate*` ni `kesh_e2e*`.
#   [journal]  copie de la sortie (défaut : fichier temporaire, nommé à la fin).
#
# Comme la CI (`.github/workflows/ci.yml`) : migrations appliquées une à une par
# `mariadb … < fichier`, sans `_sqlx_migrations`, seed `scripts/seed-dev-db.sql`,
# en root ; tests en `kesh`. La commande `cargo test` et la boucle sont gardées
# égales au workflow par `scripts/tests/gate-ci.test.sh`.
#
# Écarts à la CI, tous déclarés : le nom de la base, le datadir en tmpfs du MariaDB
# de dev, et les variables du runner GitHub.
#
# ⛔ Le script recrée une base sur le MariaDB de dev, jamais le conteneur lui-même
# (`restart` efface les bases de tous), et ne pose aucun droit : ceux de `kesh`
# couvrent déjà `kesh%`. Refuse si un `.env` est atteignable depuis le dépôt : la CI
# n'en a aucun, et `dotenvy` le lirait.
#
# Sous `mem-guard` s'il n'y est pas déjà ; les verrous de la station restent à
# l'appelant. Injections de test : KESH_DOCKER, KESH_CARGO, KESH_NO_MEMGUARD.

set -uo pipefail

DOCKER=${KESH_DOCKER:-docker}
CARGO=${KESH_CARGO:-cargo}
CONTENEUR=kesh-mariadb-dev
MDP_ROOT=kesh_dev_root

refus() {
  echo "gate-ci : refus : $1" >&2
  exit "$2"
}

BASE=${1:-}
JOURNAL=${2:-}
[[ "$BASE" =~ ^kesh_ci_[a-z0-9_]+$ ]] \
  || refus "base « $BASE » hors du motif kesh_ci_<suffixe> (a-z, 0-9, _)" 2

RACINE="$(cd "$(dirname "$0")/.." && pwd)"

d=$RACINE
while :; do
  [ ! -e "$d/.env" ] || refus "« $d/.env » atteignable depuis le dépôt : la CI n'en a aucun" 3
  [ "$d" != / ] || break
  d=$(dirname "$d")
done

if [ -z "${KESH_MEMGUARD_ACTIVE:-}" ] && [ -z "${KESH_NO_MEMGUARD:-}" ]; then
  export KESH_MEMGUARD_ACTIVE=1
  exec "$RACINE/scripts/mem-guard.sh" "$0" "$@"
fi

cd "$RACINE" || refus "dépôt illisible" 2

[ -n "$JOURNAL" ] || JOURNAL=$(mktemp "${TMPDIR:-/tmp}/gate-ci.XXXXXX.log") \
  || refus "journal temporaire impossible" 2

# `root`, comme les étapes de migration et de seed de la CI ; mot de passe par
# l'environnement, jamais en argument.
sql_root() {
  "$DOCKER" exec -i -e "MYSQL_PWD=$MDP_ROOT" "$CONTENEUR" mariadb -uroot "$@"
}

echo "▶ base $BASE recréée"
sql_root -e "DROP DATABASE IF EXISTS \`$BASE\`; CREATE DATABASE \`$BASE\`;" < /dev/null \
  || refus "recréation de la base en échec" 4

echo "▶ migrations, une à une, comme la CI"
for migration in crates/kesh-db/migrations/*.sql; do
  sql_root "$BASE" < "$migration" || refus "migration en échec : $migration" 4
done

echo "▶ seed, comme la CI"
sql_root "$BASE" < scripts/seed-dev-db.sql || refus "seed en échec" 4

# ⛔ L'environnement est fermé : ni `DATABASE_URL` du shell, ni une `KESH_*` de
# l'appelant, ni rien d'autre que cette liste n'atteint les tests.
ENV_CI=(
  "PATH=$PATH"
  "HOME=${HOME:-/nonexistent}"
  "CARGO_HOME=${CARGO_HOME:-${HOME:-/nonexistent}/.cargo}"
  "RUSTUP_HOME=${RUSTUP_HOME:-${HOME:-/nonexistent}/.rustup}"
  "CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$RACINE/target}"
  "CARGO_TERM_COLOR=always"
  "RUST_BACKTRACE=1"
  "DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/$BASE"
)

echo "▶ cargo test, comme la CI"
env -i "${ENV_CI[@]}" "$CARGO" test --workspace -j1 -- --test-threads=1 2>&1 | tee "$JOURNAL"
CODE=${PIPESTATUS[0]}

# Recompté depuis les lignes `test result:`, jamais recopié.
sed 's/\x1b\[[0-9;]*m//g' "$JOURNAL" | awk '
  /^test result:/ {
    for (i = 1; i <= NF; i++) {
      if ($i ~ /^passed;?$/)  p += $(i - 1)
      if ($i ~ /^failed;?$/)  f += $(i - 1)
      if ($i ~ /^ignored;?$/) k += $(i - 1)
    }
  }
  END { printf "gate-ci : %d passés, %d échoués, %d ignorés\n", p, f, k }
'
echo "gate-ci : journal $JOURNAL, code $CODE" >&2
exit "$CODE"
