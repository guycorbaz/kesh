#!/usr/bin/env bash
# recette-montee.test.sh — éprouve `scripts/recette-montee.sh` avec `docker`, `curl` et
# un binaire FACTICES, sur un mini-dépôt sous `mktemp -d` (Story 33-10, AC-5, AC-7).
#
# L'environnement du script étant fermé, les factices ne reçoivent rien de lui : ils
# lisent leur réglage (`conf`) et écrivent leurs journaux dans leur propre dossier.
#
# Lancé par `crates/kesh-db/tests/recette_montee_script.rs` (gate et CI) ; lançable seul.

set -euo pipefail

SCRIPTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT_SRC="$SCRIPTS_DIR/recette-montee.sh"
FIXTURES_SRC="$SCRIPTS_DIR/fixtures/recette"
COMPOSE="$SCRIPTS_DIR/../docker-compose.dev.yml"

# Sous /tmp : la racine jetable doit y être.
WORK=$(mktemp -d /tmp/recette-montee-test.XXXXXX)
trap 'rm -rf "$WORK"' EXIT

ECHECS=0
fail() { echo "    ✗ $*"; ECHECS=$((ECHECS + 1)); }
ok()   { echo "    ✓ $*"; }

# --- Le mini-dépôt et ce qui l'entoure --------------------------------------------
DEPOT="$WORK/depot"
mkdir -p "$DEPOT/scripts/fixtures/recette" "$DEPOT/target/debug" "$DEPOT/frontend/build" \
  "$WORK/kesh-gate-logs" "$WORK/dehors" "$WORK/fx" "$WORK/jetable"
