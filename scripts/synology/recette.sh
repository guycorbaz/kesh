#!/bin/bash
# recette.sh — recette des scripts Synology de sauvegarde et de restauration
# (Story 15-14b, #575). Rejoue kesh-dump.sh et kesh-restore.sh TELS QUE LIVRÉS, avec
# les comptes et le fichier d'options écrits comme le manuel d'administration les
# donne, sur des conteneurs et un réseau DE TEST nommés « kesh-recette-synology* »,
# détruits à la fin (et à l'interruption). Ne touche à aucun autre conteneur —
# jamais kesh-mariadb-dev.
#
# Usage, depuis la racine du dépôt : bash scripts/synology/recette.sh
# Prérequis : docker, sqlx (migrations du dépôt), python3. Hors gate (≈ 1 min) ;
# documentée dans docs/testing.md. Sortie : un compte rendu par étape, puis
# « RECETTE VERTE » (code 0) ou « RECETTE ROUGE » (code 1).
#
# Ce qui n'est PAS rejoué : DSM (Planificateur de tâches, Hyper Backup, Snapshot
# Replication, paquet MariaDB 10) ; root (les scripts tournent ici sous
# l'utilisateur courant ; chown/chmod des .cnf passent par un conteneur root).
set -uo pipefail
RACINE=$(cd "$(dirname "$0")/../.." && pwd -P)
SCRIPTS=$RACINE/scripts/synology
MANUEL=$RACINE/docs/manual/fr/admin-manual.tex
NOM=kesh-recette-synology
NET=$NOM-reseau; DB=$NOM-db; export COMPOSE_PROJECT_NAME=$NOM
IMG=mariadb:10.11
W=$(mktemp -d "${TMPDIR:-/tmp}/$NOM.XXXXXX")
ROOTPW=$(openssl rand -hex 16)
KESHPW='Pa@ss#;w"0rd\x/ %q'          # @ # ; " \ / espace : guillemets, échappements
BKPPW=$(openssl rand -hex 32)
ECHECS=0

