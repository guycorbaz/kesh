#!/usr/bin/env bash
# mem-guard.sh — exécute une commande dans un cgroup transitoire à mémoire bornée.
#
# DÉLÉGATION (depuis le 2026-10-27)
# ---------------------------------
# Ce script délègue à l'outil GLOBAL de la station, `mem-guard` (dans le PATH,
# ~/.local/bin/mem-guard), qui porte désormais la logique et sa documentation
# complète (`mem-guard --help`) : mêmes options (--status, --protect-shell),
# mêmes variables (KESH_MEM_MAX, KESH_MEM_SWAP, KESH_MEM_HIGH restent lues), et
# en plus un plafond du NOMBRE d'exécutions simultanées sur toute la station,
# tous projets confondus (MEM_GUARD_SLOTS, 2 par défaut ; la suivante attend).
#
# Pourquoi global : le 2026-10-27, deux sessions Claude Code d'un AUTRE projet
# (mybibli) ont été tuées par systemd-oomd pendant qu'un gate kesh tournait sous
# ce script. Chaque projet plafonnait ses propres gates, mais rien ne bornait la
# somme : la pression s'additionne sur la même tranche utilisateur. Le verrou
# `.gate.lock` (docs/agents-orchestration.md) sérialise les gates de kesh ; les
# places de `mem-guard` bornent l'ensemble de la station.
#
# Plafond par défaut de l'outil global : 10G + 1G de swap (12G + 2G ici
# auparavant). Mesure de référence (CLAUDE.md § « Plafonds mémoire ») : build à
# froid du workspace vert sous 8 Go, pic de mémoire anonyme 1,9 Go.
#
# REPLI : sans l'outil global (autre machine), le script pose lui-même un
# plafond, sans limite de simultanéité ; sans systemd utilisateur (CI,
# conteneur), il exécute la commande telle quelle. Il n'échoue jamais pour la
# seule raison qu'il ne peut pas poser de plafond.
#
# ⚠️ `MemoryHigh` reste désactivé (infinity) : mesuré nuisible le 2026-08-11,
# cf. CLAUDE.md § « `MemoryHigh` est un piège ».
#
# USAGE
#   scripts/mem-guard.sh <commande> [args...]
#   scripts/mem-guard.sh --status | --protect-shell | --help
set -euo pipefail

if command -v mem-guard >/dev/null 2>&1; then
  exec mem-guard "$@"
fi

: "${KESH_MEM_MAX:=12G}"
: "${KESH_MEM_SWAP:=2G}"
: "${KESH_MEM_HIGH:=infinity}"

case "${1:-}" in
  "" | -h | --help | --status | --protect-shell)
    echo "Outil global \`mem-guard\` introuvable dans le PATH : seule l'exécution" >&2
    echo "plafonnée est disponible ici — scripts/mem-guard.sh <commande> [args...]." >&2
    [ -z "${1:-}" ] && exit 2
    exit 0
    ;;
esac

if ! command -v systemd-run >/dev/null 2>&1 ||
  [ -z "${XDG_RUNTIME_DIR:-}" ] ||
  [ ! -S "${XDG_RUNTIME_DIR}/systemd/private" ]; then
  echo "⚠️  systemd utilisateur indisponible — exécution SANS plafond mémoire." >&2
  exec "$@"
fi

echo "▶ mem-guard (repli local) : MemoryMax=${KESH_MEM_MAX} MemorySwapMax=${KESH_MEM_SWAP}" >&2
exec systemd-run --user --scope --quiet --collect \
  --unit "kesh-memguard-$$" \
  -p MemoryHigh="${KESH_MEM_HIGH}" \
  -p MemoryMax="${KESH_MEM_MAX}" \
  -p MemorySwapMax="${KESH_MEM_SWAP}" \
  -- "$@"
