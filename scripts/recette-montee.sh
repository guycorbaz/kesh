#!/usr/bin/env bash
# recette-montee.sh — joue la montée de version et le retour, de bout en bout, sur
# des données synthétiques (Story 33-10, #925 ; B3 de la rétrospective 0.17.4).
#
#   scripts/recette-montee.sh --image gcorbaz/kesh:X.Y.Z --binaire target/debug/kesh-api \
#     --static frontend/build --port-api 3190 --port-base 3391 \
#     --migrations-attendues N --diff-attendu migrations,supplier_invoices \
#     [--apres ../kesh-gate-logs/relXYZ/verifs.sh] [--import] [--prefixe kesh-recette]
#
# Phases, chacune vérifiée, chacune son code de sortie :
#   1 Avant   (11) : l'image précédente sur une base neuve, peuplement (huit factures client,
#                    un avoir, un règlement groupé, quatre PDF figés, Story 49-1d), empreinte,
#                    `.keshbackup`, dump ; le dossier des documents de l'image copié par
#                    `docker cp`, puis le fichier d'une facture figée retiré (le regel).
#   2 Montée  (12) : le binaire neuf : N migrations, une sauvegarde `pre-upgrade/` si
#                    N > 0, journal sans erreur, seuls les fichiers de `--diff-attendu`
#                    diffèrent, et seulement par ajout (projection sur l'avant identique).
#                    Puis le crochet `--apres`, pour ce qui est propre à la release (il
#                    dispose de `SQL`, `api`, `L`). Puis les PDF : figés servis à l'octet,
#                    regel du fichier retiré, rendus gardés pour la phase 6.
#   3 Refus   (13) : si N > 0, l'image précédente refuse la base migrée.
#   4 Retour  (14) : dump restauré, image relancée, empreinte identique à celle d'avant ;
#                    puis les rendus de référence de l'image précédente.
#   5 Import  (15) : option `--import` : le `.keshbackup` dans le binaire neuf, base neuve,
#                    dossier des documents neuf : les PDF figés viennent de l'archive.
#   6 Rendus  (16) : chaque PDF rendu de nouveau par le binaire neuf a, page à page, la même
#                    trame à 300 dpi, le même texte et le même nombre de pages que sa
#                    référence ; exactement 5 paires.
#
# Limite assumée : le retour joué ici passe par le dump. La procédure du manuel, par le
# `.keshbackup` de `pre-upgrade/` dans une installation neuve, relève d'une autre recette.
#
# ⛔ Une liste blanche, non une liste noire : ce script ne connaît aucun accès réel, et
# n'en nomme aucun. Son environnement est fermé (il se relance par `env -i`, `HOME`
# neutralisé), il refuse tout démon Docker non local, sa base est un conteneur
# `mariadb:10.11` qu'il crée lui-même, il n'écrit que dans un dossier `mktemp -d` sous
# `/tmp`, qu'il nomme à sa dernière ligne et n'efface pas ; aucune option ne désigne un
# hôte, une base, une URL ni un lieu d'écriture. Il n'emploie pas le MariaDB de dev. Les
# verrous de la station et le plafond mémoire restent à l'appelant.
#
# ⚠️ Ce périmètre est celui du script, NON du crochet `--apres` : le crochet est du code de
# l'appelant, exécuté avec les droits du script et avec ses mots de passe factices. Il part
# de `$D/cwd`, `HOME` sous `$D`, et tourne sous `set -e`, avec ses exceptions : un échec dans
# une condition (`if`, `&&`, `||`), dans une fonction appelée d'une condition ou dans un
# `local x=$(…)` ne l'arrête pas. Rien ne l'empêche d'atteindre `docker`, le réseau ou un autre
# dossier : il se relit comme du code.
# L'empreinte ne couvre pas tout non plus : `docs/testing.md` dit exactement ce qu'elle
# regarde. Les documents : les PDF des factures peuplées sont comparés (archivés à l'octet,
# rendus par leur dessin, phases 2, 5 et 6) ; aucune autre pièce ne l'est.
#
# Codes : 2 option ou entrée refusée ; 3 `.env` atteignable ; 4 conteneur de recette déjà
# présent ; 5 outil manquant, démon non local, image indisponible ou conteneur de base
# inattendu.
#
# Variables de test (les seules qui traversent la relance) : KESH_DOCKER, KESH_CURL,
# KESH_MUTOOL, RECETTE_RACINE_JETABLE (sous /tmp). Qui les pose exécute déjà ce qu'il veut.

set -uo pipefail

# --- L'environnement fermé -----------------------------------------------------------
# Aucun marqueur ne fait sauter la comparaison : l'environnement se compare à la liste, et le
# script se relance, une fois, s'il diffère. Une variable exportée sous un nom de marqueur ne
# fait donc rien sauter.
# La liste vient de `env`, non de `compgen -e`, qui ne voit pas les fonctions exportées.
# `HOME` est neutralisé : ni la configuration de `docker` (contexte, `credsStore`), ni celle de
# `curl`, ni celle de `jq`.
PATH_FERME=/usr/bin:/bin
HOME_FERME=/nonexistent
environnement_ferme() {
  local e
  [ "${PATH:-}" = "$PATH_FERME" ] || return 1
  [ "${HOME:-}" = "$HOME_FERME" ] || return 1
  while IFS= read -r -d '' e; do
    case "${e%%=*}" in
      PATH | HOME | KESH_DOCKER | KESH_CURL | KESH_MUTOOL | RECETTE_RACINE_JETABLE) ;;
      PWD | SHLVL | _ | OLDPWD) ;;
      *) return 1 ;;
    esac
  done < <(/usr/bin/env -0)
  return 0
}
# Une seule relance : le premier argument `--relance-faite`, que seule la relance pose, ne fait
# rien sauter — l'environnement se compare encore — mais interdit de relancer une seconde fois.
# Sans lui, un bash qui exporterait de lui-même une variable hors de la liste relancerait sans fin.
if ! environnement_ferme; then
  if [ "${1:-}" = --relance-faite ]; then
    echo "recette-montee : refus : environnement non fermable, même après relance" >&2
    exit 5
  fi
  relance=("PATH=$PATH_FERME" "HOME=$HOME_FERME")
  [ -z "${KESH_DOCKER:-}" ] || relance+=("KESH_DOCKER=$KESH_DOCKER")
  [ -z "${KESH_CURL:-}" ] || relance+=("KESH_CURL=$KESH_CURL")
  [ -z "${KESH_MUTOOL:-}" ] || relance+=("KESH_MUTOOL=$KESH_MUTOOL")
  [ -z "${RECETTE_RACINE_JETABLE:-}" ] || relance+=("RECETTE_RACINE_JETABLE=$RECETTE_RACINE_JETABLE")
  exec /usr/bin/env -i "${relance[@]}" /usr/bin/bash "$0" --relance-faite "$@"
fi
[ "${1:-}" != --relance-faite ] || shift

DOCKER=${KESH_DOCKER:-docker}
CURL=${KESH_CURL:-curl}
MUTOOL=${KESH_MUTOOL:-mutool}
RACINE="$(cd "$(dirname "$0")/.." && pwd)"

refus() {
  echo "recette-montee : refus : $1" >&2
  exit "${2:-2}"
}

# --- Les options : tout ce qui n'est pas nommé ici est refusé ---------------------------
IMAGE="" BINAIRE="" STATIQUE="" PORT_API="" PORT_BASE="" MIGRATIONS="" DIFF_ATTENDU="" APRES="" IMPORT=0
PREFIXE=kesh-recette
valeur() { [ "$#" -ge 2 ] || refus "$1 sans valeur"; }
while [ "$#" -gt 0 ]; do
  case "$1" in
    --image) valeur "$@"; IMAGE=$2; shift 2 ;;
    --binaire) valeur "$@"; BINAIRE=$2; shift 2 ;;
    --static) valeur "$@"; STATIQUE=$2; shift 2 ;;
    --port-api) valeur "$@"; PORT_API=$2; shift 2 ;;
    --port-base) valeur "$@"; PORT_BASE=$2; shift 2 ;;
    --migrations-attendues) valeur "$@"; MIGRATIONS=$2; shift 2 ;;
    --diff-attendu) valeur "$@"; DIFF_ATTENDU=$2; shift 2 ;;
    --apres) valeur "$@"; APRES=$2; shift 2 ;;
    --prefixe) valeur "$@"; PREFIXE=$2; shift 2 ;;
    --import) IMPORT=1; shift ;;
    *) refus "option inconnue : $1" ;;
  esac
done

# --- Les contrôles, tous avant la moindre action ----------------------------------------
[[ "$IMAGE" =~ ^gcorbaz/kesh:[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9.]+)?$ ]] \
  || refus "image « $IMAGE » : gcorbaz/kesh:X.Y.Z exigée"
[[ "$PREFIXE" =~ ^kesh-[a-z0-9]+$ ]] || refus "préfixe « $PREFIXE » hors du motif kesh-<mot>"
[[ "$MIGRATIONS" =~ ^(0|[1-9][0-9]*)$ ]] || refus "--migrations-attendues : un entier, sans zéro de tête"

