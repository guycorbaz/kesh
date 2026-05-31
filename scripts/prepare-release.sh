#!/usr/bin/env bash
# prepare-release.sh — Bump version + finaliser CHANGELOG date pour une release Kesh.
#
# Évite les 2 RCs manuels observés v0.1.1 (cf. retrospective Epic Hotfix v0.1.1
# §C2) en faisant en UN SEUL commit ce que les 2 PRs #129 + #130 ont fait
# séparément :
#   1. Bump des 10 crates `crates/*/Cargo.toml` à la version cible.
#   2. Régénération `Cargo.lock` (via `cargo check`).
#   3. CHANGELOG.md : remplacer `## [X.Y.Z] — Non publié` par
#      `## [X.Y.Z] — YYYY-MM-DD` (date du jour).
#
# **N'automatise PAS** la mise à jour du README roadmap (dépend du scope précis
# de chaque release — l'auteur doit la rédiger manuellement avant ou après).
# Le script affiche un rappel.
#
# **N'automatise PAS** le tag git ni le push. L'auteur fait `git tag vX.Y.Z` +
# `git push --tags` manuellement après inspection du commit produit.
#
# Usage : `scripts/prepare-release.sh 0.1.2`

set -euo pipefail

# --- Validation argument ---

if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>" >&2
    echo "Exemple: $0 0.1.2" >&2
    exit 1
fi

NEW_VERSION="$1"

# Format X.Y.Z (semver simple, pas de pré-release v0.1).
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "ERREUR: version '$NEW_VERSION' invalide (attendu X.Y.Z, ex. 0.1.2)." >&2
    exit 1
fi

# --- Pré-flight ---

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

if [ ! -d crates ] || [ ! -f CHANGELOG.md ]; then
    echo "ERREUR: ce script doit être lancé depuis la racine du repo Kesh." >&2
    exit 1
fi

# Working tree clean (sinon le commit risque de capturer des changements parasites).
if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
    echo "ERREUR: working tree non clean. Commit ou stash les modifications en cours :" >&2
    git status --short
    exit 1
fi

# Brancher pas sur main (cohérent §"Règle de branchement avant commit").
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" = "main" ] || [ "$CURRENT_BRANCH" = "master" ]; then
    echo "ERREUR: tu es sur la branche '$CURRENT_BRANCH'. Crée d'abord une branche release :" >&2
    echo "  git checkout -b chore/release-v$NEW_VERSION" >&2
    exit 1
fi

echo "Branche courante : $CURRENT_BRANCH"
echo "Version cible    : $NEW_VERSION"
echo

# --- Détection version actuelle ---

CURRENT_VERSION=$(grep -m1 '^version = ' crates/kesh-api/Cargo.toml | sed -E 's/^version = "([^"]+)".*/\1/')
echo "Version actuelle (crates/kesh-api/Cargo.toml) : $CURRENT_VERSION"

if [ "$CURRENT_VERSION" = "$NEW_VERSION" ]; then
    echo "ERREUR: version cible identique à la version actuelle. Rien à bumper." >&2
    exit 1
fi

# --- (1) Bump des 10 crates Cargo.toml ---

echo
echo "[1/3] Bump des Cargo.toml workspace : $CURRENT_VERSION → $NEW_VERSION"

BUMPED=0
for f in crates/*/Cargo.toml; do
    if grep -q "^version = \"$CURRENT_VERSION\"" "$f"; then
        # `sed -i` portable : on cible la 1ère occurrence `version = "X.Y.Z"`
        # (les Cargo.toml ont la ligne version en position fixe ligne 3).
        sed -i "0,/^version = \"$CURRENT_VERSION\"/s//version = \"$NEW_VERSION\"/" "$f"
        echo "  ✓ $f"
        BUMPED=$((BUMPED + 1))
    fi
done

if [ "$BUMPED" -eq 0 ]; then
    echo "ERREUR: aucun crate Cargo.toml ne portait la version $CURRENT_VERSION. Anomalie." >&2
    exit 1
fi

echo "  $BUMPED crates bumpés."

# --- (2) Régénérer Cargo.lock ---

echo
echo "[2/3] Régénération de Cargo.lock"

# `cargo check --workspace` est le moyen le plus rapide de mettre à jour
# Cargo.lock avec les nouvelles versions. `--offline` évite tout download
# inattendu — la résolution doit se faire en local uniquement (workspace deps).
if ! cargo check --workspace --offline 2>&1 | tail -5; then
    echo "  cargo check --offline a échoué — retry sans --offline (peut nécessiter network)..."
    cargo check --workspace 2>&1 | tail -5
fi

# --- (3) CHANGELOG date ---

echo
echo "[3/3] CHANGELOG.md : finaliser la date pour [$NEW_VERSION]"

TODAY=$(date +%Y-%m-%d)
# `grep -F` = fixed-string : `[`/`]` doivent être littéraux (pas échappés).
PATTERN="## [$NEW_VERSION] — Non publié"
REPLACEMENT="## [$NEW_VERSION] — $TODAY"

if ! grep -qF "$PATTERN" CHANGELOG.md; then
    echo "ERREUR: pattern '$PATTERN' introuvable dans CHANGELOG.md." >&2
    echo "Le CHANGELOG doit contenir une section '$PATTERN' à finaliser." >&2
    echo "Vérifie que la section existe et que le texte exact match (espaces, tirets longs, etc.)." >&2
    exit 1
fi

# Pour sed BRE, échapper les `[` `]` (signification regex caractère class).
sed -i "s|## \\[$NEW_VERSION\\] — Non publié|$REPLACEMENT|" CHANGELOG.md
echo "  ✓ CHANGELOG.md : '$PATTERN' → '$REPLACEMENT'"

# --- Récap + invite commit ---

echo
echo "════════════════════════════════════════════════════════════════════════"
echo "✅ Release prep terminée."
echo
echo "Fichiers modifiés :"
git diff --stat | tail -15
echo
echo "Étapes restantes (manuelles) :"
echo "  1. Vérifier README.md (Feuille de route) reflète v$NEW_VERSION done"
echo "  2. git add -A && git commit -m \"chore(release): prepare v$NEW_VERSION — bump + CHANGELOG date\""
echo "  3. git push (PR + merge sur main)"
echo "  4. git tag v$NEW_VERSION + git push --tags → déclenche release.yml"
echo
echo "Si la CI release.yml échoue sur le smoke test /health, c'est qu'un Cargo.toml"
echo "a été oublié. Le script bumpe les 10 crates standards — vérifier manuellement"
echo "si de nouveaux crates ont été ajoutés depuis."
echo "════════════════════════════════════════════════════════════════════════"
