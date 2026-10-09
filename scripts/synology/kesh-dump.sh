#!/bin/bash
# kesh-dump.sh — dump de la base de Kesh, pour une installation Synology sur
# docker-compose.prod.yml (manuel d'administration, « Backup natif sur Synology DSM »).
#
# Lancé par une tâche du Planificateur de tâches de DSM (utilisateur root), AVANT
# la tâche Hyper Backup : Hyper Backup n'a pas de pré-script. Écrit, dans
# $SAUVEGARDE_DOSSIER/dump/ (créé en 700) :
#   kesh_pre_backup.sql.gz          le dump (600), --single-transaction
#   kesh_pre_backup.sql.gz.sha256   son empreinte (600)
# Le dump s'écrit d'abord en .tmp, vérifié (gzip -t), puis renommé : un dump raté
# ne remplace jamais celui de la veille et ne laisse aucun fichier (trap). Le dump
# n'est JAMAIS supprimé par ce script : c'est lui que copient Hyper Backup et les
# snapshots.
#
# Réglages (variables d'environnement ; défaut entre parenthèses) :
#   SAUVEGARDE_DOSSIER            dossier du compose (/volume1/docker/kesh), qui contient
#                       kesh-dump.cnf (compte de sauvegarde, 600, root)
#   SAUVEGARDE_BASE           la base de DATABASE_URL (kesh)
#   SAUVEGARDE_RESEAU        réseau Docker qui atteint la base (frontend)
#   SAUVEGARDE_IMAGE  image du client MariaDB (mariadb:10.11)
#
# Code de sortie non nul sur toute erreur : la tâche planifiée le signale par
# e-mail si elle est réglée pour le faire.
set -euo pipefail
umask 077
export PATH="/usr/local/bin:/usr/bin:/bin:${PATH:-}"

SAUVEGARDE_DOSSIER=${SAUVEGARDE_DOSSIER:-/volume1/docker/kesh}
SAUVEGARDE_BASE=${SAUVEGARDE_BASE:-kesh}
SAUVEGARDE_RESEAU=${SAUVEGARDE_RESEAU:-frontend}
SAUVEGARDE_IMAGE=${SAUVEGARDE_IMAGE:-mariadb:10.11}

[ -f "$SAUVEGARDE_DOSSIER/kesh-dump.cnf" ] || { echo "kesh-dump : $SAUVEGARDE_DOSSIER/kesh-dump.cnf absent" >&2; exit 1; }
mkdir -p "$SAUVEGARDE_DOSSIER/dump"
cd "$SAUVEGARDE_DOSSIER/dump"
trap 'rm -f kesh_pre_backup.sql.gz.tmp kesh_pre_backup.sql.gz.sha256.tmp' EXIT

docker run --rm --network "$SAUVEGARDE_RESEAU" \
    -v "$SAUVEGARDE_DOSSIER/kesh-dump.cnf:/etc/kesh-dump.cnf:ro" \
    "$SAUVEGARDE_IMAGE" \
    mariadb-dump --defaults-extra-file=/etc/kesh-dump.cnf \
    --single-transaction --add-drop-database --databases "$SAUVEGARDE_BASE" \
    | gzip > kesh_pre_backup.sql.gz.tmp
gzip -t kesh_pre_backup.sql.gz.tmp
sha256sum kesh_pre_backup.sql.gz.tmp | sed 's/\.tmp$//' > kesh_pre_backup.sql.gz.sha256.tmp
mv kesh_pre_backup.sql.gz.tmp kesh_pre_backup.sql.gz
mv kesh_pre_backup.sql.gz.sha256.tmp kesh_pre_backup.sql.gz.sha256
echo "kesh-dump : $(date '+%Y-%m-%d %H:%M:%S') — base $SAUVEGARDE_BASE dans $SAUVEGARDE_DOSSIER/dump/kesh_pre_backup.sql.gz"
