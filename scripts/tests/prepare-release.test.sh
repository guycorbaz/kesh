#!/usr/bin/env bash
# prepare-release.test.sh — éprouve `scripts/prepare-release.sh` sur des dépôts
# JETABLES, jamais sur le dépôt réel (le script écrit : bump, CHANGELOG).
#
# Chaque cas construit sous `mktemp -d` un mini-dépôt git portant le strict
# nécessaire — `crates/*/Cargo.toml`, `CHANGELOG.md`, deux migrations, une copie
# du script — et place en tête du PATH un `cargo` factice :
#   - `cargo run … --example perishable_exemptions` imprime `$STUB_PERISSABLES`
#     (ou échoue si `STUB_INVENTORY_FAILS=1`) et laisse une trace dans
#     `$STUB_LOG` : c'est ainsi que le test SAIT que le pré-vol a tourné ;
#   - `cargo check` ne fait rien, mais se trace aussi.
# Le script n'a donc aucune porte dérobée de test : il appelle `cargo` comme en
# vrai, et c'est le PATH qui change.
#
# Lancé par `crates/kesh-db/tests/prepare_release_script.rs` (donc par le gate
# nextest et par la CI) ; lançable seul : `bash scripts/tests/prepare-release.test.sh`.
#
# *(Issue #566 : le script refusait une version déjà bumpée et sautait alors son
# pré-vol — contrôle des exemptions périssables et datation du CHANGELOG.)*

set -euo pipefail

SCRIPT_SRC="$(cd "$(dirname "$0")/.." && pwd)/prepare-release.sh"
[ -f "$SCRIPT_SRC" ] || { echo "script introuvable : $SCRIPT_SRC" >&2; exit 2; }

WORK=$(mktemp -d "${TMPDIR:-/tmp}/prepare-release-test.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

# Aucune configuration git de la station ne doit s'en mêler (hooks, signature).
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@example.invalid
export GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@example.invalid

BORNE=20260101000001
EXEMPTEE=20260201000001

ECHECS=0
CAS=0
fail() { echo "    ✗ $*"; ECHECS=$((ECHECS + 1)); }
ok()   { echo "    ✓ $*"; }

# --- Le cargo factice --------------------------------------------------------
STUB_BIN="$WORK/bin"
mkdir -p "$STUB_BIN"
cat > "$STUB_BIN/cargo" <<'STUB'
#!/usr/bin/env bash
echo "cargo $*" >> "$STUB_LOG"
case "$*" in
  *--example\ perishable_exemptions*)
    if [ "${STUB_INVENTORY_FAILS:-0}" = 1 ]; then
      echo "error: could not compile kesh-db (stub)" >&2
      exit 101
    fi
    [ -n "${STUB_PERISSABLES:-}" ] && printf '%s\n' "$STUB_PERISSABLES"
    exit 0 ;;
  check*) exit 0 ;;
  *) echo "cargo factice : appel imprévu « $* »" >&2; exit 99 ;;
esac
STUB
chmod +x "$STUB_BIN/cargo"

# --- Mini-dépôt ----------------------------------------------------------------
# new_repo <dir> <version_api> <version_core> <entete_changelog> [tag_dans_intervalle]
new_repo() {
    local dir=$1 v_api=$2 v_core=$3 entete=$4 tag_intervalle=${5:-0}
    mkdir -p "$dir/crates/kesh-api" "$dir/crates/kesh-core" \
             "$dir/crates/kesh-db/migrations" "$dir/scripts"
    for c in kesh-api:"$v_api" kesh-core:"$v_core"; do
        printf '[package]\nname = "%s"\nversion = "%s"\nedition = "2024"\n' \
            "${c%%:*}" "${c#*:}" > "$dir/crates/${c%%:*}/Cargo.toml"
    done
    printf '# Changelog\n\n---\n\n%s\n\n### Ajouté\n\n- quelque chose.\n\n## [0.0.1] — 2026-01-01\n' \
        "$entete" > "$dir/CHANGELOG.md"
    cp "$SCRIPT_SRC" "$dir/scripts/prepare-release.sh"
    echo "SELECT 1;" > "$dir/crates/kesh-db/migrations/${BORNE}_borne.sql"
    git -C "$dir" init -q -b main
    git -C "$dir" add -A
    git -C "$dir" commit -q -m init
    if [ "$tag_intervalle" = 1 ]; then
        # Un tag publié qui porte la borne SANS la migration exemptée : il est
        # dans l'intervalle que la justification déclare vide.
        git -C "$dir" tag v0.0.9
    fi
    echo "SELECT 2;" > "$dir/crates/kesh-db/migrations/${EXEMPTEE}_exemptee.sql"
    git -C "$dir" add -A
    git -C "$dir" commit -q -m "migration exemptée"
    git -C "$dir" checkout -q -b chore/release
}