cp "$SCRIPT_SRC" "$DEPOT/scripts/recette-montee.sh"
cp "$FIXTURES_SRC"/*.xml "$DEPOT/scripts/fixtures/recette/"
echo ':' > "$WORK/dehors/binaire"
echo ':' > "$WORK/dehors/apres.sh"
echo 'true' > "$WORK/kesh-gate-logs/verifs.sh"
ln -s "$WORK/dehors/binaire" "$DEPOT/target/debug/lien"
FX="$WORK/fx"

# Le docker factice : journal, état de la base dans `$FX/migre`.
cat > "$FX/docker" <<'FACTICE'
#!/usr/bin/env bash
FX="$(dirname "$0")"
. "$FX/conf"
printf 'ARGV %s\n' "$*" >> "$FX/docker.log"
tout=" $* "
migre=$(cat "$FX/migre" 2>/dev/null || echo non)
case "${1:-}" in
  ps) cat "$FX/ps.txt" 2>/dev/null; exit 0 ;;
  inspect)
    if [ "${INSPECT_AUTRE:-non}" = oui ]; then echo un-autre-jeton; else cat "$FX/jeton"; fi
    exit 0 ;;
  stop)
    case "$2" in *-image-*) : > "$FX/up-image.stop"; rm -f "$FX/up-image" ;; esac
    exit 0 ;;
  run)
    case "$tout" in
      *" mariadb:10.11 "*)
        for a in "$@"; do case "$a" in kesh.recette=*) printf '%s\n' "${a#kesh.recette=}" > "$FX/jeton" ;; esac; done
        echo idfactice; exit 0 ;;
    esac
    if [ "$migre" = oui ] && [ "${IMAGE_DEMARRE_SUR_MIGRE:-non}" != oui ]; then
      echo "${MESSAGE_REFUS:-Error: migration 20261013000001 was previously applied but is missing in the resolved migrations}"
      exit "${CODE_REFUS:-1}"
    fi
    echo $$ > "$FX/up-image"
    while [ -f "$FX/up-image" ]; do sleep 0.05; done
    exit 0 ;;
  exec)
    case "$tout" in
      *mariadb-dump*) echo "-- dump factice"; exit 0 ;;
    esac
    sql=""; prec=""
    for a in "$@"; do [ "$prec" = "-e" ] && [ "${a#MYSQL_PWD}" = "$a" ] && sql=$a; prec=$a; done
    if [ -z "$sql" ]; then cat >/dev/null; echo non > "$FX/migre"; exit 0; fi
    # La seconde base, celle de la phase 5, n'a pas d'état distinct ici.
    case "$sql" in
      "SELECT 1") echo 1 ;;
      "SELECT COUNT(*) FROM _sqlx_migrations")
        if [ "$migre" = oui ]; then echo $((88 + ${N_REEL:-1})); else echo 88; fi ;;
      "SELECT COUNT(*) FROM bank_transactions") echo 2 ;;
      *"FROM _sqlx_migrations"*) echo "migrations-$migre" ;;
      "SELECT * FROM "*)
        t=${sql#SELECT \* FROM }; t=${t%% *}
        if [ "$migre" = oui ] && [ "${ECART:-}" = "$t" ]; then echo "$t modifiée"; else echo "$t"; fi ;;
      *) echo factice ;;
    esac
    exit 0 ;;
esac
echo "docker factice : sous-commande imprévue : $*" >&2
exit 97
FACTICE

# Le curl factice : répond selon le chemin ; `/health` seulement si un serveur vit.
cat > "$FX/curl" <<'FACTICE'
#!/usr/bin/env bash
FX="$(dirname "$0")"
. "$FX/conf"
printf 'ARGV %s\n' "$*" >> "$FX/curl.log"
url="" sortie="" ecrit="" vers_fichier=0 fail=0 stdin=0
prec=""
for a in "$@"; do
  case "$prec" in
    -o) sortie=$a ;;
    -w) ecrit=$a ;;
  esac
  case "$a" in
    http://*) url=$a ;;
    -f) fail=1 ;;
    @-) stdin=1 ;;
  esac
  prec=$a
done
[ "$stdin" = 0 ] || cat >/dev/null
chemin=${url#http://*/}
chemin=${chemin%%\?*}
reponse='{"id":1,"version":1,"status":"x","remainingAmount":"1.00"}'
case "/$chemin" in
  /health)
    if { [ -f "$FX/up-image" ] || [ -f "$FX/up-binaire" ]; }; then reponse=ok
    else exit 7; fi ;;
  /api/v1/auth/login) reponse='{"accessToken":"jeton-factice"}' ;;
  /api/v1/accounts)
    reponse='[{"id":1,"number":"1000","accountType":"Asset","active":true,"postable":true},{"id":2,"number":"1020","accountType":"Asset","active":true,"postable":true},{"id":3,"number":"2000","accountType":"Liability","active":true,"postable":true},{"id":4,"number":"6000","accountType":"Expense","active":true,"postable":true}]' ;;
  /api/v1/bank-accounts) reponse='[]' ;;
  /api/v1/bank-imports/preview) reponse='{"previewFingerprint":"f"}' ;;
  /api/v1/company/invoice-settings) reponse='{}' ;;
  /api/v1/admin/full-export) reponse='KESHBACKUP-FACTICE' ;;
esac
case "$*" in
  *"-X POST"*"/api/v1/bank-accounts"*) reponse='{"id":5}' ;;
esac
if [ -n "$sortie" ]; then printf '%s' "$reponse" > "$sortie"; else printf '%s' "$reponse"; fi
[ -z "$ecrit" ] || printf '%s' "${CODE_IMPORT:-200}"
exit 0
FACTICE

# Le binaire factice : consigne ce qu'il reçoit, « migre », puis sert jusqu'au signal.
cat > "$DEPOT/target/debug/kesh-api" <<FACTICE
#!/usr/bin/env bash
FX="$FX"
. "\$FX/conf"
env > "\$FX/binaire-env.txt"
pwd > "\$FX/binaire-pwd.txt"
echo "\$KESH_ADMIN_BACKUP_DIR" > "\$FX/binaire-backup.txt"
echo oui > "\$FX/migre"
if [ "\${PRE_UPGRADE:-oui}" = oui ] && [ "\${N_REEL:-1}" -gt 0 ]; then
  mkdir -p "\$KESH_ADMIN_BACKUP_DIR/pre-upgrade"
  echo factice > "\$KESH_ADMIN_BACKUP_DIR/pre-upgrade/x.keshbackup"
