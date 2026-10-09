#!/bin/bash
# kesh-restore.sh <dossier> — recharge la base de Kesh depuis un dump de kesh-dump.sh
# (manuel d'administration, « Recovery depuis Snapshot »).
#
# Ce script ne touche PAS à Kesh : arrêtez kesh-api AVANT, redémarrez-le APRÈS
# (dans le dossier du compose : docker compose stop kesh-api / docker compose start
# kesh-api). Il REFUSE de démarrer si un conteneur du service kesh-api est dans un
# autre état que exited, created ou dead — en marche, en boucle de redémarrage
# (restarting), en pause… —, sur tout l'hôte (label com.docker.compose.service,
# quel que soit le projet).
#
# <dossier> contient kesh_pre_backup.sql.gz et kesh_pre_backup.sql.gz.sha256 : le
# dossier dump/ d'un snapshot ou d'une restauration Hyper Backup, ou un dossier
# avant-restauration/<horodatage>/ laissé par ce script. Chemin relatif ou absolu :
# il est résolu en absolu avant toute autre chose.
#
# Dans cet ordre, en s'arrêtant à la première erreur. La base n'est touchée qu'à
# l'étape 6, aucun dump n'est écrit avant l'étape 5 ; avant, le script n'écrit que
# son verrou (dump/.verrou) :
#   1. refus si un conteneur kesh-api est actif ;
#   2. verrou du dump (dump/.verrou, celui de kesh-dump.sh) : pas de dump nocturne
#      pendant la restauration, ni deux restaurations à la fois ;
#   3. empreinte (sha256sum -c) et intégrité (gzip -t) du dump, puis nom de la base
#      qu'il recrée (CREATE DATABASE) : refus s'il diffère de SAUVEGARDE_BASE ;
#   4. sonde de la base sur le serveur, trois états (refus si la sonde échoue) :
#      ABSENTE, présente VIDE (aucune table), présente AVEC TABLES ;
#   5. base AVEC TABLES : dump de sécurité OBLIGATOIRE dans
#      avant-restauration/<horodatage>/, pris par kesh-dump.sh lui-même (mêmes
#      contrôles, .tmp puis renommage, empreinte), par le compte Kesh. S'il échoue
#      (disque plein, base illisible…), le script s'arrête : rien n'est rechargé, la
#      base est intacte. Une base illisible ne se recharge qu'après l'avoir supprimée
#      à la main, geste délibéré que décrit le manuel. Base ABSENTE ou VIDE : rien à
#      protéger, pas de dump de sécurité, c'est dit ;
#   6. rechargement par le compte Kesh (kesh-restore.cnf) : le dump recrée la base
#      (DROP DATABASE, CREATE DATABASE).
#
# Reprise après une interruption ou un échec de l'étape 6 (la base peut être à
# moitié rechargée ; Kesh est toujours arrêté). En cas d'échec, le script affiche
# les deux gestes et la liste des dossiers de avant-restauration/, du plus ancien
# au plus récent — il ne devine pas lequel est le bon :
#   - TERMINER la restauration : relancer ce script sur le MÊME dossier ;
#   - REVENIR à l'état d'avant : relancer ce script sur le PLUS ANCIEN dossier de
#     avant-restauration/ pris depuis le DÉBUT de cette restauration, c'est-à-dire
#     au premier passage (chaque passage suivant en prend un nouveau, de la base à
#     moitié rechargée ; les dossiers plus anciens viennent de restaurations
#     antérieures). Si le premier passage n'en a pas pris (base absente ou vide),
#     il n'y a pas d'état d'avant à retrouver.
#
# Réglages : SAUVEGARDE_DOSSIER, SAUVEGARDE_BASE, SAUVEGARDE_RESEAU,
# SAUVEGARDE_IMAGE, comme kesh-dump.sh (qui doit être dans le même dossier que ce
# script). SAUVEGARDE_DOSSIER contient kesh-restore.cnf (compte Kesh, 600, root).
set -euo pipefail
umask 077
export PATH="/usr/local/bin:/usr/bin:/bin:${PATH:-}"

[ $# -eq 1 ] || { echo "usage : kesh-restore.sh <dossier qui contient kesh_pre_backup.sql.gz>" >&2; exit 2; }
SOURCE=$(cd "$1" && pwd -P)
ICI=$(cd "$(dirname "$0")" && pwd -P)

SAUVEGARDE_DOSSIER=${SAUVEGARDE_DOSSIER:-/volume1/docker/kesh}
SAUVEGARDE_BASE=${SAUVEGARDE_BASE:-kesh}
SAUVEGARDE_RESEAU=${SAUVEGARDE_RESEAU:-frontend}
SAUVEGARDE_IMAGE=${SAUVEGARDE_IMAGE:-mariadb:10.11}
SAUVEGARDE_DOSSIER=$(cd "$SAUVEGARDE_DOSSIER" && pwd -P)
[ -f "$SAUVEGARDE_DOSSIER/kesh-restore.cnf" ] || { echo "kesh-restore : $SAUVEGARDE_DOSSIER/kesh-restore.cnf absent" >&2; exit 1; }
[ -f "$ICI/kesh-dump.sh" ] || { echo "kesh-restore : $ICI/kesh-dump.sh absent (les deux scripts vont ensemble)" >&2; exit 1; }

# Client MariaDB jetable, par le compte Kesh (kesh-restore.cnf), entrée standard ouverte (-i).
client() {
    docker run --rm -i --network "$SAUVEGARDE_RESEAU" \
        -v "$SAUVEGARDE_DOSSIER/kesh-restore.cnf:/etc/kesh-restore.cnf:ro" \
        "$SAUVEGARDE_IMAGE" "$@"
}

# 1. Aucun conteneur kesh-api actif — une erreur de docker vaut refus (set -e).
ETATS=$(docker ps --all --filter label=com.docker.compose.service=kesh-api --format '{{.State}}')
for ETAT in $ETATS; do
    case "$ETAT" in exited|created|dead) ;; *) echo "kesh-restore : un conteneur kesh-api est « $ETAT » — arrêtez-le d'abord (docker compose stop kesh-api) ; rien n'a été touché" >&2; exit 1;; esac
