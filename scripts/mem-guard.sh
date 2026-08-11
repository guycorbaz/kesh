#!/usr/bin/env bash
# mem-guard.sh — exécute une commande dans un cgroup transitoire à mémoire bornée.
#
# POURQUOI CE SCRIPT EXISTE
# -------------------------
# Sur la station de dev, `systemd-oomd` surveille la tranche `user@1000.service`
# et, dès que sa *pression mémoire* dépasse 50 % pendant 20 s, il tue le cgroup
# descendant qui recycle le plus de pages. Or un terminal (onglet ptyxis, tmux,
# fenêtre Claude Code) est UN SEUL cgroup qui contient l'agent ET tous ses
# enfants : cargo, rustc, mold, node, chromium. Un build lourd condamne donc la
# session qui l'a lancé — l'agent meurt en même temps que son `cargo test`, et
# tout le contexte de travail est perdu.
#
# Le remède n'est pas de « compiler moins » (les plafonds `jobs`/`test-threads`
# aident, mais ne bornent rien : 4 rustc peuvent tenir 12 Go à eux seuls). Le
# remède est de sortir le travail lourd du cgroup du terminal, dans un scope à
# lui, avec un plafond dur. Le noyau étrangle puis tue DANS cette boîte ; la
# session, elle, survit.
#
# Précédent : 2026-08-11, deux OOM en six minutes (12:57 et 13:03) — scopes à
# 18,3 Go et 9 Go tués par oomd, sessions Claude Code emportées. Plusieurs
# fenêtres travaillaient en parallèle sur des projets différents et additionnaient
# leur pression sur la même tranche utilisateur ; la victime n'est pas forcément
# la fenêtre fautive.
#
# USAGE
# -----
#   scripts/mem-guard.sh <commande> [args...]
#   scripts/mem-guard.sh cargo build --workspace --all-targets
#   scripts/mem-guard.sh npm --prefix frontend run test:unit
#
#   scripts/mem-guard.sh --protect-shell    # marque le terminal courant
#                                           # « à épargner » pour oomd
#   scripts/mem-guard.sh --status           # ce que voit oomd, ici, maintenant
#
# PLAFONDS (surchargeables par l'environnement)
#   KESH_MEM_MAX=12G   plafond dur : au-delà, le noyau tue DANS le scope. C'est
#                      lui le garde-fou. Le processus sort en 137 (SIGKILL).
#   KESH_MEM_SWAP=2G   swap borné — c'est le thrashing en swap qui fabrique la
#                      pression que mesure oomd. Le seuil réel de mise à mort est
#                      donc MAX + SWAP.
#   KESH_MEM_HIGH=infinity
#                      étranglement doux, DÉSACTIVÉ PAR DÉFAUT — et ce n'est pas
#                      un oubli, c'est une mesure.
#
# ⚠️ NE PAS RÉACTIVER `MemoryHigh` SANS MESURER À NOUVEAU. Le réglage paraît
# évidemment souhaitable (« recycler dans le scope plutôt que tuer ») et il est
# nuisible ici. Mesures du 2026-08-11, allocation de 4 Go sous plafond de 512 Mo :
#
#   MemoryHigh=384M  MemoryMax=512M  swap=0     → aucune progression en 120 s,
#                                                 processus étranglé, jamais tué
#   MemoryHigh=448M  MemoryMax=512M  swap=256M  → idem : 124 (timeout)
#   MemoryHigh=infinity MemoryMax=512M swap=0   → 137 en 2 s, net
#   MemoryHigh=infinity MemoryMax=512M swap=256M→ 137 à ~768 Mo, net
#
# Sous `MemoryHigh`, le noyau pénalise chaque allocation par un sommeil
# proportionnel au retard du recyclage. Un build qui franchit le seuil ne tombe
# pas : il RAMPE, en tenant sa mémoire et en continuant d'alimenter la pression
# qu'oomd mesure. On obtient le pire des deux mondes — un gate qui paraît figé et
# une station toujours sous tension. Un échec net en 137 est un meilleur signal.
#
# DÉGRADATION : sans systemd utilisateur (CI, conteneur), la commande est
# exécutée telle quelle avec un avertissement. Le script n'échoue jamais pour la
# seule raison qu'il ne peut pas poser de plafond.
set -euo pipefail

