#!/usr/bin/env bash
# ci-seuls-artefacts.sh — dit à la CI si une pull request ne porte que des
# artefacts de travail (`_bmad-output/`), auquel cas ses jobs lourds se sautent
# (Story 33-11, #928). Toute la décision vit ici ; le YAML n'est que de la plomberie.
#
# Usage : ci-seuls-artefacts.sh <événement> <base> <tête>
#
# Écrit UNE SEULE ligne sur sa sortie, en dernier : `code=true` ou `code=false`.
# Le motif va sur la sortie d'erreur. ⛔ Le filtre échoue vers la CI complète :
# `code=false` ne sort que si l'événement est `pull_request` et que le diff, non
# vide et lu sans erreur, ne nomme que des fichiers sous `_bmad-output/`.
#
# - `--no-renames` : un renommage rend ses deux noms ; sans lui, le déplacement de
#   `crates/x.rs` vers `_bmad-output/x.rs` ne rendrait que le nom d'arrivée.
# - `-z` : les noms bruts ; sans lui, git cite un nom accentué, qui commencerait
#   alors par un guillemet.
# - `base...tête` (trois points) : ce que la PR apporte, comparé à la base de
#   fusion, même après une fusion de `main` dans sa branche.
#
# Testé par `scripts/tests/ci-seuls-artefacts.test.sh`, que le job `changes` lance
# avant lui.

set -uo pipefail
export LC_ALL=C

conclure() {
  echo "ci-seuls-artefacts : $2" >&2
  printf 'code=%s\n' "$1"
  exit 0
}

[ "$#" -eq 3 ] || conclure true "trois arguments attendus, $# reçus"
evenement=$1
base=$2
tete=$3

[ "$evenement" = pull_request ] || conclure true "événement « $evenement » : jamais filtré"
[ -n "$base" ] && [ -n "$tete" ] || conclure true "base ou tête vide"

liste=$(mktemp) || conclure true "fichier temporaire impossible"
trap 'rm -f "$liste"' EXIT

if ! git diff --name-only --no-renames -z "$base...$tete" -- > "$liste"; then
  conclure true "git diff en échec"
fi

[ -s "$liste" ] || conclure true "diff vide"

while IFS= read -r -d '' nom || [ -n "$nom" ]; do
  case "$nom" in
    _bmad-output/?*) ;;
    *) conclure true "hors du périmètre : $nom" ;;
  esac
done < "$liste"

conclure false "seulement des artefacts de travail"
