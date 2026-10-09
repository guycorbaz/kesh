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
#   1. empreinte (sha256sum -c) et intégrité (gzip -t) du dump, puis sonde de la
#      base sur le serveur : rien n'est arrêté ni écrit si l'une échoue (serveur
#      injoignable, compte refusé…) ;
#   2. arrêt de kesh-api (docker compose -p <projet> stop), vérifié ;
#   3. si la base EXISTE : dump de sécurité de la base courante dans
#      $SAUVEGARDE_DOSSIER/avant-restauration/<horodatage>/ (dump + empreinte,
#      700/600). S'il échoue (disque plein, dossier non inscriptible…), le script
#      REDÉMARRE Kesh et SORT sans rien recharger : la base est intacte.
#      Si la base est ABSENTE (établi par la sonde) : pas de dump de sécurité,
#      c'est dit, et le rechargement continue — c'est le cas même où l'on recharge ;
#   4. rechargement par le compte Kesh (kesh-restore.cnf), qui recrée la base
#      (DROP DATABASE, CREATE DATABASE) ;
#   5. redémarrage (docker compose -p <projet> up -d).
# Si l'étape 4 échoue ou est interrompue (session SSH coupée), Kesh reste arrêté :
# relancez ce script sur le dossier du dump de sécurité qu'il a affiché (celui du
# PREMIER passage), ou sur le même dossier de dump.
#
# Réglages : SAUVEGARDE_DOSSIER, SAUVEGARDE_BASE, SAUVEGARDE_RESEAU,
# SAUVEGARDE_IMAGE, comme kesh-dump.sh ; SAUVEGARDE_PROJET, nom du projet compose
# (kesh, celui que le manuel fait créer). SAUVEGARDE_DOSSIER contient
# kesh-restore.cnf (compte Kesh, 600, root) et le compose.
set -euo pipefail
umask 077
export PATH="/usr/local/bin:/usr/bin:/bin:${PATH:-}"

[ $# -eq 1 ] || { echo "usage : kesh-restore.sh <dossier qui contient kesh_pre_backup.sql.gz>" >&2; exit 2; }
SOURCE=$(cd "$1" && pwd -P)

SAUVEGARDE_DOSSIER=${SAUVEGARDE_DOSSIER:-/volume1/docker/kesh}
SAUVEGARDE_BASE=${SAUVEGARDE_BASE:-kesh}
SAUVEGARDE_RESEAU=${SAUVEGARDE_RESEAU:-frontend}
SAUVEGARDE_IMAGE=${SAUVEGARDE_IMAGE:-mariadb:10.11}
SAUVEGARDE_PROJET=${SAUVEGARDE_PROJET:-kesh}
SAUVEGARDE_DOSSIER=$(cd "$SAUVEGARDE_DOSSIER" && pwd -P)
[ -f "$SAUVEGARDE_DOSSIER/kesh-restore.cnf" ] || { echo "kesh-restore : $SAUVEGARDE_DOSSIER/kesh-restore.cnf absent" >&2; exit 1; }

# Client MariaDB jetable, par le compte Kesh (kesh-restore.cnf), entrée standard ouverte (-i).
client() {
    docker run --rm -i --network "$SAUVEGARDE_RESEAU" \
        -v "$SAUVEGARDE_DOSSIER/kesh-restore.cnf:/etc/kesh-restore.cnf:ro" \
        "$SAUVEGARDE_IMAGE" "$@"
}

echo "kesh-restore : dump à recharger : $SOURCE/kesh_pre_backup.sql.gz"
( cd "$SOURCE" && sha256sum -c kesh_pre_backup.sql.gz.sha256 )
gzip -t "$SOURCE/kesh_pre_backup.sql.gz"
PRESENTE=$(client mariadb --defaults-extra-file=/etc/kesh-restore.cnf -N -e \
    "SELECT COUNT(*) FROM information_schema.SCHEMATA WHERE SCHEMA_NAME = '$SAUVEGARDE_BASE'" </dev/null)
case "$PRESENTE" in 0|1) ;; *) echo "kesh-restore : sonde de la base illisible ($PRESENTE)" >&2; exit 1;; esac

cd "$SAUVEGARDE_DOSSIER"
docker compose -p "$SAUVEGARDE_PROJET" stop kesh-api
if docker compose -p "$SAUVEGARDE_PROJET" ps --status running --services | grep -qx kesh-api; then
    echo "kesh-restore : kesh-api tourne encore (projet $SAUVEGARDE_PROJET ?) — rien n'a été rechargé" >&2; exit 1
fi

if [ "$PRESENTE" = 1 ]; then
    SECURITE="$SAUVEGARDE_DOSSIER/avant-restauration/$(date +%Y%m%d-%H%M%S)"
    if mkdir -p "$SECURITE" \
       && client mariadb-dump --defaults-extra-file=/etc/kesh-restore.cnf --default-character-set=utf8mb4 \
              --single-transaction --add-drop-database --databases "$SAUVEGARDE_BASE" </dev/null \
          | gzip > "$SECURITE/kesh_pre_backup.sql.gz" \
       && gzip -t "$SECURITE/kesh_pre_backup.sql.gz" \
       && ( cd "$SECURITE" && sha256sum kesh_pre_backup.sql.gz > kesh_pre_backup.sql.gz.sha256 ); then
        echo "kesh-restore : dump de sécurité de la base courante : $SECURITE"
    else
        docker compose -p "$SAUVEGARDE_PROJET" up -d
        echo "kesh-restore : ÉCHEC — dump de sécurité impossible alors que la base $SAUVEGARDE_BASE existe ; rien n'a été rechargé, la base est intacte, Kesh est redémarré" >&2
        exit 1
    fi
else
    echo "kesh-restore : la base $SAUVEGARDE_BASE n'existe pas sur le serveur — pas de dump de sécurité ; rechargement"
fi

gunzip -c "$SOURCE/kesh_pre_backup.sql.gz" \
    | client mariadb --defaults-extra-file=/etc/kesh-restore.cnf --default-character-set=utf8mb4

docker compose -p "$SAUVEGARDE_PROJET" up -d
echo "kesh-restore : base $SAUVEGARDE_BASE rechargée depuis $SOURCE ; Kesh redémarré"