# run <dir> <version> — exécute la COPIE du script du mini-dépôt.
# Remplit OUT, RC ; remet le journal du cargo factice à zéro.
run() {
    local dir=$1 version=$2
    # Garde-fou : jamais hors du répertoire jetable.
    case "$dir" in "$WORK"/*) ;; *) echo "refus : $dir hors de $WORK" >&2; exit 2 ;; esac
    export STUB_LOG="$dir.cargo.log"
    : > "$STUB_LOG"
    # La date se relève de part et d'autre de l'exécution : un passage à minuit
    # en plein cas ne fait pas rougir le test (revue P1, P1-B-4 / P1-E-7).
    DATE_AVANT=$(date +%Y-%m-%d)
    set +e
    OUT=$(PATH="$STUB_BIN:$PATH" bash "$dir/scripts/prepare-release.sh" "$version" 2>&1)
    RC=$?
    set -e
    DATE_APRES=$(date +%Y-%m-%d)
}

expect_rc()       { [ "$RC" -eq "$1" ] && ok "code de sortie $1" || fail "code de sortie $RC, attendu $1"; }
expect_out()      { grep -qF -- "$1" <<< "$OUT" && ok "dit « $1 »" || fail "ne dit pas « $1 »"; }
expect_no_out()   { grep -qF -- "$1" <<< "$OUT" && fail "dit « $1 »" || ok "ne dit pas « $1 »"; }
expect_preflight() { grep -qF -- "--example perishable_exemptions" "$STUB_LOG" \
                       && ok "pré-vol exécuté (inventaire lu)" || fail "pré-vol NON exécuté"; }
expect_changelog() { # <dir> <version> <suffixe attendu> <nombre de sections attendu>
    local n
    n=$(grep -cF "## [$2] — " "$1/CHANGELOG.md" || true)
    [ "$n" -eq "$4" ] && ok "$n section(s) [$2]" || fail "$n section(s) [$2], attendu $4"
    grep -qxF "## [$2] — $3" "$1/CHANGELOG.md" && ok "CHANGELOG : [$2] — $3" \
        || fail "CHANGELOG sans « ## [$2] — $3 »"
}
expect_dated_today() { # <dir> <version> — datée du jour de l'exécution, une seule section
    if grep -qxF "## [$2] — $DATE_AVANT" "$1/CHANGELOG.md" \
       || grep -qxF "## [$2] — $DATE_APRES" "$1/CHANGELOG.md"; then
        ok "CHANGELOG : [$2] datée du jour"
    else
        fail "CHANGELOG : [$2] non datée du jour"
    fi
    local n
    n=$(grep -cF "## [$2] — " "$1/CHANGELOG.md" || true)
    [ "$n" -eq 1 ] && ok "1 section [$2]" || fail "$n sections [$2], attendu 1"
}
expect_version() { # <dir> <crate> <version>
    grep -qxF "version = \"$3\"" "$1/crates/$2/Cargo.toml" && ok "$2 à $3" \
        || fail "$2 n'est pas à $3 ($(grep '^version' "$1/crates/$2/Cargo.toml"))"
}
expect_clean() {
    [ -z "$(git -C "$1" status --porcelain)" ] && ok "dépôt intact" \
        || fail "dépôt modifié : $(git -C "$1" status --porcelain | tr '\n' ' ')"
}
cas() { CAS=$((CAS + 1)); echo; echo "[$CAS] $*"; }

export STUB_PERISSABLES="$EXEMPTEE $BORNE"

# --- Cas ----------------------------------------------------------------------

cas "bump normal 0.12.1 → 0.13.0 : inchangé"
R="$WORK/normal"
new_repo "$R" 0.12.1 0.12.1 "## [0.13.0] — Non publié"
run "$R" 0.13.0
expect_rc 0
expect_preflight
expect_out "aucun tag publié dans l'intervalle"
expect_out "[1/3] Bump des Cargo.toml workspace : 0.12.1 → 0.13.0"
expect_out "2 crates bumpés."
expect_no_out "bump sauté"
expect_version "$R" kesh-api 0.13.0
expect_version "$R" kesh-core 0.13.0
expect_dated_today "$R" 0.13.0
grep -qF "cargo check --workspace --offline" "$STUB_LOG" && ok "Cargo.lock régénéré" || fail "cargo check non lancé"

cas "déjà bumpé : bump sauté, pré-vol exécuté, CHANGELOG daté"
R="$WORK/deja"
new_repo "$R" 0.13.0 0.13.0 "## [0.13.0] — Non publié"
run "$R" 0.13.0
expect_rc 0
expect_out "déjà à 0.13.0 : bump sauté, pré-vol exécuté"
expect_preflight
expect_out "Exemption $EXEMPTEE — intervalle déclaré vide : [$BORNE .. $EXEMPTEE)"
expect_out "aucun tag publié dans l'intervalle"
expect_out "[1/3] Bump sauté"
expect_version "$R" kesh-api 0.13.0
expect_version "$R" kesh-core 0.13.0
expect_dated_today "$R" 0.13.0
grep -qF "cargo check --workspace --offline" "$STUB_LOG" && ok "Cargo.lock vérifié" || fail "cargo check non lancé"

cas "relance après commit : idempotent (ne date pas deux fois, refait le pré-vol)"
# Une date passée, pour prouver que la relance ne la remplace pas par aujourd'hui.
sed -i "s|^## \[0.13.0\] — .*|## [0.13.0] — 2026-10-01|" "$R/CHANGELOG.md"
git -C "$R" commit -q -am "chore(release): prepare v0.13.0"
run "$R" 0.13.0
expect_rc 0
expect_preflight
expect_out "déjà datée en pré-vol : rien à écrire."
expect_no_out "→ '## [0.13.0]"
expect_out "Aucun fichier modifié"
expect_changelog "$R" 0.13.0 "2026-10-01" 1
expect_clean "$R"

cas "relance sans commit : refus sur dépôt sale, rien de plus n'est écrit"
R="$WORK/sale"
new_repo "$R" 0.13.0 0.13.0 "## [0.13.0] — Non publié"
run "$R" 0.13.0
avant=$(git -C "$R" diff)
run "$R" 0.13.0
expect_rc 1
expect_out "working tree non clean"
[ "$(git -C "$R" diff)" = "$avant" ] && ok "diff inchangé par la relance" || fail "la relance a écrit"

cas "déjà bumpé ET un tag publié dément l'exemption : refus, rien n'est daté"
R="$WORK/dementi"
new_repo "$R" 0.13.0 0.13.0 "## [0.13.0] — Non publié" 1
run "$R" 0.13.0
expect_rc 1
expect_preflight
expect_out "v0.0.9"
expect_out "Au moins une justification périssable est DÉMENTIE"
expect_changelog "$R" 0.13.0 "Non publié" 1
expect_clean "$R"

cas "déjà bumpé, inventaire illisible : refus fatal"
R="$WORK/illisible"
new_repo "$R" 0.13.0 0.13.0 "## [0.13.0] — Non publié"
STUB_INVENTORY_FAILS=1 run "$R" 0.13.0
expect_rc 1
expect_out "Impossible de lire l'inventaire des exemptions périssables"
expect_clean "$R"

cas "version cible INFÉRIEURE : refus, dépôt intact"
R="$WORK/inferieure"
new_repo "$R" 0.13.0 0.13.0 "## [0.12.9] — Non publié"
run "$R" 0.12.9
expect_rc 1
expect_out "INFÉRIEURE"
expect_version "$R" kesh-api 0.13.0
expect_clean "$R"
# Le tri doit être numérique, pas lexical : 0.9.0 < 0.13.0.
R="$WORK/inferieure-lex"
new_repo "$R" 0.13.0 0.13.0 "## [0.9.0] — Non publié"
run "$R" 0.9.0
expect_rc 1
expect_out "INFÉRIEURE"

cas "version déjà TAGUÉE (déjà bumpée et datée) : refus, la version est publiée"
R="$WORK/publiee"
new_repo "$R" 0.13.0 0.13.0 "## [0.13.0] — 2026-10-01"
git -C "$R" tag v0.13.0
run "$R" 0.13.0
expect_rc 1
expect_out "le tag v0.13.0 existe déjà"
expect_no_out "Release prep terminée"
expect_clean "$R"

cas "déjà bumpé, aucune exemption périssable : le pré-vol le dit"
R="$WORK/sans-exemption"
new_repo "$R" 0.13.0 0.13.0 "## [0.13.0] — Non publié"
STUB_PERISSABLES="" run "$R" 0.13.0
expect_rc 0
expect_preflight
expect_out "aucune exemption à fondement périssable au registre"
expect_dated_today "$R" 0.13.0

cas "section déjà datée avec suffixe : reconnue datée, non redatée"
R="$WORK/suffixe"
new_repo "$R" 0.13.0 0.13.0 "## [0.13.0] — 2026-10-01 (correctif)"
run "$R" 0.13.0
expect_rc 0
expect_preflight
expect_out "déjà datée en pré-vol : rien à écrire."
expect_changelog "$R" 0.13.0 "2026-10-01 (correctif)" 1
expect_clean "$R"

cas "section ni « Non publié » ni datée : refus nommé"
R="$WORK/indatee"
new_repo "$R" 0.13.0 0.13.0 "## [0.13.0] — bientôt"
run "$R" 0.13.0
expect_rc 1
expect_out "ni « Non publié » ni datée"
expect_clean "$R"

cas "aucune section [0.13.0] : refus"
R="$WORK/absente"
new_repo "$R" 0.13.0 0.13.0 "## [0.12.0] — Non publié"
run "$R" 0.13.0
expect_rc 1
expect_out "introuvable dans CHANGELOG.md"
expect_clean "$R"

cas "sur main : refus"
R="$WORK/main"
new_repo "$R" 0.13.0 0.13.0 "## [0.13.0] — Non publié"
git -C "$R" checkout -q main
run "$R" 0.13.0
expect_rc 1
expect_out "tu es sur la branche 'main'"
expect_clean "$R"

cas "bump partiel (kesh-api à 0.13.0, kesh-core à 0.12.1) : refus"
R="$WORK/partiel"
new_repo "$R" 0.13.0 0.12.1 "## [0.13.0] — Non publié"
run "$R" 0.13.0
expect_rc 1
expect_out "Bump partiel"
expect_out "kesh-core"
expect_clean "$R"

cas "deux sections [0.13.0] au CHANGELOG : refus"
R="$WORK/doublon"
new_repo "$R" 0.13.0 0.13.0 "## [0.13.0] — Non publié

## [0.13.0] — 2026-10-01"
run "$R" 0.13.0
expect_rc 1
expect_out "une seule est admise"
expect_clean "$R"

echo
if [ "$ECHECS" -gt 0 ]; then
    echo "⛔ $ECHECS assertion(s) en échec sur $CAS cas."
    exit 1
fi
echo "✅ $CAS cas, aucune assertion en échec."