for outil in jq md5sum sha256sum awk ss realpath join diff "$MUTOOL" "$CURL" "$DOCKER"; do
  command -v "$outil" >/dev/null || refus "outil manquant : $outil" 5
done

port_valide() { [[ "$1" =~ ^[1-9][0-9]*$ ]] && [ "$1" -ge 1024 ] && [ "$1" -le 65535 ]; }
port_valide "$PORT_API" || refus "--port-api : entier de 1024 à 65535"
port_valide "$PORT_BASE" || refus "--port-base : entier de 1024 à 65535"
[ "$PORT_API" != "$PORT_BASE" ] || refus "--port-api et --port-base sont égaux"
PORTS=("$PORT_API" "$PORT_BASE")
if [ "$IMPORT" = 1 ]; then
  PORT_BASE_2=$((PORT_BASE + 1))
  port_valide "$PORT_BASE_2" || refus "--import : --port-base + 1 hors de 1024 à 65535"
  [ "$PORT_API" != "$PORT_BASE_2" ] || refus "--import : --port-base + 1 égale --port-api"
  PORTS+=("$PORT_BASE_2")
fi
# ⛔ Jamais `… | grep -q` sous `pipefail` : sa sortie à la première ligne tue en `SIGPIPE` ce
# qui écrit encore, le tube vaut 141, et un « trouvé » se lit « absent ».
PORTS_PRIS=$(ss -ltn 2>/dev/null | awk '{print $4}' | sed 's/.*://') || refus "ss en échec" 5
for p in "${PORTS[@]}"; do
  if /usr/bin/grep -qx "$p" <<<"$PORTS_PRIS"; then
    refus "port $p déjà pris"
  fi
done