: "${KESH_MEM_MAX:=12G}"
: "${KESH_MEM_SWAP:=2G}"
# Désactivé par défaut à dessein — cf. l'encadré « NE PAS RÉACTIVER » plus haut.
: "${KESH_MEM_HIGH:=infinity}"

# Le scope du terminal courant, tel qu'oomd le voit.
current_scope() {
  sed 's|.*/||' /proc/self/cgroup 2>/dev/null | head -1
}

have_systemd_user() {
  command -v systemd-run >/dev/null 2>&1 &&
    [ -n "${XDG_RUNTIME_DIR:-}" ] &&
    [ -S "${XDG_RUNTIME_DIR}/systemd/private" ]
}

case "${1:-}" in
  -h | --help)
    grep '^#' "$0" | grep -v '^#!' | sed 's/^# \{0,1\}//'
    exit 0
    ;;

  --protect-shell)
    # Marque le cgroup du terminal courant comme « à éviter » pour oomd. Le
    # réglage est transitoire : il vit le temps du scope (donc de l'onglet) et
    # disparaît à sa fermeture. À relancer dans chaque nouvelle fenêtre.
    #
    # ⚠️ Ne protéger QUE des terminaux interactifs, et toujours en tandem avec
    # l'exécution du travail lourd sous ce même script : si tout est « avoid »,
    # oomd n'a plus de candidat raisonnable et se rabat sur autre chose.
    scope="$(current_scope)"
    if ! have_systemd_user; then
      echo "systemd utilisateur indisponible — rien à protéger." >&2
      exit 0
    fi
    case "$scope" in
      *.scope) ;;
      *)
        echo "Le terminal n'est pas dans un scope systemd ($scope) — rien à faire." >&2
        exit 0
        ;;
    esac
    systemctl --user set-property "$scope" ManagedOOMPreference=avoid
    echo "✅ $scope marqué ManagedOOMPreference=avoid (jusqu'à la fermeture de l'onglet)."
    echo "   Lancer désormais les gates via : scripts/mem-guard.sh <commande>"
    echo "   Contrepartie : si un travail lourd tourne DANS ce terminal sans passer"
    echo "   par mem-guard, oomd l'épargnera et frappera ailleurs — possiblement une"
    echo "   autre fenêtre innocente. Protéger le terminal et guarder les gates vont"
    echo "   ensemble ; l'un sans l'autre déplace le problème au lieu de le régler."
    echo "   Annuler : systemctl --user set-property $scope ManagedOOMPreference=none"
    exit 0
    ;;

  --status)
    echo "▶ cgroup du terminal : $(current_scope)"
    if have_systemd_user; then
      systemctl --user show "$(current_scope)" -p ManagedOOMPreference 2>/dev/null || true
    fi
    echo "▶ politique oomd sur la tranche utilisateur :"
    systemctl show user@"$(id -u)".service \
      -p ManagedOOMMemoryPressure -p ManagedOOMMemoryPressureLimit 2>/dev/null ||
      echo "  (illisible)"
    echo "▶ mémoire :"
    free -h | sed 's/^/  /'
    echo "▶ derniers OOM (7 jours) :"
    journalctl --since "7 days ago" --no-pager 2>/dev/null |
      grep -E 'Killed /user|killed .* process' | tail -5 |
      sed 's/^/  /' || echo "  (aucun)"
    exit 0
    ;;

  "")
    echo "Usage : scripts/mem-guard.sh <commande> [args...]   (--help pour le détail)" >&2
    exit 2
    ;;
esac

if ! have_systemd_user; then
  echo "⚠️  systemd utilisateur indisponible — exécution SANS plafond mémoire." >&2
  exec "$@"
fi

echo "▶ mem-guard : MemoryMax=${KESH_MEM_MAX} MemorySwapMax=${KESH_MEM_SWAP} (MemoryHigh=${KESH_MEM_HIGH})"

# --collect : le scope est nettoyé même s'il meurt en échec (sinon il reste en
# état « failed » et le nom est pris au run suivant).
# Le code de sortie de la commande est propagé tel quel par systemd-run --scope.
exec systemd-run --user --scope --quiet --collect \
  --unit "kesh-memguard-$$" \
  -p MemoryHigh="${KESH_MEM_HIGH}" \
  -p MemoryMax="${KESH_MEM_MAX}" \
  -p MemorySwapMax="${KESH_MEM_SWAP}" \
  -- "$@"
