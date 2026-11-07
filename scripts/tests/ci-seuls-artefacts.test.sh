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

# `main` avance APRÈS la dernière fusion, et `base` est sa pointe : seul `base...tête` (la base
# de fusion) ignore ce que `main` a reçu ; `base..tête` y verrait `crates/z.rs`, absent de la
# branche. C'est le cas qui distingue les trois points des deux (E-3 de la revue P1).
d=$(depot)
g -C "$d" checkout -qb travail
ecrire "$d" _bmad-output/a.md
g -C "$d" checkout -q main
ecrire "$d" crates/z.rs
g -C "$d" tag -f base >/dev/null
g -C "$d" checkout -q travail
ecrire "$d" _bmad-output/b.md
attendu "main avancée après la branche : trois points" false "$(lancer "$d" pull_request)"

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

# Avec un artefact, pour que le diff ne soit pas vide : seule la suppression le rend « hors ».
d=$(depot); g -C "$d" rm -q crates/x.rs; ecrire "$d" _bmad-output/a.md
attendu "un fichier supprimé, avec un artefact" true "$(lancer "$d" pull_request)"

# Un nom SOUS le dossier, dont la seconde ligne serait hors du dossier : une lecture par
# lignes, ou une sortie sans `-z`, le jugerait hors du périmètre.
d=$(depot); ecrire "$d" "_bmad-output/a
crates/b"
attendu "un nom à retour à la ligne, sous le dossier" false "$(lancer "$d" pull_request)"

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

# E-4, E-5 : une sortie qui ne finit pas par NUL est une sortie incomplète, et le dernier nom,
# sans terminateur, se lit quand même : ni l'un ni l'autre ne retombe sur `false`.
mkdir -p "$WORK/git-tronque"
cat > "$WORK/git-tronque/git" <<'FACTICE'
#!/usr/bin/env bash
printf '_bmad-output/a.md\0_bmad-output/b.md'
exit 0
FACTICE
chmod +x "$WORK/git-tronque/git"
attendu "sortie de git sans NUL final : incomplète" true \
  "$(cd "$d" && PATH="$WORK/git-tronque:$PATH" bash "$SCRIPT" pull_request aaa bbb 2>/dev/null)"
cat > "$WORK/git-tronque/git" <<'FACTICE'
#!/usr/bin/env bash
printf '_bmad-output/a.md\0crates/x.rs'
exit 0
FACTICE
attendu "le dernier nom sans NUL est lu" true \
  "$(cd "$d" && PATH="$WORK/git-tronque:$PATH" bash "$SCRIPT" pull_request aaa bbb 2>/dev/null)"

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
# Ouverture de la condition (AC-3 ; AC-6 (d), si le relevé impose `always()`). Elle se lit sur
# la ligne exacte du job `backend`, non n'importe où dans le fichier, puis s'exige aux quatre.
OUVERTURE='!cancelled()'
if grep -qxF "    if: \${{ always() && $SAUT }}" <<<"$(bloc backend)"; then OUVERTURE='always()'; fi
COND="if: \${{ $OUVERTURE && $SAUT }}"
COND_SMOKE="if: \${{ $OUVERTURE && $SAUT && needs.docker-build.result == 'success' }}"

# Sans tube : `printf | grep -q` sous `pipefail` peut lire un « trouvé » comme « absent ».
# La ligne entière, à son indentation : une condition d'étape, plus indentée, ou une ligne
# commentée ne la remplacent pas.
contient_ligne() {
  local titre=$1 texte=$2 ligne=$3
  if grep -qxF -- "$ligne" <<<"$texte"; then ok "$titre"; else fail "$titre : ligne « $ligne » absente"; fi
}

echo "  ci.yml : les trois jobs lourds dépendent de changes, et leur condition est par exclusion"
for job in backend frontend docker-build; do
  b=$(bloc "$job")
  contient_ligne "$job : needs" "$b" "    needs: changes"
  contient_ligne "$job : condition" "$b" "    $COND"
done

echo "  ci.yml : e2e-smoke"
b=$(bloc e2e-smoke)
contient_ligne "e2e-smoke : needs" "$b" "    needs: [changes, docker-build]"
contient_ligne "e2e-smoke : condition" "$b" "    $COND_SMOKE"

echo "  ci.yml : les noms de checks requis ne bougent pas"
contient_ligne "backend" "$(bloc backend)" "    name: Backend (Rust)"
contient_ligne "frontend" "$(bloc frontend)" "    name: Frontend (Svelte)"
contient_ligne "docker-build" "$(bloc docker-build)" "    name: Docker build (sanity)"
contient_ligne "e2e-smoke" "$(bloc e2e-smoke)" "    name: E2E smoke (Playwright)"

echo "  ci.yml : aucune condition positive sur la sortie"
if grep -qF "outputs.code == 'true'" "$CI_YML"; then fail "== 'true' présent"; else ok "aucun == 'true'"; fi

echo "  ci.yml : le déclencheur reste sans filtre de chemin"
if grep -qE '^\s*(paths|paths-ignore):' "$CI_YML"; then fail "filtre de chemin au workflow"; else ok "aucun"; fi

echo "  ci.yml : le job changes"
b=$(bloc changes)
contient_ligne "timeout" "$b" "    timeout-minutes: 10"
contient_ligne "historique entier" "$b" "          fetch-depth: 0"
# Lignes entières : une base et une tête permutées, ou une ligne `run:` qui n'appelle plus
# le script, ne passent pas.
LIGNE_TEST='        run: bash scripts/tests/ci-seuls-artefacts.test.sh'
LIGNE_DECISION='        run: bash scripts/ci-seuls-artefacts.sh "$EVENEMENT" "$BASE" "$TETE" >> "$GITHUB_OUTPUT"'
contient_ligne "événement" "$b" '          EVENEMENT: ${{ github.event_name }}'
contient_ligne "base" "$b" '          BASE: ${{ github.event.pull_request.base.sha }}'
contient_ligne "tête" "$b" '          TETE: ${{ github.event.pull_request.head.sha }}'
contient_ligne "l'étape des tests lance le test" "$b" "$LIGNE_TEST"
contient_ligne "l'étape decision lance le script, sortie vers GITHUB_OUTPUT" "$b" "$LIGNE_DECISION"
contient_ligne "sortie du job, de l'étape decision" "$b" '      code: ${{ steps.decision.outputs.code }}'
contient_ligne "l'étape qui écrit la sortie s'appelle decision" "$b" '        id: decision'
if grep -qE '^    (needs|if):' <<<"$b"; then fail "changes a une dépendance ou une condition"; else ok "ni needs ni if"; fi
if grep -qF 'continue-on-error' <<<"$b"; then fail "continue-on-error : un test rouge n'arrêterait plus le job"; else ok "aucun continue-on-error"; fi
# Une ligne absente ne doit pas arrêter le test sans rapport (`set -e`, `pipefail`).
l_test=$(grep -m1 -nxF -- "$LIGNE_TEST" <<<"$b") || l_test=
l_sortie=$(grep -m1 -nxF -- "$LIGNE_DECISION" <<<"$b") || l_sortie=
l_test=${l_test%%:*}
l_sortie=${l_sortie%%:*}
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