done

# 2. Verrou du dump, avant de lire le dump : le dossier donné peut être le dump/ vivant.
mkdir -p "$SAUVEGARDE_DOSSIER/dump"
mkdir "$SAUVEGARDE_DOSSIER/dump/.verrou" 2>/dev/null || { echo "kesh-restore : une sauvegarde ou une restauration est en cours (sinon : rmdir $SAUVEGARDE_DOSSIER/dump/.verrou) — rien n'a été touché" >&2; exit 1; }
trap 'rmdir "$SAUVEGARDE_DOSSIER/dump/.verrou"' EXIT
trap 'exit 1' HUP INT TERM

# 3. Le dump, et la base qu'il recrée.
echo "kesh-restore : dump à recharger : $SOURCE/kesh_pre_backup.sql.gz"
( cd "$SOURCE" && sha256sum -c kesh_pre_backup.sql.gz.sha256 )
gzip -t "$SOURCE/kesh_pre_backup.sql.gz"
BASE_DU_DUMP=$(gzip -dc "$SOURCE/kesh_pre_backup.sql.gz" \
    | awk '/^CREATE DATABASE/ { n++; if (match($0, /`[^`]+`/)) b = substr($0, RSTART + 1, RLENGTH - 2) } END { print (n == 1 ? b : "") }')
[ "$BASE_DU_DUMP" = "$SAUVEGARDE_BASE" ] || { echo "kesh-restore : le dump recrée la base « ${BASE_DU_DUMP:-?} », non « $SAUVEGARDE_BASE » (SAUVEGARDE_BASE) — rien n'a été touché" >&2; exit 1; }

# 4. Sonde : la base existe-t-elle, et combien de tables porte-t-elle ?
SONDE=$(client mariadb --defaults-extra-file=/etc/kesh-restore.cnf -N -e \
    "SELECT (SELECT COUNT(*) FROM information_schema.SCHEMATA WHERE SCHEMA_NAME = '$SAUVEGARDE_BASE'), (SELECT COUNT(*) FROM information_schema.TABLES WHERE TABLE_SCHEMA = '$SAUVEGARDE_BASE')" </dev/null)
read -r PRESENTE NB_TABLES <<< "$SONDE"
[[ "$PRESENTE" =~ ^[01]$ && "${NB_TABLES:-}" =~ ^[0-9]+$ ]] || { echo "kesh-restore : sonde de la base illisible ($SONDE) — rien n'a été touché" >&2; exit 1; }

# 5. Dump de sécurité, obligatoire si la base porte des tables.
SECURITE=""
if [ "$PRESENTE" = 1 ] && [ "$NB_TABLES" -gt 0 ]; then
    SECURITE="$SAUVEGARDE_DOSSIER/avant-restauration/$(date +%Y%m%d-%H%M%S)"
    export SAUVEGARDE_DOSSIER SAUVEGARDE_BASE SAUVEGARDE_RESEAU SAUVEGARDE_IMAGE
    DUMP_CIBLE="$SECURITE" DUMP_COMPTE=kesh-restore.cnf bash "$ICI/kesh-dump.sh" \
        || { rmdir "$SECURITE" 2>/dev/null || true; echo "kesh-restore : ÉCHEC — dump de sécurité impossible alors que la base $SAUVEGARDE_BASE porte $NB_TABLES table(s) ; rien n'a été rechargé, la base est intacte (base illisible : manuel, « Base présente mais illisible »)" >&2; exit 1; }
    echo "kesh-restore : dump de sécurité de la base courante : $SECURITE"
elif [ "$PRESENTE" = 1 ]; then
    echo "kesh-restore : la base $SAUVEGARDE_BASE existe mais ne porte aucune table — rien à protéger, pas de dump de sécurité ; rechargement"
else
    echo "kesh-restore : la base $SAUVEGARDE_BASE n'existe pas sur le serveur — pas de dump de sécurité ; rechargement"
fi

# 6. Rechargement.
if ! gunzip -c "$SOURCE/kesh_pre_backup.sql.gz" \
    | client mariadb --defaults-extra-file=/etc/kesh-restore.cnf --default-character-set=utf8mb4; then
    echo "kesh-restore : ÉCHEC du rechargement — Kesh reste arrêté, la base peut être à moitié rechargée." >&2
    echo "  terminer : relancez ce script sur $SOURCE" >&2
    echo "  revenir  : relancez-le sur le PLUS ANCIEN dossier de avant-restauration/ pris depuis le début de cette restauration (au premier passage) ; ce script ne peut pas le deviner. Dossiers présents, du plus ancien au plus récent :" >&2
    for D in "$SAUVEGARDE_DOSSIER"/avant-restauration/*/; do
        if [ -d "$D" ]; then echo "      ${D%/}" >&2; fi
    done
    echo "    pour information, ce passage : ${SECURITE:-aucun dump de sécurité (base absente ou vide)} ; si le premier passage n'en a pas pris, il n'y a pas d'état d'avant à retrouver" >&2
    exit 1
fi
echo "kesh-restore : base $SAUVEGARDE_BASE rechargée depuis $SOURCE — redémarrez Kesh (docker compose start kesh-api)"
