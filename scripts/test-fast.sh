#!/usr/bin/env bash
# test-fast.sh — gate backend rapide via cargo-nextest.
#
# Équivaut au bloc « Backend (Rust) » de CLAUDE.md §Test Locally First, mais
# parallélise les suites d'intégration à base éphémère (`#[sqlx::test]`) au lieu
# du `--test-threads=1` global. Seuls les tests lib `kesh-db` (DB dev `kesh`
# partagée) restent sérialisés — via le test-group `shared-db-serial` de
# `.config/nextest.toml`.
#
# Usage :
#   scripts/test-fast.sh            # fmt + clippy + nextest (défaut)
#   scripts/test-fast.sh --no-lint  # nextest seul (itération rapide)
#   scripts/test-fast.sh --ci       # profil nextest ci (retries=1, fail-fast off)
#
# Pré-requis : MariaDB dev démarré + `cargo install cargo-nextest` (ou binaire
# prébuilt https://get.nexte.st). DATABASE_URL par défaut = conteneur dev local.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

: "${DATABASE_URL:=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh}"
export DATABASE_URL

PROFILE="default"
RUN_LINT=1
for arg in "$@"; do
  case "$arg" in
    --no-lint) RUN_LINT=0 ;;
    --ci)      PROFILE="ci" ;;
    -h|--help) grep '^#' "$0" | grep -v '^#!' | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "Argument inconnu : $arg" >&2; exit 2 ;;
  esac
done

if ! command -v cargo-nextest >/dev/null 2>&1; then
  echo "cargo-nextest introuvable. Installer : cargo install cargo-nextest" >&2
  echo "  (ou binaire prébuilt : curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ~/.cargo/bin)" >&2
  exit 3
fi

if [ "$RUN_LINT" = 1 ]; then
  echo "▶ cargo fmt --check"
  cargo fmt --all -- --check
  echo "▶ cargo clippy (all-targets, -D warnings)"
  cargo clippy --workspace --all-targets -- -D warnings
fi

echo "▶ cargo nextest run (profil ${PROFILE} — lib kesh-db sérialisé, reste parallèle)"
cargo nextest run --profile "$PROFILE"

echo "✅ Gate backend rapide vert."
