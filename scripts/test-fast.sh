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
# Le garde des droits de dev passe en premier (cf. docs/testing.md § « Base de
# dev jetable ») : il est sauté, en le disant, sans docker ou sans conteneur.
#
# MÉMOIRE : le gate se relance automatiquement dans un cgroup à mémoire bornée
# (`scripts/mem-guard.sh`), pour qu'un pic tue le gate et non le terminal — donc
# pas la session de travail qui l'a lancé. Cf. CLAUDE.md §« Plafonds mémoire ».
# Neutralisable par `KESH_NO_MEMGUARD=1`, et sans effet là où systemd utilisateur
# n'existe pas (CI, conteneur).
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Ré-exécution sous plafond mémoire. Le marqueur évite la récursion infinie.
if [ -z "${KESH_MEMGUARD_ACTIVE:-}" ] && [ -z "${KESH_NO_MEMGUARD:-}" ]; then
  export KESH_MEMGUARD_ACTIVE=1
  exec scripts/mem-guard.sh "$0" "$@"
fi

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

# Garde des droits du MariaDB de dev (Story 31-2) : les droits réels de `kesh`
# doivent être ceux du fichier versionné, sans quoi un test peut passer grâce à
# un privilège que la production n'a pas. Avant toute compilation, sous
# `--no-lint` comme sans ; s'il échoue, le gate s'arrête (`set -e`).
echo "▶ garde des droits du MariaDB de dev"
scripts/check-dev-grants.sh

if [ "$RUN_LINT" = 1 ]; then
  echo "▶ cargo fmt --check"
  cargo fmt --all -- --check
  echo "▶ cargo clippy (all-targets, -D warnings)"
  cargo clippy --workspace --all-targets -- -D warnings
fi

echo "▶ cargo nextest run (profil ${PROFILE} — lib kesh-db sérialisé, reste parallèle)"
cargo nextest run --profile "$PROFILE"

echo "✅ Gate backend rapide vert."