fi
echo \$\$ > "\$FX/up-binaire"
trap 'rm -f "\$FX/up-binaire"; exit 0' TERM
while :; do sleep 0.05; done
FACTICE
chmod +x "$FX/docker" "$FX/curl" "$DEPOT/target/debug/kesh-api"

# Deux ports libres, trouvés au moment du test, sous la plage éphémère du système : les
# connexions à la base des autres tests y prendraient sinon le port entre deux contrôles.
port_libre() {
  local p=$1
  while ss -ltn | awk '{print $4}' | sed 's/.*://' | /usr/bin/grep -qx "$p"; do p=$((p + 2)); done
  echo "$p"
}
PA=$(port_libre 23310)
PB=$(port_libre 23410)

# Remet les factices à neuf. Arguments : lignes de `conf`.
preparer() {
  : > "$FX/docker.log"; : > "$FX/curl.log"
  rm -f "$FX/migre" "$FX/up-image" "$FX/up-binaire" "$FX/ps.txt" "$FX/binaire-env.txt" "$FX/binaire-pwd.txt" "$FX/jeton"
  printf '%s\n' "$@" > "$FX/conf"
}

BASE_ARGS=(--image gcorbaz/kesh:0.17.4 --binaire target/debug/kesh-api --static frontend/build
  --port-api "$PA" --port-base "$PB" --migrations-attendues 1 --diff-attendu migrations)

# Lance le script du mini-dépôt, racine jetable sous $WORK. Sortie dans `$FX/sortie`.
RC=0
lancer() {
  RC=0
  KESH_DOCKER="$FX/docker" KESH_CURL="$FX/curl" RECETTE_RACINE_JETABLE="$WORK/jetable" \
    bash "$DEPOT/scripts/recette-montee.sh" "$@" > "$FX/sortie" 2>&1 || RC=$?
}
sortie_dit() { /usr/bin/grep -qF -- "$1" "$FX/sortie"; }

# Refus attendu, ZÉRO appel docker et curl.
refuse() {
  local titre=$1; shift
  preparer "N=1"
  lancer "$@"
  if [ "$RC" -eq 0 ]; then fail "$titre : accepté"
  elif [ -s "$FX/docker.log" ] || [ -s "$FX/curl.log" ]; then fail "$titre : des appels ($(wc -l < "$FX/docker.log") docker)"
  else ok "$titre (code $RC)"; fi
}

# Remplace la valeur d'une option de BASE_ARGS.
avec() {
  local opt=$1 val=$2 i
  AV=("${BASE_ARGS[@]}")
  for i in "${!AV[@]}"; do
    if [ "${AV[$i]}" = "$opt" ]; then AV[$((i + 1))]=$val; fi
  done
}

# --- AC-5 : les refus --------------------------------------------------------------
echo "  options inconnues"
for opt in --base --host --database-url --journal --env-file; do
  refuse "$opt" "${BASE_ARGS[@]}" "$opt" valeur
done

echo "  images"
for img in gcorbaz/kesh:latest gcorbaz/kesh autre/kesh:0.17.4 gcorbaz/kesh:0.17 mariadb:10.11 "gcorbaz/kesh:0.17.4;x"; do
  avec --image "$img"; refuse "image $img" "${AV[@]}"
done

echo "  entrées hors de leur place"
avec --binaire "$WORK/dehors/binaire"; refuse "--binaire, chemin direct" "${AV[@]}"
avec --binaire "target/../../dehors/binaire"; refuse "--binaire, par .." "${AV[@]}"
avec --binaire "target/debug/lien"; refuse "--binaire, lien symbolique" "${AV[@]}"
avec --static "$WORK/dehors"; refuse "--static hors du dépôt" "${AV[@]}"
refuse "--apres hors de sa place" "${BASE_ARGS[@]}" --apres "$WORK/dehors/apres.sh"
refuse "--apres dans le dépôt" "${BASE_ARGS[@]}" --apres "scripts/recette-montee.sh"

