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

check_perishable_exemptions() {
  # --- Exemptions du registre de rejeu dont le fondement SE PÉRIME ---
  #
  # Certaines entrées de `EXEMPT_MIGRATIONS` (crates/kesh-db/src/post_restore.rs)
  # ne s'appuient ni sur la fenêtre d'importabilité ni sur une propriété du schéma,
  # mais sur un FAIT DATÉ : « aucune version publiée ne se situe dans
  # [borne .. la migration exemptée) ». Publier une version dans cet intervalle
  # rend la justification fausse, et désactive le rejeu du backfill définitivement
  # et en silence. Le test `exemptions_claiming_out_of_window_really_are_out_of_window`
  # ne les couvre PAS : il ne contrôle que le fondement « Hors fenêtre ».
  #
  # ⛔ CE BLOC NE GREPE RIEN. Le registre est lu par du Rust, qui le connaît comme
  # une donnée (`ExemptionBasis::PerishableSince`). Sa version précédente cherchait
  # un marqueur textuel accentué dans le source et en déduisait la version par awk :
  # quatre modes d'échec en deux passes de revue, tous muets. Story 24-5 (#375),
  # findings P2-3 puis P3-2 / P3-3.

  # ⛔ L'ÉCHEC DE CETTE LECTURE EST FATAL, IL N'EST PAS « PAS D'EXEMPTION ».
  # Avec un `|| true`, un crate qui ne compile pas rendait la variable vide et le
  # rappel DISPARAISSAIT en silence — exactement le mode d'échec que tout ce
  # dispositif combat, pour la cinquième fois sur le même artefact. Éprouvé :
  # `cargo` neutralisé ⇒ 0 octet de sortie et `exit 0`. Relevé après la passe 4,
  # qui l'avait déclaré « robuste » sans l'exécuter.
  # stderr va dans un fichier À PART : le mêler à stdout (`2>&1`) ferait passer un
  # warning de compilation pour une ligne d'inventaire, que la boucle plus bas
  # lirait comme une exemption.
  ERR_PERISSABLES=$(mktemp)
  # `trap` RETURN : une interruption entre le mktemp et le rm -f laisserait
  # traîner le fichier. C'est le seul cas restant, aucune autre sortie n'existe
  # entre les deux (P5-9).
  trap 'rm -f "$ERR_PERISSABLES"' RETURN
  if ! PERISSABLES=$(cargo run -q -p kesh-db --example perishable_exemptions 2>"$ERR_PERISSABLES"); then
    echo
    echo "⛔ Impossible de lire l'inventaire des exemptions périssables :"
    sed 's/^/     /' "$ERR_PERISSABLES"
    echo "   Ce rappel est le SEUL contrôle de ces justifications. Ne pas poser de tag"
    echo "   tant qu'il n'a pas pu s'exécuter."
    rm -f "$ERR_PERISSABLES"
    exit 1
  fi
  rm -f "$ERR_PERISSABLES"

  if [ -z "$PERISSABLES" ]; then
    # ⛔ Le cas nominal PARLE, lui aussi. Tout le passif de cet artefact est
    # d'avoir été muet : un silence ne doit plus jamais valoir « rien à
    # signaler » (P5-9).
    echo "  ✓ aucune exemption à fondement périssable au registre."
    return 0
  fi
  {
    echo
    echo "⚠️  Exemption(s) de rejeu à fondement PÉRISSABLE dans post_restore.rs."
    echo "    Chacune repose sur « aucune version publiée dans tel intervalle » — un fait"
    echo "    que CETTE release peut rendre faux, silencieusement et définitivement."
    echo

    # La question décidable n'est pas « quel est le dernier tag ? » — celui-là
    # désigne la release précédente et ne dit rien de l'intervalle. Elle est :
    # UN TAG DÉJÀ PUBLIÉ SE SITUE-T-IL DANS L'INTERVALLE ? Un tag y est si son
    # arbre porte la migration de la borne basse mais PAS la migration exemptée.
    FAUTIVES=0
    while read -r version borne; do
      [ -z "$version" ] && continue
      echo "    Exemption $version — intervalle déclaré vide : [$borne .. $version)"
      trouve=0
      for tag in $(git tag --sort=-creatordate); do
        a_borne=$(git ls-tree -r --name-only "$tag" -- crates/kesh-db/migrations/ 2>/dev/null \
                  | grep -c "/${borne}_" || true)
        a_version=$(git ls-tree -r --name-only "$tag" -- crates/kesh-db/migrations/ 2>/dev/null \
                    | grep -c "/${version}_" || true)
        if [ "$a_borne" -gt 0 ] && [ "$a_version" -eq 0 ]; then
          echo "      ⛔ $tag ($(git log -1 --format=%cs "$tag" 2>/dev/null || echo '?')) EST DANS"
          echo "         L'INTERVALLE : la justification est FAUSSE. Le backfill ne sera jamais"
          echo "         rejoué chez qui a installé cette version puis restaure un backup."
          echo "         ⇒ inscrire la migration au registre POST_RESTORE_BACKFILLS, ou"
          echo "           re-motiver l'exemption — AVANT de poser le tag."
          trouve=1
          FAUTIVES=1
        fi
      done
      [ "$trouve" -eq 0 ] && echo "      ✓ aucun tag publié dans l'intervalle — la justification tient."
    done <<< "$PERISSABLES"

    if [ "$FAUTIVES" -eq 1 ]; then
      echo
      echo "⛔ Au moins une justification périssable est DÉMENTIE par un tag publié."
      exit 1
    fi
    echo
  }
}

