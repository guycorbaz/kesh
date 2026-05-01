#!/bin/sh
# Install Git hooks from scripts/hooks/ into .git/hooks/.
#
# Usage : bash scripts/install-hooks.sh
#
# À exécuter une fois après clone du repo, et après tout ajout/update
# d'un hook dans scripts/hooks/.
#
# Pourquoi pas core.hooksPath ? Parce qu'il pointe vers un chemin du
# worktree, et un hook versionné n'existe que sur les branches qui le
# contiennent — main pré-merge n'est pas protégée. Copier dans
# .git/hooks/ rend le hook indépendant de la branche checkout.

set -e

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "❌ Pas dans un repo git." >&2
  exit 1
}

src="$repo_root/scripts/hooks"
dst="$repo_root/.git/hooks"

if [ ! -d "$src" ]; then
  echo "❌ $src introuvable." >&2
  exit 1
fi

mkdir -p "$dst"

for hook in "$src"/*; do
  [ -f "$hook" ] || continue
  name=$(basename "$hook")
  cp "$hook" "$dst/$name"
  chmod +x "$dst/$name"
  echo "✓ installed: $name"
done

echo ""
echo "Done. Hooks installed in .git/hooks/ (not versioned)."
echo "Re-run this script after any change to scripts/hooks/."