echo "  ports"
avec --port-api 80; refuse "port-api sous 1024" "${AV[@]}"
avec --port-api 70000; refuse "port-api trop grand" "${AV[@]}"
avec --port-base abc; refuse "port-base non entier" "${AV[@]}"
avec --port-base 65535; refuse "--import et --port-base 65535" "${AV[@]}" --import
# Un port réellement pris : le système le choisit, si bien qu'aucune connexion en cours
# ne l'occupe déjà. Rend le port dans PRIS, le processus dans ECOUTE.
ecouter() {
  local f="$FX/port-ecoute"
  : > "$f"
  python3 -c 'import socket,sys,time
s=socket.socket(); s.bind(("127.0.0.1",0)); s.listen(1)
open(sys.argv[1],"w").write(str(s.getsockname()[1])); time.sleep(60)' "$f" &
  ECOUTE=$!
  for _ in $(seq 1 100); do [ -s "$f" ] && break; sleep 0.05; done
  PRIS=$(cat "$f")
  [ -n "$PRIS" ] || { echo "montage : aucun port d'écoute" >&2; exit 2; }
  ss -ltn | awk '{print $4}' | sed 's/.*://' | /usr/bin/grep -qx "$PRIS" \
    || { echo "montage : le port $PRIS n'écoute pas" >&2; exit 2; }
}
ecouter
avec --port-api "$PRIS"; refuse "port-api pris" "${AV[@]}"
avec --port-base "$PRIS"; refuse "port-base pris" "${AV[@]}"
avec --port-base "$((PRIS - 1))"; refuse "--port-base + 1 pris, avec --import" "${AV[@]}" --import
kill "$ECOUTE" 2>/dev/null; wait "$ECOUTE" 2>/dev/null || true

echo "  dossier jetable"
RC2=0
preparer "N=1"
KESH_DOCKER="$FX/docker" KESH_CURL="$FX/curl" RECETTE_RACINE_JETABLE=/var \
  bash "$DEPOT/scripts/recette-montee.sh" "${BASE_ARGS[@]}" > "$FX/sortie" 2>&1 || RC2=$?
if [ "$RC2" -ne 0 ] && [ ! -s "$FX/docker.log" ]; then ok "hors de /tmp refusée (code $RC2)"; else fail "hors de /tmp"; fi

mkdir -p "$WORK/env-parent/enfant"; touch "$WORK/env-parent/.env"
preparer "N=1"
RC2=0
KESH_DOCKER="$FX/docker" KESH_CURL="$FX/curl" RECETTE_RACINE_JETABLE="$WORK/env-parent/enfant" \
  bash "$DEPOT/scripts/recette-montee.sh" "${BASE_ARGS[@]}" > "$FX/sortie" 2>&1 || RC2=$?
if [ "$RC2" -eq 3 ] && [ ! -s "$FX/docker.log" ]; then ok ".env dans un parent de la racine jetable"; else fail ".env parent : code $RC2"; fi

echo "  conteneur d'un run précédent : aucun autre appel que la liste"
preparer "N=1"
echo kesh-recette-db-123 > "$FX/ps.txt"
lancer "${BASE_ARGS[@]}"
if [ "$RC" -eq 4 ] && sortie_dit 'docker stop <nom>' && sortie_dit kesh-recette-db-123 \
  && [ "$(wc -l < "$FX/docker.log")" -eq 1 ] && /usr/bin/grep -q 'ARGV ps ' "$FX/docker.log" && [ ! -s "$FX/curl.log" ]; then
  ok "refusé, nom et geste dits"
else fail "code $RC : $(cat "$FX/sortie" | tail -2)"; fi

