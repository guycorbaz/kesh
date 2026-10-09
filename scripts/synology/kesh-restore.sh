#!/bin/bash
# kesh-restore.sh <dossier> — recharge la base de Kesh depuis un dump de kesh-dump.sh
# (manuel d'administration, « Recovery depuis Snapshot »).
#
# <dossier> contient kesh_pre_backup.sql.gz et kesh_pre_backup.sql.gz.sha256 : le
# dossier dump/ d'un snapshot ou d'une restauration Hyper Backup, ou un dossier
# avant-restauration/<horodatage>/ laissé par ce script. Chemin relatif ou absolu :
# il est résolu une fois, en absolu, avant toute autre chose.
#
# Dans cet ordre, en s'arrêtant à la première erreur :
#   1. empreinte (sha256sum -c) et intégrité (gzip -t) du dump : rien n'est
#      arrêté ni écrit si l'une échoue ;
#   2. arrêt de kesh-api (docker compose stop, dans $SAUVEGARDE_DOSSIER) ;
#   3. dump de sécurité de la base courante dans
#      $SAUVEGARDE_DOSSIER/avant-restauration/<horodatage>/ (dump + empreinte, 700/600) —
#      NON bloquant : si la base est absente ou illisible, l'échec est dit et le
#      rechargement continue (c'est précisément le cas où l'on recharge) ;
#   4. rechargement par le compte Kesh (kesh-restore.cnf), qui recrée la base
#      (DROP DATABASE, CREATE DATABASE) ;
#   5. redémarrage (docker compose up -d).
# Si l'étape 4 échoue, Kesh reste arrêté : relancez ce script sur le dossier du
# dump de sécurité qu'il a affiché (celui du PREMIER passage).
#
# Réglages : SAUVEGARDE_DOSSIER, SAUVEGARDE_BASE, SAUVEGARDE_RESEAU, SAUVEGARDE_IMAGE, comme
# kesh-dump.sh ; SAUVEGARDE_DOSSIER contient kesh-restore.cnf (compte Kesh, 600, root) et
# le compose.
set -euo pipefail
umask 077
export PATH="/usr/local/bin:/usr/bin:/bin:${PATH:-}"

[ $# -eq 1 ] || { echo "usage : kesh-restore.sh <dossier qui contient kesh_pre_backup.sql.gz>" >&2; exit 2; }
SOURCE=$(cd "$1" && pwd -P)

SAUVEGARDE_DOSSIER=${SAUVEGARDE_DOSSIER:-/volume1/docker/kesh}
SAUVEGARDE_BASE=${SAUVEGARDE_BASE:-kesh}
SAUVEGARDE_RESEAU=${SAUVEGARDE_RESEAU:-frontend}
SAUVEGARDE_IMAGE=${SAUVEGARDE_IMAGE:-mariadb:10.11}
[ -f "$SAUVEGARDE_DOSSIER/kesh-restore.cnf" ] || { echo "kesh-restore : $SAUVEGARDE_DOSSIER/kesh-restore.cnf absent" >&2; exit 1; }

echo "kesh-restore : dump à recharger : $SOURCE/kesh_pre_backup.sql.gz"
( cd "$SOURCE" && sha256sum -c kesh_pre_backup.sql.gz.sha256 )
gzip -t "$SOURCE/kesh_pre_backup.sql.gz"

cd "$SAUVEGARDE_DOSSIER"
docker compose stop kesh-api

SECURITE="$SAUVEGARDE_DOSSIER/avant-restauration/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$SECURITE"
if docker run --rm --network "$SAUVEGARDE_RESEAU" \
       -v "$SAUVEGARDE_DOSSIER/kesh-restore.cnf:/etc/kesh-restore.cnf:ro" \
       "$SAUVEGARDE_IMAGE" \
       mariadb-dump --defaults-extra-file=/etc/kesh-restore.cnf \
       --single-transaction --add-drop-database --databases "$SAUVEGARDE_BASE" \
       | gzip > "$SECURITE/kesh_pre_backup.sql.gz" \
   && gzip -t "$SECURITE/kesh_pre_backup.sql.gz"; then
    ( cd "$SECURITE" && sha256sum kesh_pre_backup.sql.gz > kesh_pre_backup.sql.gz.sha256 )
    echo "kesh-restore : dump de sécurité de la base courante : $SECURITE"
else
    mv "$SECURITE" "$SECURITE-echec"
    echo "kesh-restore : AVERTISSEMENT — base courante absente ou illisible, aucun dump de sécurité (voir $SECURITE-echec) ; le rechargement continue" >&2
fi

gunzip -c "$SOURCE/kesh_pre_backup.sql.gz" | docker run --rm -i --network "$SAUVEGARDE_RESEAU" \
    -v "$SAUVEGARDE_DOSSIER/kesh-restore.cnf:/etc/kesh-restore.cnf:ro" \
    "$SAUVEGARDE_IMAGE" \
    mariadb --defaults-extra-file=/etc/kesh-restore.cnf

docker compose up -d
echo "kesh-restore : base $SAUVEGARDE_BASE rechargée depuis $SOURCE ; Kesh redémarré"