# Une entrée en lecture seule : son `realpath -e` doit rester sous `lieu`. `..` et lien
# symbolique ne passent donc pas.
sous() {
  local nom=$1 chemin=$2 lieu=$3 reel
  reel=$(cd "$RACINE" && realpath -e -- "$chemin" 2>/dev/null) || refus "$nom « $chemin » introuvable"
  case "$reel" in
    "$lieu"/*) printf '%s\n' "$reel" ;;
    *) refus "$nom « $chemin » hors de sa place" ;;
  esac
}
[ -n "$BINAIRE" ] || refus "--binaire manque"
[ -n "$STATIQUE" ] || refus "--static manque"
BINAIRE=$(sous --binaire "$BINAIRE" "$RACINE") || exit 2
STATIQUE=$(sous --static "$STATIQUE" "$RACINE") || exit 2
[ -f "$BINAIRE" ] && [ -x "$BINAIRE" ] || refus "--binaire : pas un fichier exécutable"
[ -d "$STATIQUE" ] || refus "--static : pas un dossier"
if [ -n "$APRES" ]; then
  VOISIN=$(realpath -m -- "$RACINE/../kesh-gate-logs")
  APRES=$(sous --apres "$APRES" "$VOISIN") || exit 2
fi

RACINE_JETABLE=/tmp
if [ -n "${RECETTE_RACINE_JETABLE:-}" ]; then
  RACINE_JETABLE=$(realpath -e -- "$RECETTE_RACINE_JETABLE" 2>/dev/null) \
    || refus "RECETTE_RACINE_JETABLE introuvable"
  case "$RACINE_JETABLE" in
    /tmp | /tmp/*) ;;
    *) refus "RECETTE_RACINE_JETABLE hors de /tmp" ;;
  esac
fi

# `dotenvy` cherche un `.env` dans le répertoire courant du binaire et ses parents : aucun
# ne doit l'être, ce qui rend sa lecture sans objet.
d=$RACINE_JETABLE
while :; do
  [ ! -e "$d/.env" ] || refus "« $d/.env » atteignable par le binaire" 3
  [ "$d" != / ] || break
  d=$(dirname "$d")
done

# Le démon visé : un socket local, seul. Le script ne lit aucun fichier de configuration : il
# demande à `docker`, et ne dit pas ce qu'il a vu.
HOTE_DOCKER=$("$DOCKER" context inspect --format '{{.Endpoints.docker.Host}}' 2>/dev/null) \
  || refus "contexte Docker illisible" 5
case "$HOTE_DOCKER" in
  unix:///*) ;;
  *) refus "le démon Docker visé n'est pas local : la recette ne joue que sur cette machine" 5 ;;
esac

restes=$("$DOCKER" ps -a --filter label=kesh.recette --format '{{.Names}}') \
  || refus "docker ps en échec" 5
if [ -n "$restes" ]; then
  refus "conteneurs d'un run précédent, ou d'un run en cours : $restes. Pour retirer un reste : docker stop <nom>" 4
fi

# Tirées d'avance : un tirage dans `docker run` se confondrait avec une image qui ne répond
# pas, et laisserait un client qu'aucun `docker stop` n'atteint.
for image in mariadb:10.11 "$IMAGE"; do
  if ! "$DOCKER" image inspect "$image" >/dev/null 2>&1; then
    "$DOCKER" pull "$image" >/dev/null 2>&1 || refus "image $image indisponible" 5
  fi
done

# --- Le dossier jetable : le script n'écrit que là ----------------------------------------
D=$(mktemp -d "$RACINE_JETABLE/kesh-recette.XXXXXX") || refus "dossier jetable impossible" 5
mkdir -p "$D/documents" "$D/documents-import" "$D/inbox" "$D/backup" "$D/cwd" "$D/home" "$D/journal" "$D/pdf"
L=$D/journal
# Les PDF téléchargés, un fichier par pièce et par phase : `$P/<phase>-<pièce>.pdf`.
P=$D/pdf
if [ -n "$APRES" ]; then
  cp -- "$APRES" "$D/apres.sh" || refus "crochet illisible" 2
fi

# Valeurs factices : celles de `docs/testing.md`, littérales et sans portée.
ADMIN_NOM=admin
ADMIN_MDP=e2e-admin-password-12chars
JWT=dev-secret-at-least-32-bytes-long-for-testing
SMTP_HOTE=smtp.invalid
SMTP_NOM=e2e
SMTP_MDP=e2e
SMTP_DE=kesh@example.invalid
MDP_ROOT=recette_root
MDP_KESH=recette_kesh
BASE=kesh_recette

JETON=$$-$RANDOM
CT_BASE=$PREFIXE-db-$$
CT_BASE_2=$PREFIXE-db2-$$
CT_IMAGE=$PREFIXE-image-$$
URL=http://127.0.0.1:$PORT_API
PID_BINAIRE=""
PID_IMAGE=""

# --- La sortie, sur tout chemin ----------------------------------------------------------
# Arrête un conteneur par son nom, s'il porte le jeton de ce run, et lui seul : un homonyme
# n'est pas à nous. `--rm` le retire.
arreter_conteneur() { # $1 = nom
  local vu
  vu=$("$DOCKER" inspect -f '{{index .Config.Labels "kesh.recette"}}' "$1" 2>/dev/null) || return 0
  [ "$vu" = "$JETON" ] || return 0
  "$DOCKER" stop "$1" >/dev/null 2>&1
}
# Le client `docker run` attaché ne sort pas sur SIGTERM : il le relaie au conteneur, où
# `kesh-api`, PID 1 sans gestionnaire de signal, l'ignore. Seul SIGKILL l'arrête à coup sûr.
attendre_fin() { # $1 = PID, $2 = nombre de pas de 0,2 s ; rend 1 s'il vit encore
  local i
  for i in $(seq 1 "$2"); do
    kill -0 "$1" 2>/dev/null || return 0
    sleep 0.2
  done
  ! kill -0 "$1" 2>/dev/null
}
arreter_image() {
  if [ -n "$PID_IMAGE" ]; then
    arreter_conteneur "$CT_IMAGE"
    # L'attente est bornée, `wait` compris : 20 s après `docker stop`, puis SIGTERM et 2 s,
    # puis SIGKILL. Le `wait` ne vient qu'une fois le client mort.
    if ! attendre_fin "$PID_IMAGE" 100; then
      kill "$PID_IMAGE" 2>/dev/null
      if ! attendre_fin "$PID_IMAGE" 10; then
        kill -KILL "$PID_IMAGE" 2>/dev/null
        attendre_fin "$PID_IMAGE" 10
      fi
      # Le conteneur, lui, peut survivre à son client, ou n'avoir été créé qu'après le premier
      # `docker stop` : un second essai, puis son nom, pour le retirer à la main.
      arreter_conteneur "$CT_IMAGE"
      echo "recette-montee : client docker de l'image tué ; si le conteneur reste : docker stop $CT_IMAGE" >&2
    fi
    wait "$PID_IMAGE" 2>/dev/null
    PID_IMAGE=""
  fi
}
arreter_binaire() {
  if [ -n "$PID_BINAIRE" ]; then
    kill "$PID_BINAIRE" 2>/dev/null
    wait "$PID_BINAIRE" 2>/dev/null
    PID_BINAIRE=""
  fi
}
nettoyer() {
  local code=$?
  trap - EXIT
  arreter_binaire
  arreter_image
  arreter_conteneur "$CT_BASE"
  arreter_conteneur "$CT_BASE_2"
  # Les seuls ports de la recette : le reste de la station n'a rien à faire au journal.
  # `PORT_BASE_2` n'existe qu'avec `--import` : sinon, le motif vide ne retient rien.
  ss -ltn 2>/dev/null | awk -v api=":$PORT_API\$" -v base=":$PORT_BASE\$" -v base2="${PORT_BASE_2:+:$PORT_BASE_2\$}" \
    '$4 ~ api || $4 ~ base || (base2 != "" && $4 ~ base2)' > "$L/ss-final.txt"
  [ ! -s "$L/ss-final.txt" ] || echo "recette-montee : un port de la recette écoute encore ($L/ss-final.txt)" >&2
  # Sur la sortie d'origine : un `echec` peut survenir sous une redirection de son appelant.
  echo "recette-montee : code $code, dossier $D" >&3
  exit "$code"
}
# Le descripteur 3 ne se transmet à aucun processus de fond (`3>&-` aux lancements) : un
# appelant qui lit la sortie par un tube attendrait sinon la fin de tout survivant.
exec 3>&1
trap nettoyer EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
echec() {
  echo "recette-montee : ÉCHEC phase $1 : $2" >&2
  exit "$((10 + $1))"
}

# --- La base -----------------------------------------------------------------------------
lancer_base() { # $1 = nom, $2 = port hôte
  "$DOCKER" run --rm -d --name "$1" --label "kesh.recette=$JETON" --memory=2g \
    --tmpfs /var/lib/mysql -p "127.0.0.1:$2:3306" \
    -e "MARIADB_ROOT_PASSWORD=$MDP_ROOT" -e "MARIADB_DATABASE=$BASE" \
    -e "MARIADB_USER=kesh" -e "MARIADB_PASSWORD=$MDP_KESH" \
    mariadb:10.11 >/dev/null || return 1
  local vu
  vu=$("$DOCKER" inspect -f '{{index .Config.Labels "kesh.recette"}}' "$1") || return 1
  [ "$vu" = "$JETON" ] || {
    echo "le conteneur $1 ne porte pas le jeton attendu" >&2
    exit 5
  }
  local i
  for i in $(seq 1 90); do
    # Par TCP : le serveur temporaire de l'initialisation n'écoute pas sur ce port.
    "$DOCKER" exec -e "MYSQL_PWD=$MDP_KESH" "$1" mariadb -h127.0.0.1 -ukesh -N "$BASE" -e "SELECT 1" >/dev/null 2>&1 && return 0
    sleep 1
  done
  return 1
}
# DATABASE_URL se construit ici, et seulement ici.
url_base() { printf 'mysql://kesh:%s@127.0.0.1:%s/%s' "$MDP_KESH" "$1" "$BASE"; }

CT_COURANT=$CT_BASE
SQL() {
  "$DOCKER" exec -i -e "MYSQL_PWD=$MDP_ROOT" "$CT_COURANT" mariadb -uroot -N "$BASE" -e "$1"
}

# --- L'API -------------------------------------------------------------------------------
# Tout appel est borné : un serveur qui accepte sans répondre ne fige ni une attente ni une
# phase, et un SIGTERM reçu pendant un appel est traité au plus tard à son terme. Un appel
# plus long (export, import) repasse `--max-time` : pour `curl`, la dernière valeur l'emporte.
C() { "$CURL" -q --noproxy '*' -s --connect-timeout 5 --max-time 60 "$@"; }
TOKEN=""
attendre_sante() { # $1 = PID du processus servi
  local i
  for i in $(seq 1 750); do
    C -f --max-time 5 "$URL/health" >/dev/null 2>&1 && return 0
    kill -0 "$1" 2>/dev/null || return 1
    sleep 0.2
  done
  return 1
}
login() {
  local p
  for p in "$ADMIN_MDP" admin123; do
    TOKEN=$(C -f -X POST "$URL/api/v1/auth/login" -H 'Content-Type: application/json' \
      -d "{\"username\":\"$ADMIN_NOM\",\"password\":\"$p\"}" | jq -r .accessToken 2>/dev/null)
    [ -n "$TOKEN" ] && [ "$TOKEN" != null ] && return 0
  done
  return 1
}
# Un refus de l'API rend un statut non nul, et garde le corps pour le diagnostic.
api() {
  local m=$1 p=$2
  shift 2
  C --fail-with-body -X "$m" "$URL$p" -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' "$@"
}
# Un identifiant rendu par l'API, ou la phase échoue.
id_de() { # $1 = phase, $2 = étiquette, $3 = JSON
  local v
  v=$(printf '%s' "$3" | jq -r '.id // empty' 2>/dev/null)
  [ -n "$v" ] || echec "$1" "$2 : pas d'identifiant dans la réponse : $(printf '%s' "$3" | head -c 300)"
  printf '%s\n' "$v"
}
camt() { # $1 = preview|create, $2 = fichier, $3 = compte, reste = champs -F
  local quoi=$1 f=$2 compte=$3 u=$URL/api/v1/bank-imports
  shift 3
  [ "$quoi" != preview ] || u=$u/preview
  C -X POST "$u" -H "Authorization: Bearer $TOKEN" -F "file=@$f;type=application/xml" -F "bankAccountId=$compte" "$@"
}

# --- Les serveurs ------------------------------------------------------------------------
# L'image précédente : aucun chemin de l'hôte, aucune variable héritée, `-e NOM=VALEUR`.
lancer_image() { # $1 = suffixe du journal
  "$DOCKER" run --rm --name "$CT_IMAGE" --label "kesh.recette=$JETON" --network host \
    -e "DATABASE_URL=$(url_base "$PORT_BASE")" \
    -e "KESH_HOST=127.0.0.1" -e "KESH_PORT=$PORT_API" -e "KESH_COOKIE_SECURE=false" \
    -e "KESH_TEST_MODE=true" \
    -e "KESH_INBOX_DIR=/tmp/inbox" -e "KESH_DOCUMENTS_DIR=/tmp/documents" -e "KESH_ADMIN_BACKUP_DIR=/tmp/backup" \
    -e "KESH_ADMIN_USERNAME=$ADMIN_NOM" -e "KESH_ADMIN_PASSWORD=$ADMIN_MDP" -e "KESH_JWT_SECRET=$JWT" \
    -e "KESH_SMTP_HOST=$SMTP_HOTE" -e "KESH_SMTP_USER=$SMTP_NOM" -e "KESH_SMTP_PASSWORD=$SMTP_MDP" -e "KESH_SMTP_FROM=$SMTP_DE" \
    "$IMAGE" > "$L/image-$1.log" 2>&1 3>&- &
  PID_IMAGE=$!
}
# Le binaire neuf : liste exacte, depuis `$D/cwd`. Tout ce qu'il écrit est sous `$D` ; il ne lit
# hors de là que ses textes et son frontend, dans le dépôt.
lancer_binaire() { # $1 = suffixe du journal, $2 = port de la base, $3 = dossier des documents (sous `$D`)
  (cd "$D/cwd" && exec /usr/bin/env -i \
    "PATH=$PATH_FERME" "HOME=$D/home" \
    "DATABASE_URL=$(url_base "$2")" \
    "KESH_HOST=127.0.0.1" "KESH_PORT=$PORT_API" "KESH_COOKIE_SECURE=false" \
    "KESH_STATIC_DIR=$STATIQUE" "KESH_LOCALES_DIR=$RACINE/crates/kesh-i18n/locales" \
    "KESH_INBOX_DIR=$D/inbox" "KESH_DOCUMENTS_DIR=$3" "KESH_ADMIN_BACKUP_DIR=$D/backup" \
    "KESH_ADMIN_USERNAME=$ADMIN_NOM" "KESH_ADMIN_PASSWORD=$ADMIN_MDP" "KESH_JWT_SECRET=$JWT" \
    "KESH_SMTP_HOST=$SMTP_HOTE" "KESH_SMTP_USER=$SMTP_NOM" "KESH_SMTP_PASSWORD=$SMTP_MDP" "KESH_SMTP_FROM=$SMTP_DE" \
    "$BINAIRE") > "$L/binaire-$1.log" 2>&1 < /dev/null 3>&- &
  PID_BINAIRE=$!
}

# --- L'empreinte -------------------------------------------------------------------------
# Ce qu'elle couvre est dit à `docs/testing.md` : y tenir la liste à jour.
TABLES="journal_entries journal_entry_lines invoices invoice_lines invoice_settlements supplier_invoices supplier_invoice_lines bank_imports bank_transactions bank_accounts contacts accounts companies fiscal_years company_invoice_settings credit_notes credit_note_lines"
AUDIT_N=0
AUDIT_DECALAGE=0
# Les colonnes d'une table, relevées à l'empreinte `avant`, dans l'ordre de `SELECT *`, chaque nom
# entre apostrophes inverses (`CHAR(96)`) : la projection de D2 (Story 49-1d) les relit telles quelles.
COLONNES=$L/colonnes-avant
est_table() { case " $TABLES " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }
empreinte() { # $1 = étiquette
  local o=$L/empreinte-$1 j=$L/projection-$1 t id
  mkdir -p "$o" "$j"
  SQL "SELECT version, description, success, HEX(checksum) FROM _sqlx_migrations ORDER BY version" > "$o/migrations" || return 1
  if [ "$1" = avant ]; then
    mkdir -p "$COLONNES"
    for t in $TABLES; do
      SQL "SET SESSION group_concat_max_len = 65535; SELECT GROUP_CONCAT(CONCAT(CHAR(96), COLUMN_NAME, CHAR(96)) ORDER BY ORDINAL_POSITION) FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = '$t'" > "$COLONNES/$t" || return 1
      [ -s "$COLONNES/$t" ] && [ "$(cat "$COLONNES/$t")" != NULL ] || return 1
    done
  fi
  for t in $TABLES; do
    # `ORDER BY 1` : la première colonne, `id` ou `company_id` (réglages de facturation).
    SQL "SELECT * FROM $t ORDER BY 1" > "$o/$t" || return 1
    # Projection sur les colonnes d'avant, pour une table que `--diff-attendu` nomme : hors de
    # l'empreinte (son dossier propre), comparée par `ecarts_hors_attendu`. Une colonne d'avant
    # retirée ou renommée fait échouer la requête, donc la phase : voulu.
    if [ "$1" != avant ] && attendu_en_diff "$t"; then
      SQL "SELECT $(cat "$COLONNES/$t") FROM $t ORDER BY 1" > "$j/$t" || return 1
    fi
  done
  # `users`, sans ce que réécrit l'avis « Recovery effectué » à chaque démarrage.
  SQL "SELECT id, username, role, active, company_id, email FROM users ORDER BY id" > "$o/users" || return 1
  # `audit_log`, les `AUDIT_N` premières entrées, sans leur numéro : celles d'avant la montée.
  # La suite s'écrit, légitimement, et l'import d'une sauvegarde ajoute ses entrées, dans
  # l'ordre, après celles de l'installation qui la reçoit (`AUDIT_DECALAGE`, relevé juste avant
  # l'import). Compter, et non borner par numéro : un numéro sauté (rollback) ne décale rien.
  # Hypothèse restante : aucune entrée écrite entre ce relevé et la restauration.
  [ "$1" != avant ] || AUDIT_N=$(SQL "SELECT COUNT(*) FROM audit_log") || return 1
  SQL "SELECT user_id, action, entity_type, entity_id, details_json, created_at, actor_type, actor_api_key_id, actor_label, company_id FROM audit_log WHERE id > $AUDIT_DECALAGE ORDER BY id LIMIT $AUDIT_N" > "$o/audit_log" || return 1
  SQL "SELECT kesh_version_min_required FROM _kesh_version" > "$o/min_required" || return 1
  login || return 1
  : > "$o/api"
  for id in $(cat "$L/ids-factures"); do api GET "/api/v1/invoices/$id" | jq -S . >> "$o/api" || return 1; done
  for id in $(cat "$L/ids-fournisseur"); do api GET "/api/v1/supplier-invoices/$id" | jq -S . >> "$o/api" || return 1; done
  for id in $(cat "$L/ids-avoirs"); do api GET "/api/v1/credit-notes/$id" | jq -S . >> "$o/api" || return 1; done
  # Le rapport TVA de la période du peuplement (D10) : le seul lecteur filtré par le type de
  # ligne que la recette fait tourner sur les données d'avant. Gardé aussi à part, pour le
  # contrôle de non-vacuité de la phase 1.
  api GET "/api/v1/reports/vat?$(cat "$L/requete-tva")" | jq -S . > "$L/tva-$1.json" || return 1
  cat "$L/tva-$1.json" >> "$o/api"
  (cd "$o" && md5sum -- *) | awk '{print $2, $1}' | sort > "$L/empreinte-$1.md5"
}
# Les fichiers qui diffèrent entre deux empreintes, un par ligne.
ecarts() { # $1, $2 = étiquettes
  join -a1 -a2 -e ABSENT -o 0,1.2,2.2 "$L/empreinte-$1.md5" "$L/empreinte-$2.md5" | awk '$2 != $3 {print $1}'
}
attendu_en_diff() {
  case ",$DIFF_ATTENDU," in *",$1,"*) return 0 ;; *) return 1 ;; esac
}
# `api`, réduit aux clés d'avant (D2) : chaque document d'après, projeté sur le document d'avant de
# même rang, récursivement — objets réduits aux clés d'avant (une clé disparue est marquée),
# tableaux élément par élément à longueur égale, même type JSON exigé. La sortie est la projection,
# un document par ligne de `jq -S`, à comparer à l'avant normalisé de même façon.
JQ_PROJECTION='
def proj($a):
  . as $b
  | if ($a | type) != ($b | type) then {"ÉCART DE TYPE": [($a | type), ($b | type)]}
    elif ($a | type) == "object" then
      reduce ($a | keys_unsorted[]) as $k ({};
        . + {($k): (if ($b | has($k)) then ($b[$k] | proj($a[$k])) else {"CLÉ DISPARUE": $k} end)})
    elif ($a | type) == "array" then
      if ($a | length) != ($b | length) then {"ÉCART DE LONGUEUR": [($a | length), ($b | length)]}
      else [range(0; $a | length) as $i | ($b[$i] | proj($a[$i]))] end
    else $b end;
if ($av | length) != ($ap | length) then {"ÉCART DU NOMBRE DE DOCUMENTS": [($av | length), ($ap | length)]}
else range(0; $av | length) as $i | ($ap[$i] | proj($av[$i])) end'
# Rend 0 si la projection de `$3` (étiquette d'après) sur `$2` (avant) égale l'avant ; écrit le diff
# de la projection au journal sinon.
projection_identique() { # $1 = fichier, $2, $3 = étiquettes
  local avant=$L/empreinte-$2/$1 j=$L/projection-$3 n=$L/projection-$3/$1.avant
  if [ "$1" = api ]; then
    jq -S -c --slurpfile av "$avant" --slurpfile ap "$L/empreinte-$3/api" -n "$JQ_PROJECTION" > "$j/api" || return 1
    jq -S -c . "$avant" > "$n" || return 1
  else
    [ -f "$j/$1" ] || return 1
    cp -- "$avant" "$n"
  fi
  diff -- "$n" "$j/$1" > "$L/diff-projection-$3-$1.txt" && return 0
  return 1
}
ecarts_hors_attendu() { # $1 = phase, $2, $3 = étiquettes
  local e faute=""
  for e in $(ecarts "$2" "$3"); do
    if ! attendu_en_diff "$e"; then
      faute="$faute $e"
    elif { est_table "$e" || [ "$e" = api ]; } && ! projection_identique "$e" "$2" "$3"; then
      # Story 49-1d (D2) : une table ou `api` nommés ne peuvent différer que par ajout.
      faute="$faute $e (valeurs d'avant changées : $L/diff-projection-$3-$e.txt)"
    fi
  done
  [ -z "$faute" ] || echec "$1" "écart hors de --diff-attendu ($2 → $3) :$faute"
}
journal_sans_erreur() { # $1 = phase, $2 = journal
  # « Recovery effectué » est au niveau ERROR à dessein : c'est l'avis du compte d'administration
  # réinitialisé par les variables du script, attendu ici sur toute base peuplée.
  # ⛔ Capturé, jamais `| grep -q` : sous `pipefail`, le `SIGPIPE` de ce qui écrit encore ferait
  # lire un journal long, à l'erreur en tête, comme un journal sans erreur.
  [ -r "$2" ] || echec "$1" "journal illisible : $2"
  local erreurs
  erreurs=$(sed 's/\x1b\[[0-9;]*m//g' "$2" | /usr/bin/grep -v 'Recovery effectué' | /usr/bin/grep -E 'panicked at| ERROR ')
  [ -z "$erreurs" ] || echec "$1" "erreur au journal $2"
}
nb_migrations() { SQL "SELECT COUNT(*) FROM _sqlx_migrations"; }

# --- Les PDF (Story 49-1d) ---------------------------------------------------------------
# Les pièces peuplées, une par ligne de `$L/pieces` : « <pièce> <identifiant> <numéro> », pièce
# F1 à F8 (factures) ou A1 (avoir). Les numéros sont relus à l'API.
id_piece() { awk -v p="$1" '$1 == p {print $2}' "$L/pieces"; }
numero_piece() { awk -v p="$1" '$1 == p {print $3}' "$L/pieces"; }
chemin_pdf() { # $1 = pièce
  case "$1" in
    A*) printf '/api/v1/credit-notes/%s/pdf' "$(id_piece "$1")" ;;
    *) printf '/api/v1/invoices/%s/pdf' "$(id_piece "$1")" ;;
  esac
}
pdf_valide() { [ -s "$1" ] && [ "$(head -c 5 -- "$1")" = "%PDF-" ]; }
empreinte_sha() { sha256sum -- "$1" | awk '{print $1}'; }
# Un téléchargement : le statut HTTP et le corps, gardé tel quel (un refus JSON reste lisible).
telecharger() { # $1 = chemin d'API, $2 = fichier ; rend le statut HTTP sur la sortie
  local code
  code=$(C -o "$2" -w '%{http_code}' "$URL$1" -H "Authorization: Bearer $TOKEN") || code=${code:-000}
  printf '%s' "$code"
}
# Un PDF exigé : HTTP 200 et en-tête `%PDF-`, sinon la phase échoue en nommant la pièce.
pdf_exige() { # $1 = phase, $2 = pièce, $3 = fichier
  local code
  code=$(telecharger "$(chemin_pdf "$2")" "$3")
  [ "$code" = 200 ] && pdf_valide "$3" \
    || echec "$1" "PDF de $2 : HTTP $code, en-tête « $(head -c 5 -- "$3" 2>/dev/null | tr -cd '[:print:]') » — pas un PDF (corps gardé : $3)"
}
# Un rendu gardé pour la phase 6 : un échec n'arrête pas la phase, la paire ne comptera pas.
pdf_rendu() { # $1 = pièce, $2 = fichier
  local code
  code=$(telecharger "$(chemin_pdf "$1")" "$2")
  pdf_valide "$2" && [ "$code" = 200 ] || echo "  ⚠️ rendu de $1 : HTTP $code, pas un PDF ($2) : la paire ne comptera pas"
}
# Une facture figée, servie par le serveur en cours : les octets de la phase 1, et l'empreinte en
# base. $1 = phase, $2 = pièce, $3 = étiquette du fichier.
pdf_fige_identique() {
  local f=$P/$3-$2.pdf
  pdf_exige "$1" "$2" "$f"
  cmp -s -- "$P/avant-$2.pdf" "$f" || echec "$1" "PDF archivé de $2 : octets différents de ceux de la phase 1 ($f)"
  [ "$(empreinte_sha "$f")" = "$(SQL "SELECT pdf_sha256 FROM invoices WHERE id = $(id_piece "$2")")" ] \
    || echec "$1" "PDF archivé de $2 : SHA-256 différent de pdf_sha256 en base"
}

# --- Phase 1 : avant ---------------------------------------------------------------------
echo "▶ phase 1 : avant ($IMAGE)"
lancer_base "$CT_BASE" "$PORT_BASE" || echec 1 "base injoignable"
lancer_image avant
attendre_sante "$PID_IMAGE" || echec 1 "l'image ne répond pas ($L/image-avant.log)"
MIGRATIONS_AVANT=$(nb_migrations)
C -f -X POST "$URL/api/v1/_test/seed" -H 'Content-Type: application/json' -d '{"preset":"with-company"}' >/dev/null \
  || echec 1 "_test/seed en échec"
login || echec 1 "connexion impossible"
JOUR=$(date +%F)
COMPTES=$(api GET /api/v1/accounts) || echec 1 "lecture du plan comptable en échec : $COMPTES"
compte_numero() { printf '%s' "$COMPTES" | jq -r --arg n "$1" '[.[]|select(.number==$n)][0].id // empty'; }
CAISSE=$(compte_numero 1000)
CREANCIERS=$(compte_numero 2000)
CHARGE=$(printf '%s' "$COMPTES" | jq -r '[.[]|select(.accountType=="Expense" and .active and .postable)][0].id // empty')
[ -n "$CAISSE" ] && [ -n "$CREANCIERS" ] && [ -n "$CHARGE" ] \
  || echec 1 "plan comptable inattendu"

# Un appel du peuplement : tout refus de l'API fait échouer la phase, avec son corps.
peupler() { # $1 = étiquette, reste = arguments d'`api` ; rend le corps sur la sortie
  local etiquette=$1 corps
  shift
  corps=$(api "$@") || echec 1 "$etiquette : refus de l'API : $(printf '%s' "$corps" | head -c 300)"
  printf '%s' "$corps"
}
# Story 49-1d (D1) — deux préconditions d'abord : sans compte bancaire PRINCIPAL, aucun PDF de
# facture ne se rend ; sans compte de différences d'arrondi désigné, aucune facture à l'arrondi non
# nul ne se valide. Le compte bancaire est lié au 1020 (le règlement groupé l'exige).
BANQUE=$(compte_numero 1020)
if [ -z "$BANQUE" ]; then
  R=$(peupler "compte 1020" POST /api/v1/accounts -d '{"number":"1020","name":"Banque de recette","accountType":"Asset","parentId":null}') || exit $?
  BANQUE=$(id_de 1 "compte 1020" "$R") || exit $?
fi
ARRONDI=$(printf '%s' "$COMPTES" | jq -r '[.[]|select((.name|test("rrondi";"i")) and .active and .postable)][0].id // empty')
[ -n "$ARRONDI" ] || ARRONDI=$CHARGE
# L'IBAN d'exemple du relevé synthétique.
IBAN=CH4431999123000889012
R=$(peupler "lecture des comptes bancaires" GET /api/v1/bank-accounts) || exit $?
COMPTE=$(printf '%s' "$R" | jq -r --arg i "$IBAN" '[.[]|(.account // .)|select(.iban==$i)][0].id // empty')
if [ -z "$COMPTE" ]; then
  R=$(peupler "compte bancaire" POST /api/v1/bank-accounts -d "{\"bankName\":\"Banque Synthetique\",\"iban\":\"$IBAN\",\"isPrimary\":true,\"journalAccountId\":$BANQUE}") || exit $?
  COMPTE=$(id_de 1 "compte bancaire" "$(printf '%s' "$R" | jq -c '(.account // .)' 2>/dev/null)") || exit $?
else
  R=$(printf '%s' "$R" | jq -c --arg i "$IBAN" --argjson b "$BANQUE" '[.[]|(.account // .)|select(.iban==$i)][0] | .isPrimary=true | .journalAccountId=$b') \
    || echec 1 "compte bancaire existant illisible"
  printf '%s' "$R" > "$L/compte-bancaire.json"
  peupler "compte bancaire rendu principal" PUT "/api/v1/bank-accounts/$COMPTE" --data-binary "@$L/compte-bancaire.json" >/dev/null || exit $?
fi
R=$(peupler "lecture des réglages de facturation" GET /api/v1/company/invoice-settings) || exit $?
R=$(printf '%s' "$R" | jq --argjson p "$CREANCIERS" --argjson a "$ARRONDI" \
  '.defaultPayableAccountId=$p | .defaultRoundingAccountId=$a | .roundTo5Centimes=true') || echec 1 "réglages de facturation illisibles"
printf '%s' "$R" > "$L/reglages-facturation.json"
peupler "écriture des réglages de facturation" PUT /api/v1/company/invoice-settings --data-binary "@$L/reglages-facturation.json" >/dev/null || exit $?
[ "$(SQL "SELECT COUNT(*) FROM bank_accounts WHERE is_primary = TRUE AND archived = FALSE")" = 1 ] \
  && [ "$(SQL "SELECT COUNT(*) FROM bank_accounts WHERE is_primary = TRUE AND archived = FALSE AND journal_account_id = $BANQUE")" = 1 ] \
  || echec 1 "précondition : pas exactement un compte bancaire principal, lié au 1020"
[ "$(SQL "SELECT COUNT(*) FROM company_invoice_settings WHERE default_rounding_account_id IS NOT NULL")" = 1 ] \
  || echec 1 "précondition : aucun compte de différences d'arrondi désigné"

R=$(peupler client POST /api/v1/contacts -d '{"contactType":"Personne","name":"Synthetique","firstName":"Pia","lastName":"Synthetique","isClient":true,"isSupplier":false,"addressStructured":{"street":"Marktgasse","building":"28","postalCode":"9400","city":"Rorschach","country":"CH"},"defaultPaymentTerms":"30 jours net"}') || exit $?
CLIENT=$(id_de 1 client "$R") || exit $?
# Huit factures (D1), dans l'ordre de leurs pièces : F1 à F8. Les désignations de F2 à F8 restent
# dans ce que la 49-4b ne change pas (au plus 44 caractères, ASCII imprimable, espaces simples,
# aucune de bord) ; celle de F1, figée, dépasse 44 caractères.
ligne() { printf '{"description":"%s","quantity":"%s","unitPrice":"%s","vatRate":"%s"}' "$1" "$2" "$3" "$4"; }
: > "$L/ids-factures"
: > "$L/pieces-ids"
creer_facture() { # $1 = pièce, $2 = lignes JSON
  local r v
  r=$(peupler "facture $1" POST /api/v1/invoices -d "{\"contactId\":$CLIENT,\"date\":\"$JOUR\",\"dueDate\":\"$JOUR\",\"paymentTerms\":\"30 jours net\",\"lines\":$2}") || exit $?
  v=$(id_de 1 "facture $1" "$r") || exit $?
  printf '%s\n' "$v" >> "$L/ids-factures"
  printf '%s %s\n' "$1" "$v" >> "$L/pieces-ids"
}
creer_facture F1 "[$(ligne "Conseil en organisation comptable, seance du matin et de l apres-midi" 4.5 200.00 8.10)]"
creer_facture F2 "[$(ligne Conseil 1 200.00 8.10)]"
creer_facture F3 "[$(ligne Conseil 2 200.00 8.10)]"
creer_facture F4 "[$(ligne Conseil 1 100.00 8.10),$(ligne Livres 1 33.33 2.60),$(ligne "Frais hors champ" 1 10.01 0.00)]"
creer_facture F5 "[$(ligne Accompagnement 1 150.00 8.10)]"
creer_facture F6 "[$(ligne Accompagnement 2 150.00 8.10)]"
creer_facture F7 "[$(ligne Conseil 1 300.00 8.10),$(ligne Documentation 2 25.00 2.60)]"
creer_facture F8 "[$(ligne Formation 1 120.00 8.10)]"
piece_id() { awk -v p="$1" '$1 == p {print $2}' "$L/pieces-ids"; }
F1=$(piece_id F1) F2=$(piece_id F2) F4=$(piece_id F4) F5=$(piece_id F5) F6=$(piece_id F6) F7=$(piece_id F7) F8=$(piece_id F8)
for f in F1 F2 F4 F5 F6 F7 F8; do
  peupler "validation de $f" POST "/api/v1/invoices/$(piece_id "$f")/validate" >/dev/null || exit $?
done
R=$(peupler "lecture de la facture 2" GET "/api/v1/invoices/$F2") || exit $?
DU=$(printf '%s' "$R" | jq -r '.remainingAmount // .amountDue // .totalTtc // empty')
[ -n "$DU" ] || echec 1 "reste dû de la facture 2 illisible"
# Story 34-5 (#951) : la version de la facture est exigée depuis la 0.17.6 ; une
# image antérieure l'ignore (le corps n'y refuse pas les champs inconnus).
VF2=$(printf '%s' "$R" | jq -r '.version // empty')
[[ "$VF2" =~ ^[0-9]+$ ]] || echec 1 "version de la facture 2 illisible"
peupler "règlement de la facture 2" POST "/api/v1/invoices/$F2/settlements" -d "{\"settlementType\":\"internal_account\",\"accountId\":$CAISSE,\"amount\":\"$DU\",\"settledOn\":\"$JOUR\",\"version\":$VF2}" >/dev/null || exit $?
# Les PDF figés en 0.19.0 (D1) : F1, F5, F7 — avant son avoir, F7 se fige validée — et F8, dont le
# fichier sera retiré de la copie des documents.
for f in F1 F5 F7 F8; do pdf_exige 1 "$f" "$P/avant-$f.pdf"; done
R=$(peupler "avoir de F7" POST /api/v1/credit-notes -d "{\"invoiceId\":$F7,\"date\":\"$JOUR\"}") || exit $?
A1=$(id_de 1 "avoir de F7" "$R") || exit $?
printf '%s\n' "$A1" > "$L/ids-avoirs"
printf 'A1 %s\n' "$A1" >> "$L/pieces-ids"

R=$(peupler fournisseur POST /api/v1/contacts -d '{"contactType":"Entreprise","name":"Fournisseur Synthetique SA","isClient":false,"isSupplier":true}') || exit $?
FOURNISSEUR=$(id_de 1 fournisseur "$R") || exit $?
: > "$L/ids-fournisseur"
for a in 123.45 80.00 55.50 10.00; do
  R=$(peupler "facture fournisseur" POST /api/v1/supplier-invoices -d "{\"contactId\":$FOURNISSEUR,\"supplierInvoiceNumber\":\"F-$a\",\"invoiceDate\":\"$JOUR\",\"dueDate\":null,\"paymentReference\":null,\"lines\":[{\"description\":\"Fourniture\",\"quantity\":\"1\",\"unitPrice\":\"$a\",\"vatRate\":\"0\",\"expenseAccountId\":$CHARGE}]}") || exit $?
  V=$(id_de 1 "facture fournisseur" "$R") || exit $?
  printf '%s\n' "$V" >> "$L/ids-fournisseur"
done
for n in 1 2 3; do
  SI=$(sed -n "${n}p" "$L/ids-fournisseur")
  R=$(peupler "lecture de la facture fournisseur $SI" GET "/api/v1/supplier-invoices/$SI") || exit $?
  V=$(printf '%s' "$R" | jq .version)
  peupler "validation de la facture fournisseur $SI" POST "/api/v1/supplier-invoices/$SI/validate" -d "{\"version\":$V}" >/dev/null || exit $?
done
SI2=$(sed -n 2p "$L/ids-fournisseur")
SI3=$(sed -n 3p "$L/ids-fournisseur")
peupler "paiement de la facture fournisseur" POST "/api/v1/supplier-invoices/$SI2/pay" -d "{\"settlementType\":\"internal_account\",\"accountId\":$CAISSE,\"paymentDate\":\"$JOUR\"}" >/dev/null || exit $?
peupler "annulation de la facture fournisseur" POST "/api/v1/supplier-invoices/$SI3/cancel" >/dev/null || exit $?

# L'état réel, relu en base : c'est lui que l'empreinte va comparer.
SQL "SELECT GROUP_CONCAT(CONCAT(id,':',status) ORDER BY id) FROM supplier_invoices" > "$L/statuts-fournisseur.txt"
[ "$(cat "$L/statuts-fournisseur.txt")" = "1:open,2:paid,3:cancelled,4:draft" ] \
  || echec 1 "statuts fournisseur inattendus : $(cat "$L/statuts-fournisseur.txt")"

camt preview "$RACINE/scripts/fixtures/recette/releve-A.xml" "$COMPTE" > "$L/apercu-A.json"
camt create "$RACINE/scripts/fixtures/recette/releve-A.xml" "$COMPTE" \
  -F "previewFingerprint=$(jq -r .previewFingerprint "$L/apercu-A.json")" > "$L/import-A.json"
[ "$(SQL 'SELECT COUNT(*) FROM bank_transactions')" = 2 ] || echec 1 "relevé synthétique non importé ($L/import-A.json)"

# Le règlement groupé de F5 et F6 (D1) : un relevé saisi à la main au jour de la recette — un
# crédit égal à la somme de leurs TTC — puis la proposition `invoice_group` acceptée. Le relevé A,
# daté d'octobre 2026, ne peut régler aucune facture du jour.
GROUPE=$(SQL "SELECT SUM(total_amount) FROM invoices WHERE id IN ($F5, $F6)")
[[ "$GROUPE" =~ ^[0-9]+\.[0-9]{2}$ ]] || echec 1 "TTC de F5 et F6 illisible : $GROUPE"
peupler "relevé saisi du règlement groupé" POST /api/v1/bank-imports/manual -d "{\"bankAccountId\":$COMPTE,\"periodFrom\":\"$JOUR\",\"periodTo\":\"$JOUR\",\"statementNumber\":\"R-GROUPE\",\"openingBalance\":\"0.00\",\"closingBalance\":\"$GROUPE\",\"lines\":[{\"bookingDate\":\"$JOUR\",\"valueDate\":null,\"details\":\"Versement groupe\",\"counterpartyName\":null,\"reference\":null,\"amount\":\"$GROUPE\",\"direction\":\"credit\"}],\"confirmDuplicateFile\":true,\"confirmDuplicateLines\":true,\"confirmOpeningBalanceGap\":true}" > "$L/releve-groupe.json" || exit $?
[ "$(SQL 'SELECT COUNT(*) FROM bank_transactions')" = 3 ] || echec 1 "relevé du règlement groupé non saisi ($L/releve-groupe.json)"
TX=$(SQL "SELECT id FROM bank_transactions WHERE amount > 0 ORDER BY id DESC LIMIT 1")
[[ "$TX" =~ ^[0-9]+$ ]] || echec 1 "transaction du règlement groupé introuvable"
peupler "règlement groupé de F5 et F6" POST /api/v1/reconciliation/accept -d "{\"bankAccountId\":$COMPTE,\"proposals\":[{\"type\":\"invoice_group\",\"bankTransactionId\":$TX,\"invoiceIds\":[$F5,$F6]}]}" > "$L/groupe.json" || exit $?
jq -e '(.failed|length) == 0 and (.accepted|length) == 1' "$L/groupe.json" >/dev/null || echec 1 "règlement groupé refusé ($L/groupe.json)"

# L'état réel, relu en base, pièce par pièce (D1).
SQL "SELECT GROUP_CONCAT(status ORDER BY id) FROM invoices" > "$L/statuts-clients.txt"
[ "$(cat "$L/statuts-clients.txt")" = "validated,validated,draft,validated,validated,validated,cancelled,validated" ] \
  || echec 1 "statuts clients inattendus : $(cat "$L/statuts-clients.txt")"
[ "$(SQL 'SELECT COUNT(*) FROM invoice_settlements')" = 3 ] || echec 1 "règlements clients : 3 attendus (F2, F5, F6)"
[ "$(SQL 'SELECT COUNT(*) FROM invoice_settlements WHERE journal_entry_line_id IS NOT NULL')" = 2 ] \
  || echec 1 "règlement groupé : 2 règlements à ligne d'écriture désignée attendus"
[ "$(SQL 'SELECT COUNT(*) FROM credit_notes')" = 1 ] || echec 1 "avoir absent"
[ "$(SQL 'SELECT GROUP_CONCAT(id ORDER BY id) FROM invoices WHERE pdf_sha256 IS NOT NULL')" = "$F1,$F5,$F7,$F8" ] \
  || echec 1 "factures figées : F1, F5, F7, F8 attendues"
[ "$(SQL "SELECT CONCAT(COUNT(*), ':', COUNT(DISTINCT vat_rate), ':', SUM(vat_rate = 0)) FROM invoice_lines WHERE invoice_id = $F4")" = "3:3:1" ] \
  || echec 1 "F4 : trois lignes, trois taux dont 0.00 attendus"
[ "$(SQL "SELECT rounding_amount <> 0 FROM invoices WHERE id = $F4")" = 1 ] || echec 1 "F4 : arrondi nul"
[ "$(SQL "SELECT COUNT(*) FROM invoice_lines WHERE invoice_id = $F1 AND CHAR_LENGTH(description) > 44")" = 1 ] \
  || echec 1 "F1 : désignation de plus de 44 caractères attendue"
[ "$(SQL "SELECT COUNT(*) FROM invoice_lines WHERE invoice_id <> $F1")" -gt 0 ] \
  && [ "$(SQL "SELECT (SELECT COUNT(*) FROM invoice_lines WHERE invoice_id <> $F1 AND (CHAR_LENGTH(description) > 44 OR description NOT REGEXP '^[!-~]+( [!-~]+)*\$')) + (SELECT COUNT(*) FROM credit_note_lines WHERE CHAR_LENGTH(description) > 44 OR description NOT REGEXP '^[!-~]+( [!-~]+)*\$')")" = 0 ] \
  || echec 1 "une désignation de F2 à F8 ou de l'avoir sort de la forme de D1 (44 caractères, ASCII imprimable, espaces simples)"
# Les numéros, relus à l'API : une paire de rendus ne compte que si son texte contient le sien.
: > "$L/pieces"
while read -r piece id; do
  case "$piece" in
    A*) num=$(api GET "/api/v1/credit-notes/$id" | jq -r '.creditNoteNumber // empty') ;;
    *) num=$(api GET "/api/v1/invoices/$id" | jq -r '.invoiceNumber // "brouillon"') ;;
  esac
  [ -n "$num" ] || echec 1 "numéro de $piece illisible"
  printf '%s %s %s\n' "$piece" "$id" "$num" >> "$L/pieces"
done < "$L/pieces-ids"
# Le rapport TVA de la période du peuplement (D10), sur l'exercice du jour.
EXERCICE=$(SQL "SELECT id FROM fiscal_years WHERE start_date <= '$JOUR' AND end_date >= '$JOUR' ORDER BY id LIMIT 1")
[[ "$EXERCICE" =~ ^[0-9]+$ ]] || echec 1 "aucun exercice ne couvre le $JOUR"
printf 'fiscalYearId=%s&periodStart=%s&periodEnd=%s' "$EXERCICE" "$JOUR" "$JOUR" > "$L/requete-tva"

empreinte avant || echec 1 "empreinte avant"
# AC8 : un rapport vide des deux côtés ne prouverait rien.
jq -e '(.totalBaseHt | tonumber) != 0' "$L/tva-avant.json" >/dev/null 2>&1 \
  || echec 1 "rapport TVA sans chiffre d'affaires ($L/tva-avant.json)"
C -f --max-time 600 -o "$D/avant.keshbackup" "$URL/api/v1/admin/full-export" -H "Authorization: Bearer $TOKEN" \
  || echec 1 "export .keshbackup en échec"
[ -s "$D/avant.keshbackup" ] || echec 1 ".keshbackup vide"
# Le dossier des documents de l'image (D3), copié par `docker cp` — aucun montage —, depuis le seul
# conteneur qui porte le jeton de ce run ; puis le fichier de F8 retiré, pour le regel de la phase 2.
# La sauvegarde ci-dessus, prise avant, le garde.
[ "$("$DOCKER" inspect -f '{{index .Config.Labels "kesh.recette"}}' "$CT_IMAGE" 2>/dev/null)" = "$JETON" ] \
  || echec 1 "le conteneur de l'image ne porte pas le jeton du run"
"$DOCKER" cp "$CT_IMAGE:/tmp/documents/." "$D/documents/" >/dev/null || echec 1 "docker cp des documents en échec"
for f in F1 F5 F7 F8; do
  CHEMIN=$(SQL "SELECT pdf_storage_path FROM invoices WHERE id = $(piece_id "$f")")
  [[ "$CHEMIN" =~ ^[0-9a-f]{64}\.pdf$ ]] && [ -f "$D/documents/$CHEMIN" ] \
    || echec 1 "PDF de $f absent de la copie des documents (« $CHEMIN »)"
  [ "$f" != F8 ] || rm -f -- "$D/documents/$CHEMIN"
done
arreter_image
"$DOCKER" exec -e "MYSQL_PWD=$MDP_ROOT" "$CT_BASE" mariadb-dump -uroot --single-transaction --routines --triggers \
  --add-drop-database --databases "$BASE" > "$D/avant.sql" || echec 1 "dump en échec"
[ -s "$D/avant.sql" ] || echec 1 "dump vide"
echo "  $MIGRATIONS_AVANT migrations, empreinte et sauvegardes prises"

# --- Phase 2 : montée --------------------------------------------------------------------
echo "▶ phase 2 : montée (binaire neuf, $MIGRATIONS migration(s) attendue(s))"
lancer_binaire montee "$PORT_BASE" "$D/documents"
attendre_sante "$PID_BINAIRE" || echec 2 "le binaire ne répond pas ($L/binaire-montee.log)"
# Sa version, que relit le script complémentaire de la release (nom de la sauvegarde `pre-upgrade/`).
C -f --max-time 5 -o "$L/health-binaire.json" "$URL/health" || echec 2 "/health du binaire illisible"
MIGRATIONS_APRES=$(nb_migrations)
[ "$MIGRATIONS_APRES" -eq "$((MIGRATIONS_AVANT + MIGRATIONS))" ] \
  || echec 2 "$MIGRATIONS_AVANT migrations avant, $MIGRATIONS_APRES après, $MIGRATIONS attendue(s)"
NB_PRE=$(find "$D/backup/pre-upgrade" -type f 2>/dev/null | wc -l)
if [ "$MIGRATIONS" -gt 0 ]; then
  [ "$NB_PRE" -ge 1 ] || echec 2 "aucune sauvegarde dans $D/backup/pre-upgrade"
else
  [ "$NB_PRE" -eq 0 ] || echec 2 "une sauvegarde pre-upgrade sans migration"
fi
empreinte apres || echec 2 "empreinte après"
ecarts_hors_attendu 2 avant apres
if [ -n "$APRES" ]; then
  # Sans `||` à droite : il neutraliserait le `set -e` du sous-shell, et seul le dernier
  # statut du crochet compterait.
  # shellcheck disable=SC1091
  (
    cd "$D/cwd" || exit 1
    HOME=$D/home
    set -e
    . "$D/apres.sh"
  ) > "$L/apres.log" 2>&1
  CODE_APRES=$?
  [ "$CODE_APRES" -eq 0 ] || echec 2 "crochet --apres en échec, code $CODE_APRES ($L/apres.log)"
fi
# Les PDF, après l'empreinte et le crochet (Story 49-1d, D4 a, AC4) : une facture figée est servie à
# l'octet de la phase 1 ; celle dont le fichier manque (F8) rend `410`, se regèle (`200`, une réponse
# JSON de facture), puis sert le PDF du regel, à l'empreinte de son nouveau `pdf_sha256`. Les pièces
# jamais figées en 0.19.0 sont rendues et gardées pour la phase 6.
login || echec 2 "connexion impossible"
for f in F1 F5 F7; do pdf_fige_identique 2 "$f" apres; done
CODE=$(telecharger "$(chemin_pdf F8)" "$P/apres-F8-manquant.json")
[ "$CODE" = 410 ] || echec 2 "F8, fichier retiré : HTTP $CODE au lieu de 410 ($P/apres-F8-manquant.json)"
CODE=$(C -o "$L/regel-F8.json" -w '%{http_code}' -X POST "$URL$(chemin_pdf F8)/refreeze" -H "Authorization: Bearer $TOKEN") || CODE=${CODE:-000}
[ "$CODE" = 200 ] && jq -e '.id' "$L/regel-F8.json" >/dev/null 2>&1 \
  || echec 2 "regel de F8 : HTTP $CODE ($L/regel-F8.json)"
pdf_exige 2 F8 "$P/apres-F8-regel.pdf"
[ "$(empreinte_sha "$P/apres-F8-regel.pdf")" = "$(SQL "SELECT pdf_sha256 FROM invoices WHERE id = $(id_piece F8)")" ] \
  || echec 2 "PDF du regel de F8 : SHA-256 différent du nouveau pdf_sha256"
for f in F2 F4 F6 A1; do pdf_rendu "$f" "$P/apres-$f.pdf"; done
# Lu en dernier : ce qu'empreinte et crochet ont fait écrire au binaire compte aussi.
journal_sans_erreur 2 "$L/binaire-montee.log"
arreter_binaire

# --- Phase 3 : refus ---------------------------------------------------------------------
if [ "$MIGRATIONS" -gt 0 ]; then
  echo "▶ phase 3 : l'image précédente refuse la base migrée"
  lancer_image refus
  i=0
  while kill -0 "$PID_IMAGE" 2>/dev/null && [ "$i" -lt 300 ]; do
    if C -f --max-time 5 "$URL/health" >/dev/null 2>&1; then
      echec 3 "l'image démarre sur la base migrée"
    fi
    sleep 0.2
    i=$((i + 1))
  done
  kill -0 "$PID_IMAGE" 2>/dev/null && echec 3 "l'image n'est pas sortie en 60 s"
  wait "$PID_IMAGE"
  CODE_IMAGE=$?
  PID_IMAGE=""
  [ "$CODE_IMAGE" -ne 0 ] || echec 3 "l'image sort en 0"
  # Deux refus justes : `sqlx`, ou, sur une migration breaking, `check_downgrade_protection`,
  # qui passe avant lui. Ni l'un ni l'autre : l'image a échoué pour une autre raison.
  if /usr/bin/grep -q 'previously applied but is missing' "$L/image-refus.log"; then
    echo "  refusée : migration inconnue d'elle (sqlx)"
  elif /usr/bin/grep -q 'La base a été migrée par Kesh' "$L/image-refus.log"; then
    echo "  refusée : version minimale relevée (migration breaking)"
  else
    echec 3 "le journal ne dit pas le refus ($L/image-refus.log)"
  fi
else
  echo "▶ phase 3 : sautée, aucune migration, donc aucun refus à observer"
fi

# --- Phase 4 : retour --------------------------------------------------------------------
echo "▶ phase 4 : retour par le dump"
"$DOCKER" exec -i -e "MYSQL_PWD=$MDP_ROOT" "$CT_BASE" mariadb -uroot < "$D/avant.sql" || echec 4 "restauration en échec"
lancer_image retour
attendre_sante "$PID_IMAGE" || echec 4 "l'image ne repart pas ($L/image-retour.log)"
empreinte retour || echec 4 "empreinte retour"
[ -z "$(ecarts avant retour)" ] || echec 4 "empreinte retour différente : $(ecarts avant retour | tr '\n' ' ')"
# Les références de la phase 6 (D5) : les pièces jamais figées, rendues par l'image précédente sur la
# base d'avant restaurée — après son empreinte, avant la lecture de son journal et son arrêt. Son
# dossier des documents est vide : aucune pièce figée n'y est demandée.
for f in F2 F4 F6 A1; do pdf_rendu "$f" "$P/retour-$f.pdf"; done
journal_sans_erreur 4 "$L/image-retour.log"
arreter_image

# --- Phase 5 : import --------------------------------------------------------------------
if [ "$IMPORT" = 1 ]; then
  echo "▶ phase 5 : le .keshbackup dans le binaire neuf, base neuve"
  lancer_base "$CT_BASE_2" "$PORT_BASE_2" || echec 5 "seconde base injoignable"
  CT_COURANT=$CT_BASE_2
  # Un dossier des documents neuf (D6) : un PDF figé servi après l'import vient de l'archive.
  lancer_binaire import "$PORT_BASE_2" "$D/documents-import"
  attendre_sante "$PID_BINAIRE" || echec 5 "le binaire ne répond pas ($L/binaire-import.log)"
  login || echec 5 "connexion impossible"
  AUDIT_DECALAGE=$(SQL "SELECT COALESCE(MAX(id), 0) FROM audit_log") || echec 5 "journal d'audit illisible"
  CODE_HTTP=$(C --max-time 600 -o "$L/import-reponse.json" -w '%{http_code}' -X POST "$URL/api/v1/admin/full-import" \
    -H "Authorization: Bearer $TOKEN" -F "file=@$D/avant.keshbackup")
  [ "$CODE_HTTP" = 200 ] || echec 5 "import : HTTP $CODE_HTTP ($L/import-reponse.json)"
  empreinte import || echec 5 "empreinte import"
  ecarts_hors_attendu 5 avant import
  # F8 compris : l'archive, prise avant le retrait de son fichier, le porte.
  for f in F1 F5 F7 F8; do pdf_fige_identique 5 "$f" import; done
  journal_sans_erreur 5 "$L/binaire-import.log"
  arreter_binaire
fi

# --- Phase 6 : rendus ----------------------------------------------------------------------
# Story 49-1d (D4 b, D5) : chaque PDF rendu de nouveau par le binaire neuf contre sa référence —
# rendue par l'image précédente en phase 4, ou, pour le regel de F8, son PDF figé de la phase 1.
# Une paire compte si ses deux PDF sont valides, ont au moins une page et que leur texte contient
# le numéro de la pièce ; elle doit alors avoir le même nombre de pages, le même texte et, page à
# page, la même trame à 300 dpi (empreinte MD5 prise sur la sortie standard de `mutool` : aucune
# trame n'est écrite). « Même trame » : le même dessin au pixel près, avec ce `mutool` — non le
# même fichier (dictionnaire `Info`, `/ID`, structure). Exactement 5 paires, avec ou sans import.
echo "▶ phase 6 : rendus (trame à 300 dpi, texte, pages)"
"$MUTOOL" -v > "$L/mutool-version.txt" 2>&1 || echec 6 "mutool -v en échec"
pages_de() { "$MUTOOL" info "$1" 2>>"$L/mutool.log" | awk '/^Pages:/ {print $2}'; }
PAIRES=0
for paire in "F2 retour-F2 apres-F2" "F4 retour-F4 apres-F4" "F6 retour-F6 apres-F6" "A1 retour-A1 apres-A1" "F8 avant-F8 apres-F8-regel"; do
  read -r piece ref ren <<< "$paire"
  ref=$P/$ref.pdf ren=$P/$ren.pdf
  num=$(numero_piece "$piece")
  if ! pdf_valide "$ref" || ! pdf_valide "$ren"; then
    echo "  $piece : paire non comptée (un des deux PDF manque ou n'en est pas un)"; continue
  fi
  pr=$(pages_de "$ref") || echec 6 "$piece : mutool info en échec ($ref)"
  pn=$(pages_de "$ren") || echec 6 "$piece : mutool info en échec ($ren)"
  if ! [[ "$pr" =~ ^[1-9][0-9]*$ ]]; then echo "  $piece : paire non comptée (référence sans page)"; continue; fi
  [ "$pr" = "$pn" ] || echec 6 "$piece : $pr page(s) en référence, ${pn:-?} rendue(s)"
  "$MUTOOL" draw -q -F txt -o - "$ref" > "$L/rendu-$piece-reference.txt" 2>>"$L/mutool.log" || echec 6 "$piece : texte de la référence illisible"
  "$MUTOOL" draw -q -F txt -o - "$ren" > "$L/rendu-$piece-rendu.txt" 2>>"$L/mutool.log" || echec 6 "$piece : texte du rendu illisible"
  if [ -z "$num" ] || ! /usr/bin/grep -qF -- "$num" "$L/rendu-$piece-reference.txt" || ! /usr/bin/grep -qF -- "$num" "$L/rendu-$piece-rendu.txt"; then
    echo "  $piece : paire non comptée (le texte ne contient pas le numéro « $num »)"; continue
  fi
  diff -- "$L/rendu-$piece-reference.txt" "$L/rendu-$piece-rendu.txt" > "$L/diff-rendu-$piece.txt" \
    || echec 6 "$piece : texte rendu différent ($L/diff-rendu-$piece.txt)"
  for p in $(seq 1 "$pr"); do
    hr=$("$MUTOOL" draw -q -r 300 -c rgb -F pnm -o - "$ref" "$p" 2>>"$L/mutool.log" | md5sum) || echec 6 "$piece : trame de la référence, page $p"
    hn=$("$MUTOOL" draw -q -r 300 -c rgb -F pnm -o - "$ren" "$p" 2>>"$L/mutool.log" | md5sum) || echec 6 "$piece : trame du rendu, page $p"
    [ "$hr" = "$hn" ] || echec 6 "$piece : page $p, trame différente à 300 dpi (texte égal)"
  done
  echo "  $piece : $pr page(s), même dessin"
  PAIRES=$((PAIRES + 1))
done
[ "$PAIRES" -eq 5 ] || echec 6 "$PAIRES paire(s) de rendus comptée(s), 5 exigées"

echo "✅ recette-montee : phases jouées, $MIGRATIONS migration(s), $PAIRES paires de rendus, journaux sous $L"