# --- AC-5 : le chemin nominal, l'environnement et les conteneurs ----------------------
APPELANT="$WORK/parent-appelant/appelant"
mkdir -p "$APPELANT"
echo 'KESH_ADMIN_BACKUP_DIR=/inexistant/sonde' | tee "$APPELANT/.env" "$WORK/parent-appelant/.env" "$DEPOT/.env" >/dev/null
preparer "N_REEL=1"
RC=0
(cd "$APPELANT" && DATABASE_URL=mysql://sonde:sonde@sonde.invalid/sonde KESH_ADMIN_BACKUP_DIR=/inexistant/sonde \
  KESH_SMTP_PASSWORD=sonde-smtp KESH_RECETTE_RELANCE=1 KESH_DOCKER="$FX/docker" KESH_CURL="$FX/curl" RECETTE_RACINE_JETABLE="$WORK/jetable" \
  bash "$DEPOT/scripts/recette-montee.sh" "${BASE_ARGS[@]}" --import --diff-attendu migrations) > "$FX/sortie" 2>&1 || RC=$?
echo "  chemin nominal, cinq phases"
[ "$RC" -eq 0 ] && ok "code 0" || { fail "code $RC"; tail -5 "$FX/sortie"; }
for p in "phase 1" "phase 2" "phase 3" "phase 4" "phase 5"; do sortie_dit "▶ $p" && ok "$p jouée" || fail "$p"; done
ordre=$(/usr/bin/grep -oE '▶ phase [1-5]' "$FX/sortie" | tr -d '▶ phase' | tr -d '\n')
[ "$ordre" = 12345 ] && ok "dans l'ordre" || fail "ordre : $ordre"