# --- (0) PRÉ-VOL : tout ce qui peut refuser la release se vérifie AVANT de muter ---
#
# ⛔ Ce script MUTE PUIS VALIDE, et cela a un coût réel : à l'abandon il laisse
# les dix `Cargo.toml` bumpés et le dépôt sale, dans un état d'où il n'est même
# pas rejouable — la garde « working tree clean » refuse, et si on la contourne,
# `CURRENT_VERSION` relu depuis un Cargo.toml déjà bumpé déclenche « version
# cible identique ». C'est exactement ce qui est arrivé le 2026-09-09 : dix
# crates bumpés `0.11.1 → 0.12.0`, `CHANGELOG.md` intact, parce que l'étape 3
# ne trouvait pas son motif.
#
# D'où ce pré-vol. Les deux contrôles ci-dessous étaient à l'étape 3 et à la fin ;
# ils ne mutent rien et peuvent donc parler AVANT.
#
# *(Relevé en passe 5 de revue de code de la Story 24-5, #375, finding P5-6.)*

echo
echo "[0/3] Pré-vol"

# (0a) La section du CHANGELOG que l'étape 3 finalisera doit exister.
PATTERN="## [$NEW_VERSION] — Non publié"
if ! grep -qF "$PATTERN" CHANGELOG.md; then
    echo "ERREUR: pattern '$PATTERN' introuvable dans CHANGELOG.md." >&2
    echo "Le CHANGELOG doit contenir une section '$PATTERN' à finaliser." >&2
    echo "Vérifie que la section existe et que le texte exact match (espaces, tirets longs, etc.)." >&2
    echo "⇒ Refusé AVANT toute modification : le dépôt est intact." >&2
    exit 1
fi
echo "  ✓ CHANGELOG.md porte la section à finaliser."

# (0b) Exemptions du registre de rejeu dont le fondement SE PÉRIME.
check_perishable_exemptions
echo

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
# `$PATTERN` a été posé ET vérifié en pré-vol (0a) : rien à revalider ici, et
# surtout rien qui puisse encore refuser après que les Cargo.toml ont bougé.
REPLACEMENT="## [$NEW_VERSION] — $TODAY"

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
