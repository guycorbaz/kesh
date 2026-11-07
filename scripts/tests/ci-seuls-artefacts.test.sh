#!/usr/bin/env bash
# ci-seuls-artefacts.test.sh — éprouve `scripts/ci-seuls-artefacts.sh` sur un
# mini-dépôt git RÉEL sous `mktemp -d`, chaque cas par de vrais commits, et garde
# les conditions de `.github/workflows/ci.yml` (Story 33-11, #928, AC-5).
#
# Lancé par le job `changes` de la CI, avant le script : un test rouge arrête le
# job avant toute sortie, et les quatre autres jobs tournent. Lançable seul :
# `bash scripts/tests/ci-seuls-artefacts.test.sh`.

set -euo pipefail
export LC_ALL=C

SCRIPTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="$SCRIPTS_DIR/ci-seuls-artefacts.sh"
CI_YML="${CI_YML_POUR_TEST:-$SCRIPTS_DIR/../.github/workflows/ci.yml}"
[ -f "$SCRIPT" ] || { echo "script introuvable : $SCRIPT" >&2; exit 2; }
[ -f "$CI_YML" ] || { echo "ci.yml introuvable : $CI_YML" >&2; exit 2; }

WORK=$(mktemp -d "${TMPDIR:-/tmp}/ci-seuls-artefacts-test.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

ECHECS=0
fail() { echo "    ✗ $*"; ECHECS=$((ECHECS + 1)); }
ok()   { echo "    ✓ $*"; }

# Un git sans configuration de l'appelant, identité passée en ligne de commande.
g() { git -c user.name=Test -c user.email=test@example.invalid -c init.defaultBranch=main -c commit.gpgsign=false "$@"; }

# Un mini-dépôt neuf, un commit de base : `crates/x.rs`. Rend son chemin.
depot() {
  local d
  d=$(mktemp -d "$WORK/depot.XXXXXX")
  mkdir -p "$d/crates"
  g -C "$d" init -q
  echo base > "$d/crates/x.rs"
  g -C "$d" add -A
  g -C "$d" commit -qm base
  g -C "$d" tag base
  echo "$d"
}

# Écrit les fichiers nommés dans le dépôt et commite (sur HEAD).
ecrire() {
  local d=$1; shift
  local f
  for f in "$@"; do
    mkdir -p "$d/$(dirname "$f")"
    echo contenu > "$d/$f"
  done
  g -C "$d" add -A
  g -C "$d" commit -qm travail
}

# La dernière ligne de sortie du script sur base..HEAD.
lancer() {
  local d=$1 evenement=$2
  (cd "$d" && bash "$SCRIPT" "$evenement" "$(g rev-parse base)" "$(g rev-parse HEAD)" 2>/dev/null)
}

attendu() {
  local titre=$1 voulu=$2 obtenu=$3
  echo "  $titre"
  if [ "$obtenu" = "code=$voulu" ]; then
    ok "code=$voulu"
  else
    fail "attendu code=$voulu, obtenu « $obtenu »"
  fi
}

# --- Les cas qui filtrent --------------------------------------------------------
d=$(depot); ecrire "$d" _bmad-output/a.md
attendu "un artefact seul" false "$(lancer "$d" pull_request)"

d=$(depot); ecrire "$d" "_bmad-output/nom à espace.md" "_bmad-output/accentué é.md"
attendu "noms à espace et accentué sous _bmad-output/" false "$(lancer "$d" pull_request)"

d=$(depot); ecrire "$d" "_bmad-output/sous/dossier/b.yaml" _bmad-output/a.md
attendu "plusieurs artefacts, sous-dossier compris" false "$(lancer "$d" pull_request)"

# Une fusion de `main` dans la branche n'entre pas au diff : trois points.
d=$(depot)
g -C "$d" checkout -qb travail
ecrire "$d" _bmad-output/a.md
g -C "$d" checkout -q main
ecrire "$d" crates/y.rs
g -C "$d" tag -f base >/dev/null
g -C "$d" checkout -q travail
g -C "$d" merge -q --no-edit main
ecrire "$d" _bmad-output/c.md
attendu "fusion de main dans la branche : la PR n'apporte que des artefacts" false "$(lancer "$d" pull_request)"

# --- Les cas qui ne filtrent pas -------------------------------------------------
d=$(depot); ecrire "$d" _bmad-output/a.md
attendu "push, même diff" true "$(lancer "$d" push)"

d=$(depot); ecrire "$d" _bmad-output/a.md crates/y.rs
attendu "un artefact et du code" true "$(lancer "$d" pull_request)"

d=$(depot); ecrire "$d" _bmad-output-bis/a.md
attendu "dossier voisin, _bmad-output-bis/" true "$(lancer "$d" pull_request)"

d=$(depot); ecrire "$d" _bmad-output
attendu "_bmad-output comme fichier" true "$(lancer "$d" pull_request)"

d=$(depot); ecrire "$d" docs/manual/fr/x.tex
attendu "un manuel" true "$(lancer "$d" pull_request)"

d=$(depot); ecrire "$d" CHANGELOG.md
attendu "CHANGELOG" true "$(lancer "$d" pull_request)"

d=$(depot); ecrire "$d" .github/workflows/ci.yml
attendu "le workflow" true "$(lancer "$d" pull_request)"

d=$(depot); ecrire "$d" "docs/accentué é.md"
attendu "nom accentué hors du dossier" true "$(lancer "$d" pull_request)"

d=$(depot); mkdir -p "$d/_bmad-output"; g -C "$d" mv crates/x.rs _bmad-output/x.rs; g -C "$d" commit -qm renomme
attendu "renommage de crates/ vers _bmad-output/" true "$(lancer "$d" pull_request)"

d=$(depot); ecrire "$d" _bmad-output/x.md
g -C "$d" tag -f base >/dev/null
g -C "$d" mv _bmad-output/x.md crates/x.md; g -C "$d" commit -qm renomme
attendu "renommage de _bmad-output/ vers crates/" true "$(lancer "$d" pull_request)"

d=$(depot)
attendu "diff vide" true "$(lancer "$d" pull_request)"

d=$(depot); ecrire "$d" _bmad-output/a.md
attendu "arguments manquants" true "$(cd "$d" && bash "$SCRIPT" pull_request 2>/dev/null)"
attendu "aucun argument" true "$(cd "$d" && bash "$SCRIPT" 2>/dev/null)"
attendu "base vide" true "$(cd "$d" && bash "$SCRIPT" pull_request "" HEAD 2>/dev/null)"

attendu "SHA inconnu" true "$(cd "$d" && bash "$SCRIPT" pull_request 0123456789012345678901234567890123456789 HEAD 2>/dev/null)"

# Historiques sans base commune : `git diff A...B` échoue.
d=$(depot)
g -C "$d" checkout -q --orphan orpheline
g -C "$d" rm -rfq .
ecrire "$d" _bmad-output/a.md
attendu "branche orpheline" true "$(lancer "$d" pull_request)"

# `git` factice : rend un nom d'artefact, puis échoue.
d=$(depot); ecrire "$d" _bmad-output/a.md
mkdir -p "$WORK/git-factice"
cat > "$WORK/git-factice/git" <<'FACTICE'
#!/usr/bin/env bash
printf '_bmad-output/a.md\0'
exit 1
FACTICE
chmod +x "$WORK/git-factice/git"
attendu "git qui rend un artefact puis sort en 1" true \
  "$(cd "$d" && PATH="$WORK/git-factice:$PATH" bash "$SCRIPT" pull_request aaa bbb 2>/dev/null)"

# Le script n'écrit qu'une ligne sur sa sortie, et le motif ailleurs.
d=$(depot); ecrire "$d" _bmad-output/a.md
echo "  une seule ligne sur la sortie, motif sur la sortie d'erreur"
n=$(lancer "$d" pull_request | wc -l)
[ "$n" -eq 1 ] && ok "une ligne" || fail "$n lignes"
e=$(cd "$d" && bash "$SCRIPT" pull_request "$(g rev-parse base)" HEAD 2>&1 >/dev/null)
[ -n "$e" ] && ok "motif présent" || fail "aucun motif"

# --- La garde de ci.yml ----------------------------------------------------------
# Le bloc d'un job : de sa clé (deux espaces d'indentation) à la clé suivante.
bloc() { awk -v j="  $1:" '$0 == j {p=1; print; next} p && /^  [A-Za-z0-9_-]+:$/ {exit} p {print}' "$CI_YML"; }

SAUT="!(needs.changes.result == 'success' && needs.changes.outputs.code == 'false')"
# Ouverture de la condition (AC-3 ; AC-6 (d), si le relevé impose `always()`).
OUVERTURE='!cancelled()'
if grep -qF "always() && $SAUT" "$CI_YML"; then OUVERTURE='always()'; fi
COND="if: \${{ $OUVERTURE && $SAUT }}"
COND_SMOKE="if: \${{ $OUVERTURE && $SAUT && needs.docker-build.result == 'success' }}"

contient() {
  local titre=$1 texte=$2 motif=$3
  if printf '%s\n' "$texte" | grep -qF -- "$motif"; then ok "$titre"; else fail "$titre : « $motif » absent"; fi
}

echo "  ci.yml : les trois jobs lourds dépendent de changes, et leur condition est par exclusion"
for job in backend frontend docker-build; do
  b=$(bloc "$job")
  contient "$job : needs" "$b" "    needs: changes"
  contient "$job : condition" "$b" "    $COND"
done

echo "  ci.yml : e2e-smoke"
b=$(bloc e2e-smoke)
contient "e2e-smoke : needs" "$b" "    needs: [changes, docker-build]"
contient "e2e-smoke : condition" "$b" "    $COND_SMOKE"

echo "  ci.yml : les noms de checks requis ne bougent pas"
contient "backend" "$(bloc backend)" "    name: Backend (Rust)"
contient "frontend" "$(bloc frontend)" "    name: Frontend (Svelte)"
contient "docker-build" "$(bloc docker-build)" "    name: Docker build (sanity)"
contient "e2e-smoke" "$(bloc e2e-smoke)" "    name: E2E smoke (Playwright)"

echo "  ci.yml : aucune condition positive sur la sortie"
if grep -qF "outputs.code == 'true'" "$CI_YML"; then fail "== 'true' présent"; else ok "aucun == 'true'"; fi

echo "  ci.yml : le déclencheur reste sans filtre de chemin"
if grep -qE '^\s*(paths|paths-ignore):' "$CI_YML"; then fail "filtre de chemin au workflow"; else ok "aucun"; fi

echo "  ci.yml : le job changes"
b=$(bloc changes)
contient "timeout" "$b" "    timeout-minutes: 10"
contient "sans dépendance : fetch-depth" "$b" "fetch-depth: 0"
contient "événement" "$b" 'github.event_name'
contient "base" "$b" 'github.event.pull_request.base.sha'
contient "tête" "$b" 'github.event.pull_request.head.sha'
contient "sortie" "$b" '>> "$GITHUB_OUTPUT"'
contient "sortie du job" "$b" 'code: ${{ steps.'
if printf '%s\n' "$b" | grep -qE '^    (needs|if):'; then fail "changes a une dépendance ou une condition"; else ok "ni needs ni if"; fi
l_test=$(printf '%s\n' "$b" | grep -n 'ci-seuls-artefacts.test.sh' | head -1 | cut -d: -f1)
l_sortie=$(printf '%s\n' "$b" | grep -n 'GITHUB_OUTPUT' | head -1 | cut -d: -f1)
if [ -n "$l_test" ] && [ -n "$l_sortie" ] && [ "$l_test" -lt "$l_sortie" ]; then
  ok "les tests précèdent l'écriture de la sortie"
else
  fail "ordre des étapes : tests ligne ${l_test:-?}, sortie ligne ${l_sortie:-?}"
fi

echo
if [ "$ECHECS" -gt 0 ]; then
  echo "❌ $ECHECS assertion(s) en échec"
  exit 1
fi
echo "✅ aucune assertion en échec"