echo "  environnement fermé du binaire"
D=$(/usr/bin/grep -oE 'dossier .*$' "$FX/sortie" | tail -1 | sed 's/^dossier //')
case "$D" in "$WORK/jetable/"*) ok "dossier sous la racine jetable" ;; *) fail "dossier : $D" ;; esac
noms=$(cut -d= -f1 "$FX/binaire-env.txt" | /usr/bin/grep -vxE 'PWD|SHLVL|_|OLDPWD' | sort | tr '\n' ' ')
attendu="DATABASE_URL HOME KESH_ADMIN_BACKUP_DIR KESH_ADMIN_PASSWORD KESH_ADMIN_USERNAME KESH_COOKIE_SECURE KESH_DOCUMENTS_DIR KESH_HOST KESH_INBOX_DIR KESH_JWT_SECRET KESH_PORT KESH_SMTP_FROM KESH_SMTP_HOST KESH_SMTP_PASSWORD KESH_SMTP_USER KESH_STATIC_DIR PATH "
[ "$noms" = "$attendu" ] && ok "liste exacte" || fail "liste : $noms"
/usr/bin/grep -qE 'sonde|KESH_RECETTE_RELANCE' "$FX/binaire-env.txt" && fail "une sonde a traversé" || ok "aucune sonde"
[ "$(cat "$FX/binaire-pwd.txt")" = "$D/cwd" ] && ok "pwd = \$D/cwd" || fail "pwd : $(cat "$FX/binaire-pwd.txt")"
for v in KESH_ADMIN_BACKUP_DIR KESH_DOCUMENTS_DIR KESH_INBOX_DIR HOME; do
  val=$(/usr/bin/grep "^$v=" "$FX/binaire-env.txt" | cut -d= -f2-)
  case "$val" in "$D"/*) ok "$v sous \$D" ;; *) fail "$v : $val" ;; esac
done
/usr/bin/grep -qE "^DATABASE_URL=mysql://kesh:[^@]*@127\.0\.0\.1:$((PB + 1))/kesh_recette$" "$FX/binaire-env.txt" \
  && ok "DATABASE_URL de sa base jetable" || fail "$(/usr/bin/grep DATABASE_URL "$FX/binaire-env.txt")"
[ -f "$D/backup/pre-upgrade/x.keshbackup" ] && ok "pre-upgrade sous \$D/backup" || fail "pre-upgrade"

echo "  conteneurs"
run_base=$(/usr/bin/grep -m1 'ARGV run .*mariadb:10.11' "$FX/docker.log" || true)
for m in "--rm" "--label kesh.recette=" "--tmpfs /var/lib/mysql" "-p 127.0.0.1:$PB:3306" "--memory=2g"; do
  case "$run_base" in *"$m"*) ok "base : $m" ;; *) fail "base sans $m" ;; esac
done
run_image=$(/usr/bin/grep -m1 'ARGV run .*gcorbaz/kesh:' "$FX/docker.log" || true)
case "$run_image" in *"--network host"*) ok "image : --network host" ;; *) fail "image : réseau" ;; esac
for m in "PATH=" "HOME=" "KESH_STATIC_DIR="; do
  case "$run_image" in *"-e $m"*) fail "image reçoit $m" ;; *) ok "image sans $m" ;; esac
done
/usr/bin/grep -E 'ARGV run ' "$FX/docker.log" > "$FX/runs"
if /usr/bin/grep -qE -- '--env-file|(^| )-v( |$)|--volume|--mount' "$FX/runs"; then fail "montage ou env-file"; else ok "aucun montage, aucun --env-file"; fi
if /usr/bin/grep -E 'ARGV run ' "$FX/docker.log" | tr ' ' '\n' | awk 'p {if ($0 !~ /=/) bad=1; p=0} $0=="-e" {p=1} END {exit bad}'; then ok "tout -e porte son ="; else fail "-e sans ="; fi
if /usr/bin/grep -rq kesh-mariadb-dev "$FX/docker.log" "$SCRIPT_SRC"; then fail "kesh-mariadb-dev apparaît"; else ok "kesh-mariadb-dev n'apparaît jamais"; fi
/usr/bin/grep -q 'ARGV stop kesh-recette-db-' "$FX/docker.log" && ok "base arrêtée par son nom" || fail "base non arrêtée"
/usr/bin/grep -qE 'image: mariadb:10\.11$' "$COMPOSE" && /usr/bin/grep -q 'mariadb:10\.11 ' "$SCRIPT_SRC" \
  && ok "mariadb:10.11 égale docker-compose.dev.yml" || fail "image de base différente du compose"

echo "  préfixe des conteneurs"
preparer "N_REEL=1"
lancer "${BASE_ARGS[@]}" --prefixe kesh-e3310
if [ "$RC" -eq 0 ] && /usr/bin/grep -q -- '--name kesh-e3310-db-' "$FX/docker.log" && /usr/bin/grep -q -- '--name kesh-e3310-image-' "$FX/docker.log"; then ok "kesh-e3310-*"; else fail "préfixe : code $RC"; fi
refuse "préfixe hors du motif" "${BASE_ARGS[@]}" --prefixe "mariadb"
refuse "préfixe à injection" "${BASE_ARGS[@]}" --prefixe "kesh-a;b"

echo "  jeton du conteneur"
preparer "N_REEL=1" "INSPECT_AUTRE=oui"
lancer "${BASE_ARGS[@]}"
[ "$RC" -eq 5 ] && ok "un autre jeton fait refuser" || fail "code $RC"
if /usr/bin/grep -q 'ARGV run .*gcorbaz/kesh:' "$FX/docker.log"; then fail "image lancée malgré tout"; else ok "image non lancée"; fi

# --- AC-5 : l'enchaînement --------------------------------------------------------
echo "  écart d'empreinte"
preparer "N_REEL=1" "ECART=invoices"
lancer "${BASE_ARGS[@]}"
[ "$RC" -eq 12 ] && sortie_dit invoices && ok "hors de --diff-attendu : échec" || fail "code $RC"
preparer "N_REEL=1" "ECART=invoices"
avec --diff-attendu migrations,invoices; lancer "${AV[@]}"
[ "$RC" -eq 0 ] && ok "dans --diff-attendu : passe" || fail "code $RC"
preparer "N_REEL=1" "ECART=invoices"
avec --diff-attendu migrations,invoice; lancer "${AV[@]}"
[ "$RC" -eq 12 ] && ok "un nom proche ne couvre pas" || fail "code $RC"

echo "  migrations"
preparer "N_REEL=2"
lancer "${BASE_ARGS[@]}"
[ "$RC" -eq 12 ] && ok "un écart au nombre attendu : échec" || fail "code $RC"
preparer "N_REEL=1" "PRE_UPGRADE=non"
lancer "${BASE_ARGS[@]}"
[ "$RC" -eq 12 ] && sortie_dit pre-upgrade && ok "aucun fichier pre-upgrade : échec" || fail "code $RC"

echo "  refus de l'image"
preparer "N_REEL=1" "IMAGE_DEMARRE_SUR_MIGRE=oui"
lancer "${BASE_ARGS[@]}"
[ "$RC" -eq 13 ] && sortie_dit "démarre" && ok "une image qui démarre : échec" || fail "code $RC"
preparer "N_REEL=1" "MESSAGE_REFUS=autre-erreur"
lancer "${BASE_ARGS[@]}"
[ "$RC" -eq 13 ] && ok "journal sans « previously applied » : échec" || fail "code $RC"
preparer "N_REEL=1" "CODE_REFUS=0"
lancer "${BASE_ARGS[@]}"
[ "$RC" -eq 13 ] && ok "image qui sort en 0 : échec" || fail "code $RC"

echo "  N = 0"
preparer "N_REEL=0"
avec --migrations-attendues 0; lancer "${AV[@]}"
[ "$RC" -eq 0 ] && sortie_dit "sautée" && ok "phase 3 sautée, et dit" || { fail "code $RC"; tail -3 "$FX/sortie"; }
/usr/bin/grep -q 'kesh_recette' "$FX/binaire-env.txt" && ok "binaire lancé" || fail "binaire"

echo "  crochet --apres"
echo 'SQL "SELECT 1" > /dev/null && api GET /api/v1/accounts > /dev/null && echo "L=$L" && exit 1' > "$WORK/kesh-gate-logs/verifs.sh"
preparer "N_REEL=1"
lancer "${BASE_ARGS[@]}" --apres "$WORK/kesh-gate-logs/verifs.sh"
[ "$RC" -eq 12 ] && ok "un crochet en échec fait échouer la phase" || fail "code $RC"
echo 'SQL "SELECT 1" > /dev/null && api GET /api/v1/accounts > /dev/null && [ -d "$L" ]' > "$WORK/kesh-gate-logs/verifs.sh"
preparer "N_REEL=1"
lancer "${BASE_ARGS[@]}" --apres "$WORK/kesh-gate-logs/verifs.sh"
[ "$RC" -eq 0 ] && ok "le crochet dispose de SQL, api, L" || fail "code $RC"

# --- AC-7 -------------------------------------------------------------------------
echo "  AC-7 : aucun chemin de la station, aucune donnée réelle"
MOTIF='/home/|\$\{?HOME\}?/|~/|/volume1|gvfs|kesh-test|192\.168\.|\.local\b|\bnas\b'
SONDE=$(mktemp "$WORK/sonde.XXXXXX"); echo 'x=/volume1/y' > "$SONDE"
/usr/bin/grep -qiE "$MOTIF" "$SONDE" && ok "le motif reconnaît sa sonde" || fail "motif muet"
if /usr/bin/grep -rniE "$MOTIF" "$SCRIPT_SRC" "$FIXTURES_SRC"; then fail "chemin de la station"; else ok "aucun"; fi
if /usr/bin/grep -rn gcorbaz "$SCRIPT_SRC" "$FIXTURES_SRC" | /usr/bin/grep -v 'gcorbaz/kesh:'; then fail "gcorbaz hors de l'image"; else ok "gcorbaz seulement dans gcorbaz/kesh:"; fi
/usr/bin/grep -q 'CH4431999123000889012' "$FIXTURES_SRC/releve-A.xml" && ok "IBAN d'exemple" || fail "IBAN du relevé"

echo
if [ "$ECHECS" -gt 0 ]; then
  echo "❌ $ECHECS assertion(s) en échec"
  exit 1
fi
echo "✅ aucune assertion en échec"
