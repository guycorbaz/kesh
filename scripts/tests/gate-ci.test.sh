#!/usr/bin/env bash
# gate-ci.test.sh — éprouve `scripts/gate-ci.sh` sans jamais toucher au vrai `docker`
# ni au vrai `cargo` (Story 33-10, AC-1, AC-3, AC-7).
#
# Chaque cas bâtit sous `mktemp -d` un MINI-DÉPÔT, copie du script et jeu de migrations
# factices. `KESH_DOCKER` et `KESH_CARGO` pointent vers des factices qui consignent leur
# appel. Seule la garde ci.yml ⇄ script lit les vrais fichiers.
#
# Lancé par `crates/kesh-db/tests/gate_ci_script.rs` (gate et CI) ; lançable seul.

set -euo pipefail

SCRIPTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT_SRC="$SCRIPTS_DIR/gate-ci.sh"
CI_YML="$SCRIPTS_DIR/../.github/workflows/ci.yml"

WORK=$(mktemp -d "${TMPDIR:-/tmp}/gate-ci-test.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

ECHECS=0
fail() { echo "    ✗ $*"; ECHECS=$((ECHECS + 1)); }
ok()   { echo "    ✓ $*"; }

mkdir -p "$WORK/bin"
# L'environnement de `cargo` étant fermé, les factices lisent leur réglage dans leur
# propre dossier : `courant` (le mini-dépôt), `code-docker`, `code-cargo`.
cat > "$WORK/bin/docker" <<'FACTICE'
#!/usr/bin/env bash
ici="$(dirname "$0")"
echo "docker $*" >> "$(cat "$ici/courant")/appels.log"
cat >/dev/null
exit "$(cat "$ici/code-docker")"
FACTICE
cat > "$WORK/bin/cargo" <<'FACTICE'
#!/usr/bin/env bash
ici="$(dirname "$0")"
echo "cargo $*" >> "$(cat "$ici/courant")/appels.log"
env > "$(cat "$ici/courant")/env.txt"
cat <<'SORTIE'
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: FAILED. 2 passed; 4 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.10s
SORTIE
exit "$(cat "$ici/code-cargo")"
FACTICE
chmod +x "$WORK/bin/docker" "$WORK/bin/cargo"

# Un mini-dépôt neuf ; rend son chemin.
depot() {
  local d
  d=$(mktemp -d "$WORK/depot.XXXXXX")
  mkdir -p "$d/scripts" "$d/crates/kesh-db/migrations"
  cp "$SCRIPT_SRC" "$d/scripts/gate-ci.sh"
  echo "SELECT 1;" > "$d/crates/kesh-db/migrations/20260101000001_a.sql"
  echo "SELECT 2;" > "$d/crates/kesh-db/migrations/20260101000002_b.sql"
  echo "SELECT 3;" > "$d/scripts/seed-dev-db.sql"
  echo "$d"
}

# Lance le script d'un mini-dépôt ; `appels.log` y consigne les appels.
lancer() {
  local d=$1; shift
  : > "$d/appels.log"
  echo "$d" > "$WORK/bin/courant"
  echo "${STUB_CARGO_CODE:-0}" > "$WORK/bin/code-cargo"
  echo "${STUB_DOCKER_CODE:-0}" > "$WORK/bin/code-docker"
  TMPDIR="$WORK" KESH_DOCKER="$WORK/bin/docker" KESH_CARGO="$WORK/bin/cargo" KESH_NO_MEMGUARD=1 \
    bash "$d/scripts/gate-ci.sh" "$@"
}

# --- Les refus, chacun avec zéro appel ---------------------------------------------
echo "  bases refusées, zéro appel"
for b in kesh kesh_gate kesh_gate_x kesh_e2e kesh_e2e_x root mysql "" kesh_ci_ KESH_CI_X "kesh_ci_a;b" "kesh_ci_a b" kesh_cix; do
  d=$(depot)
  if lancer "$d" "$b" >/dev/null 2>&1; then fail "« $b » acceptée"
  elif [ -s "$d/appels.log" ]; then fail "« $b » : des appels"
  else ok "« $b » refusée"; fi
done

echo "  .env atteignable, zéro appel"
d=$(depot); touch "$d/.env"
if lancer "$d" kesh_ci_x >/dev/null 2>&1; then fail "à la racine : accepté"
elif [ -s "$d/appels.log" ]; then fail "à la racine : des appels"
else ok "à la racine"; fi

PARENT=$(mktemp -d "$WORK/parent.XXXXXX"); touch "$PARENT/.env"
d="$PARENT/depot"; mkdir -p "$d/scripts" "$d/crates/kesh-db/migrations"
cp "$SCRIPT_SRC" "$d/scripts/gate-ci.sh"
if lancer "$d" kesh_ci_x >/dev/null 2>&1; then fail "dans un parent : accepté"
elif [ -s "$d/appels.log" ]; then fail "dans un parent : des appels"
else ok "dans un parent"; fi

# `cargo test` lance chaque binaire depuis son crate, et `dotenvy` remonte de là.
d=$(depot); mkdir -p "$d/crates/kesh-api"; touch "$d/crates/kesh-api/.env"
if lancer "$d" kesh_ci_x >/dev/null 2>&1; then fail "dans un crate : accepté"
elif [ -s "$d/appels.log" ]; then fail "dans un crate : des appels"
else ok "dans un crate"; fi
d=$(depot); mkdir -p "$d/target/x" "$d/frontend/node_modules/y"; touch "$d/target/x/.env" "$d/frontend/node_modules/y/.env"
if lancer "$d" kesh_ci_x >/dev/null 2>&1; then ok "sous target/ et node_modules/ : ignoré (sonde positive)"; else fail "target/ refusé à tort"; fi

echo "  journal : éprouvé avant tout appel"
d=$(depot)
if lancer "$d" kesh_ci_x "$WORK/inexistant/journal.log" >/dev/null 2>&1; then fail "dossier du journal absent : accepté"
elif [ -s "$d/appels.log" ]; then fail "dossier du journal absent : des appels"
else ok "dossier du journal absent : refusé, zéro appel"; fi
d=$(depot); APPELANT=$(mktemp -d "$WORK/appelant.XXXXXX")
(cd "$APPELANT" && lancer "$d" kesh_ci_x relatif.log >/dev/null 2>&1) || fail "journal relatif en échec"
[ -s "$APPELANT/relatif.log" ] && ok "journal relatif : chez l'appelant" || fail "journal relatif : ailleurs"
[ ! -e "$d/relatif.log" ] && ok "et non dans le dépôt" || fail "journal relatif dans le dépôt"

# --- Le chemin nominal --------------------------------------------------------------
echo "  environnement fermé, et la base forcée"
d=$(depot)
sortie=$(DATABASE_URL=mysql://sonde:sonde@sonde.invalid/sonde KESH_ADMIN_BACKUP_DIR=/inexistant/sonde \
  KESH_SONDE=sonde RUSTFLAGS=sonde lancer "$d" kesh_ci_nominal "$d/journal.log" 2>/dev/null) || fail "chemin nominal en échec"
noms=$(cut -d= -f1 "$d/env.txt" | /usr/bin/grep -vxE 'PWD|SHLVL|_|OLDPWD' | LC_ALL=C sort | tr '\n' ' ')
attendu="CARGO_HOME CARGO_TARGET_DIR CARGO_TERM_COLOR DATABASE_URL HOME PATH RUSTUP_HOME RUST_BACKTRACE "
[ "$noms" = "$attendu" ] && ok "liste exacte" || fail "environnement : $noms"
/usr/bin/grep -qx 'DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh_ci_nominal' "$d/env.txt" \
  && ok "DATABASE_URL forcée" || fail "DATABASE_URL : $(/usr/bin/grep DATABASE_URL "$d/env.txt")"
/usr/bin/grep -q sonde "$d/env.txt" && fail "une sonde a traversé" || ok "aucune sonde"
/usr/bin/grep -qx 'CARGO_TERM_COLOR=always' "$d/env.txt" && /usr/bin/grep -qx 'RUST_BACKTRACE=1' "$d/env.txt" \
  && ok "variables de la CI" || fail "variables de la CI"

echo "  migrations par la boucle, seed, aucun sqlx, jamais un redémarrage"
n=$(/usr/bin/grep -c '^docker exec -i -e MYSQL_PWD=kesh_dev_root kesh-mariadb-dev mariadb -uroot kesh_ci_nominal$' "$d/appels.log")
[ "$n" -eq 3 ] && ok "deux migrations et le seed, une par une" || fail "$n appels"
/usr/bin/grep -q 'DROP DATABASE IF EXISTS `kesh_ci_nominal`; CREATE DATABASE `kesh_ci_nominal`' "$d/appels.log" \
  && ok "base recréée" || fail "base non recréée"
/usr/bin/grep -qE 'docker (restart|stop|rm|compose)|GRANT|REVOKE' "$d/appels.log" && fail "geste interdit" || ok "aucun geste interdit"
/usr/bin/grep -q sqlx "$d/appels.log" && fail "sqlx appelé" || ok "aucun sqlx"
/usr/bin/grep -q "sqlx migrate" "$SCRIPT_SRC" && fail "sqlx dans le script" || ok "aucun sqlx au script"

echo "  la commande de la CI"
/usr/bin/grep -qx 'cargo test --workspace -j1 -- --test-threads=1' "$d/appels.log" \
  && ok "cargo test, à l'identique" || fail "$(/usr/bin/grep '^cargo' "$d/appels.log")"

echo "  le décompte, recompté depuis test result:"
derniere=$(printf '%s\n' "$sortie" | tail -1)
[ "$derniere" = "gate-ci : 5 passés, 4 échoués, 3 ignorés" ] && ok "$derniere" || fail "« $derniere »"
[ -s "$d/journal.log" ] && ok "journal écrit" || fail "journal vide"

echo "  le code de sortie de cargo"
d=$(depot)
STUB_CARGO_CODE=101 lancer "$d" kesh_ci_x >/dev/null 2>&1 && c=0 || c=$?
[ "$c" -eq 101 ] && ok "101 propagé" || fail "code $c"
d=$(depot)
STUB_CARGO_CODE=0 lancer "$d" kesh_ci_x >/dev/null 2>&1 && c=0 || c=$?
[ "$c" -eq 0 ] && ok "0 propagé" || fail "code $c"

echo "  une migration en échec arrête tout"
d=$(depot)
STUB_DOCKER_CODE=1 lancer "$d" kesh_ci_x >/dev/null 2>&1 && c=0 || c=$?
[ "$c" -ne 0 ] && ok "code $c" || fail "code 0"
/usr/bin/grep -q '^cargo' "$d/appels.log" && fail "cargo lancé quand même" || ok "cargo non lancé"

# --- La garde ci.yml ⇄ script (fichiers réels) --------------------------------------
echo "  ci.yml et le script disent la même chose"
ligne=$(/usr/bin/grep -E '^\s*run: cargo test ' "$CI_YML" | sed -E 's/^\s*run: cargo //')
[ -n "$ligne" ] && ok "ci.yml : cargo $ligne" || fail "ci.yml : aucune ligne « run: cargo test »"
/usr/bin/grep -qF "\"\$CARGO\" $ligne " "$SCRIPT_SRC" && ok "le script lance la même" || fail "script ≠ ci.yml : $ligne"
/usr/bin/grep -qF 'for migration in crates/kesh-db/migrations/*.sql; do' "$CI_YML" \
  && ok "ci.yml : la boucle" || fail "ci.yml n'applique plus les migrations par sa boucle"
/usr/bin/grep -qF 'mariadb -h 127.0.0.1 -u root -pkesh_root kesh < "$migration"' "$CI_YML" \
  && ok "ci.yml : en root, une à une" || fail "ci.yml : la ligne de migration a changé"
/usr/bin/grep -qF 'mariadb -h 127.0.0.1 -u root -pkesh_root kesh < scripts/seed-dev-db.sql' "$CI_YML" \
  && ok "ci.yml : le seed" || fail "ci.yml : la ligne du seed a changé"
/usr/bin/grep -qF 'for migration in crates/kesh-db/migrations/*.sql; do' "$SCRIPT_SRC" \
  && ok "script : la boucle" || fail "script : la boucle"
/usr/bin/grep -qF 'DATABASE_URL: mysql://kesh:kesh_dev@127.0.0.1:3306/kesh' "$CI_YML" \
  && /usr/bin/grep -qF 'DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/$BASE' "$SCRIPT_SRC" \
  && ok "même utilisateur" || fail "utilisateur de test différent"

# --- AC-7 : aucun chemin de la station ni accès réel, écrit ---------------------------
echo "  AC-7 : aucun chemin de la station"
MOTIF='/home/|\$\{?HOME\}?/|~/|/volume1|gvfs|kesh-test|192\.168\.|\.local\b|\bnas\b'
SONDE=$(mktemp "$WORK/sonde.XXXXXX"); echo 'x=/volume1/y' > "$SONDE"
/usr/bin/grep -qiE "$MOTIF" "$SONDE" && ok "le motif reconnaît sa sonde" || fail "motif muet"
if /usr/bin/grep -rniE "$MOTIF" "$SCRIPT_SRC"; then fail "chemin de la station"; else ok "aucun"; fi
if /usr/bin/grep -n gcorbaz "$SCRIPT_SRC"; then fail "gcorbaz"; else ok "aucun gcorbaz"; fi

echo
if [ "$ECHECS" -gt 0 ]; then
  echo "❌ $ECHECS assertion(s) en échec"
  exit 1
fi
echo "✅ aucune assertion en échec"