for n in "$NET" "$DB"; do case "$n" in *kesh-mariadb-dev*) echo "refus : $n"; exit 1;; esac; done
nettoyer() {
    ( cd "$W/kesh" 2>/dev/null && docker compose down --remove-orphans >/dev/null 2>&1 )
    docker rm -f "$DB" >/dev/null 2>&1; docker network rm "$NET" >/dev/null 2>&1
    docker run --rm -v "$W:/w" "$IMG" rm -rf /w/kesh /w/restaure >/dev/null 2>&1; rm -rf "$W"
}
trap nettoyer EXIT
ok()  { echo "  OK    $*"; }
ko()  { echo "  ÉCHEC $*"; ECHECS=$((ECHECS+1)); }
verifier() { if [ "$2" = "$3" ]; then ok "$1 ($2)"; else ko "$1 : $2, attendu $3"; fi; }
sql()  { docker exec -i "$DB" mariadb -uroot -p"$ROOTPW" -N "$@"; }
compter() { sql -e "SELECT COUNT(*) FROM kesh.accounts" 2>/dev/null || echo absente; }
kesh_api() { ( cd "$W/kesh" && docker compose ps --status running --services 2>/dev/null | grep -cx kesh-api ); }
lancer() { SAUVEGARDE_DOSSIER="$W/kesh" SAUVEGARDE_RESEAU="$NET" bash "$@"; }
vides() { find "$W/kesh" -type f -empty 2>/dev/null | wc -l; }
listing() {  # contenu du lstlisting du manuel qui contient $1
    python3 - "$MANUEL" "$1" <<'PY'
import re, sys
t = open(sys.argv[1], encoding='utf-8').read()
b = [x for x in re.findall(r'\\begin\{lstlisting\}(?:\[[^\]]*\])?\n(.*?)\\end\{lstlisting\}', t, re.S) if sys.argv[2] in x]
assert len(b) == 1, (sys.argv[2], len(b)); sys.stdout.write(b[0])
PY
}
cnf() {  # fichier d'options écrit comme le manuel le dit : mot de passe décodé, entre guillemets, \" et \\ échappés
    local e; e=$(printf '%s' "$2" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')
    listing '[client]' | python3 -c 'import sys
t=sys.stdin.read(); h,u,p=sys.argv[1:4]
t=t.replace("\"192.168.1.10\"","\"%s\""%h).replace("port = 3307","port = 3306").replace("\"kesh_backup\"","\"%s\""%u)
sys.stdout.write(t.replace("\"<mot de passe en clair, décodé>\"","\"%s\""%p))' "$DB" "$1" "$e"
}

echo "== Mise en place (réseau, base, comptes du manuel, schéma Kesh, seed)"
docker network create "$NET" >/dev/null
docker run -d --name "$DB" --network "$NET" -p 127.0.0.1::3306 -e MARIADB_ROOT_PASSWORD="$ROOTPW" "$IMG" >/dev/null
for i in $(seq 120); do
    [ "$(docker logs "$DB" 2>&1 | grep -c 'ready for connections')" -ge 2 ] && sql -e 'SELECT 1' >/dev/null 2>&1 && break; sleep 1
done
PORT=$(docker port "$DB" 3306/tcp | head -1 | sed 's/.*://')
# Compte Kesh : les lignes de l'« Initialisation manuelle de la base », placeholders remplacés.
listing "CREATE USER 'kesh'@'%'" | sed -n '/^CREATE DATABASE/,/^FLUSH/p' \
  | python3 -c 'import sys; pw=sys.argv[1].replace("\\","\\\\").replace("'"'"'","\\'"'"'"); sys.stdout.write(sys.stdin.read().replace("<sortie de openssl rand -hex 32>", pw))' "$KESHPW" | sql
listing "TO 'kesh_backup'" | sed "s#<sortie de openssl rand -hex 32>#$BKPPW#" | sql
sql -e "SHOW GRANTS FOR 'kesh_backup'@'%'" | sed 's/^/  /'
ENC=$(python3 -c 'import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=""))' "$KESHPW")
( cd "$RACINE" && DATABASE_URL="mysql://kesh:$ENC@127.0.0.1:$PORT/kesh" sqlx migrate run --source crates/kesh-db/migrations >/dev/null ) && ok "schéma Kesh (compte Kesh, mot de passe à \" et \\)" || ko "migrations"
sql kesh < "$RACINE/scripts/seed-dev-db.sql"
mkdir -p "$W/kesh" "$W/restaure"
cnf kesh_backup "$BKPPW" > "$W/kesh/kesh-dump.cnf"; cnf kesh "$KESHPW" > "$W/kesh/kesh-restore.cnf"
docker run --rm -v "$W/kesh:/k" "$IMG" sh -c 'chown root:root /k/kesh-dump.cnf /k/kesh-restore.cnf && chmod 600 /k/kesh-dump.cnf /k/kesh-restore.cnf'
printf 'services:\n  kesh-api:\n    image: %s\n    command: ["sleep", "infinity"]\n' "$IMG" > "$W/kesh/docker-compose.yml"
( cd "$W/kesh" && docker compose up -d >/dev/null 2>&1 ); verifier "kesh-api factice en marche" "$(kesh_api)" 1
A=$(compter); verifier "lignes de départ (accounts)" "$A" 5

echo "== 1. Dump"
lancer "$SCRIPTS/kesh-dump.sh" && ok "kesh-dump.sh sort en 0" || ko "kesh-dump.sh"
( cd "$W/kesh/dump" && sha256sum -c --quiet kesh_pre_backup.sql.gz.sha256 ) && ok "empreinte vérifiée" || ko "empreinte"
verifier "droits de dump/" "$(stat -c %a "$W/kesh/dump")" 700
verifier "droits du dump" "$(stat -c %a "$W/kesh/dump/kesh_pre_backup.sql.gz")" 600
verifier "droits de l'empreinte" "$(stat -c %a "$W/kesh/dump/kesh_pre_backup.sql.gz.sha256")" 600
verifier ".tmp restants" "$(ls "$W"/kesh/dump/*.tmp 2>/dev/null | wc -l)" 0
cp -a "$W/kesh/dump" "$W/restaure/dump"      # « Restaurer dans un nouveau dossier » : l'état A (5)
H=$(sha256sum "$W/kesh/dump/kesh_pre_backup.sql.gz" | cut -d' ' -f1)

echo "== 1-bis. --defaults-extra-file placé après une autre option : refusé"
docker run --rm --network "$NET" -v "$W/kesh/kesh-dump.cnf:/etc/x.cnf:ro" "$IMG" mariadb-dump --single-transaction --defaults-extra-file=/etc/x.cnf --databases kesh >/dev/null 2>&1
verifier "code de sortie" "$?" 7

echo "== 2. Dump raté (réseau absent, puis mot de passe faux) : la copie de la veille reste, aucun fichier vide"
SAUVEGARDE_DOSSIER="$W/kesh" SAUVEGARDE_RESEAU=$NOM-absent bash "$SCRIPTS/kesh-dump.sh" >/dev/null 2>&1 && ko "dump raté sorti en 0" || ok "dump raté (réseau) : sortie non nulle"
docker run --rm -v "$W/kesh:/k" "$IMG" sh -c 'cp -p /k/kesh-dump.cnf /k/kesh-dump.cnf.vrai && sed -i "s/^password = .*/password = \"faux\"/" /k/kesh-dump.cnf'
lancer "$SCRIPTS/kesh-dump.sh" >/dev/null 2>&1 && ko "dump raté sorti en 0" || ok "dump raté (mot de passe) : sortie non nulle"
docker run --rm -v "$W/kesh:/k" "$IMG" sh -c 'mv /k/kesh-dump.cnf.vrai /k/kesh-dump.cnf'
verifier "dump de la veille intact" "$(sha256sum "$W/kesh/dump/kesh_pre_backup.sql.gz" | cut -d' ' -f1)" "$H"
verifier ".tmp restants" "$(ls "$W"/kesh/dump/*.tmp 2>/dev/null | wc -l)" 0
verifier "fichiers vides" "$(vides)" 0

echo "== 3. Dump vivant différent du dump restauré (B), puis base modifiée (C)"
sql -e "SET FOREIGN_KEY_CHECKS=0; DELETE FROM kesh.accounts WHERE id % 2 = 0"
lancer "$SCRIPTS/kesh-dump.sh" >/dev/null && ok "dump vivant B ($(compter) lignes)"
sql -e "SET FOREIGN_KEY_CHECKS=0; DELETE FROM kesh.accounts ORDER BY id LIMIT 1"
C=$(compter); echo "  base courante C : $C lignes"

echo "== 4. Empreinte fausse, puis archive tronquée : arrêt avant tout arrêt de Kesh et toute écriture"
mkdir -p "$W/faux"; cp "$W/restaure/dump/kesh_pre_backup.sql.gz" "$W/faux/"
echo "$(printf '0%.0s' $(seq 64))  kesh_pre_backup.sql.gz" > "$W/faux/kesh_pre_backup.sql.gz.sha256"
lancer "$SCRIPTS/kesh-restore.sh" "$W/faux" >/dev/null 2>&1 && ko "empreinte fausse sortie en 0" || ok "empreinte fausse : sortie non nulle"
head -c 4000 "$W/restaure/dump/kesh_pre_backup.sql.gz" > "$W/faux/kesh_pre_backup.sql.gz"; ( cd "$W/faux" && sha256sum kesh_pre_backup.sql.gz > kesh_pre_backup.sql.gz.sha256 )
lancer "$SCRIPTS/kesh-restore.sh" "$W/faux" >/dev/null 2>&1 && ko "archive tronquée sortie en 0" || ok "archive tronquée : sortie non nulle"
verifier "base inchangée (aucun DROP)" "$(compter)" "$C"
verifier "kesh-api jamais arrêté" "$(kesh_api)" 1
verifier "aucun dump de sécurité pris" "$(ls "$W/kesh/avant-restauration" 2>/dev/null | wc -l)" 0

echo "== 5. Contrôle négatif : rechargement par le compte de sauvegarde (kesh-dump.cnf à la place de kesh-restore.cnf)"
docker run --rm -v "$W/kesh:/k" "$IMG" sh -c 'cp -p /k/kesh-restore.cnf /k/kesh-restore.cnf.vrai && cp -p /k/kesh-dump.cnf /k/kesh-restore.cnf'
SORTIE=$(lancer "$SCRIPTS/kesh-restore.sh" "$W/restaure/dump" 2>&1); RC=$?
docker run --rm -v "$W/kesh:/k" "$IMG" sh -c 'mv /k/kesh-restore.cnf.vrai /k/kesh-restore.cnf'
[ $RC -ne 0 ] && ok "sortie non nulle ($RC)" || ko "contrôle négatif sorti en 0"
echo "$SORTIE" | grep -q "ERROR 1044 .* to database 'kesh'" && ok "refus de privilège : $(echo "$SORTIE" | grep -m1 'ERROR 1044')" || ko "ERROR 1044 attendu"
verifier "base intacte" "$(compter)" "$C"
verifier "kesh-api laissé arrêté (étape 4 ratée)" "$(kesh_api)" 0
( cd "$W/kesh" && docker compose up -d >/dev/null 2>&1 )
docker run --rm -v "$W/kesh:/k" "$IMG" rm -rf /k/avant-restauration

echo "== 6. Restauration par un CHEMIN RELATIF, depuis le dossier restauré (le dump vivant B diffère)"
SORTIE=$(cd "$W/restaure" && lancer "$SCRIPTS/kesh-restore.sh" dump 2>&1); RC=$?; echo "$SORTIE" | sed 's/^/  | /'
verifier "code de sortie" "$RC" 0
verifier "base = dump RESTAURÉ (A), non le dump vivant (B)" "$(compter)" "$A"
verifier "kesh-api redémarré" "$(kesh_api)" 1
SECU=$(echo "$SORTIE" | sed -n 's/.*dump de sécurité de la base courante : //p')
[ -n "$SECU" ] && ok "dump de sécurité : ${SECU#$W/}" || ko "dump de sécurité non annoncé"
verifier "droits du dossier de sécurité" "$(stat -c %a "$SECU")" 700
verifier "droits du dump de sécurité" "$(stat -c %a "$SECU/kesh_pre_backup.sql.gz")" 600

echo "== 7. Secours : relancer le script sur le dossier du dump de sécurité (état C)"
SORTIE=$(lancer "$SCRIPTS/kesh-restore.sh" "$SECU" 2>&1); RC=$?
verifier "code de sortie" "$RC" 0
verifier "base = état C d'avant la restauration" "$(compter)" "$C"

echo "== 8. Base absente : le dump de sécurité échoue, le rechargement continue"
sql -e "DROP DATABASE kesh"
SORTIE=$(lancer "$SCRIPTS/kesh-restore.sh" "$W/restaure/dump" 2>&1); RC=$?
verifier "code de sortie" "$RC" 0
echo "$SORTIE" | grep -q "AVERTISSEMENT — base courante absente ou illisible" && ok "échec du dump de sécurité dit" || ko "avertissement absent"
verifier "base rechargée (A)" "$(compter)" "$A"
verifier "dossier de sécurité marqué en échec" "$(ls -d "$W"/kesh/avant-restauration/*-echec 2>/dev/null | wc -l)" 1
verifier "kesh-api redémarré" "$(kesh_api)" 1

echo
if [ "$ECHECS" -eq 0 ]; then echo "RECETTE VERTE"; exit 0; else echo "RECETTE ROUGE ($ECHECS échec(s))"; exit 1; fi
