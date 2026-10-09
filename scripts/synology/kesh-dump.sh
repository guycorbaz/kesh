#!/bin/bash
# kesh-dump.sh — dump de la base de Kesh, pour une installation Synology sur
# docker-compose.prod.yml (manuel d'administration, « Backup natif sur Synology DSM »).
#
# Lancé par une tâche du Planificateur de tâches de DSM (utilisateur root), AVANT
# la tâche Hyper Backup : Hyper Backup n'a pas de pré-script. Écrit, dans
# $SAUVEGARDE_DOSSIER/dump/ (forcé en 700) :
#   kesh_pre_backup.sql.gz          le dump (600), --single-transaction, utf8mb4
#   kesh_pre_backup.sql.gz.sha256   son empreinte (600)
# Le dump s'écrit d'abord en .tmp, vérifié (gzip -t, au moins un CREATE TABLE,
# ligne finale « -- Dump completed »), puis renommé : un dump raté ou vide ne
# remplace jamais celui de la veille et ne laisse aucun fichier (trap). L'empreinte
# est renommée AVANT le dump : une interruption entre les deux renommages laisse
# l'empreinte neuve à côté du dump de la veille — kesh-restore.sh refuse alors ce
# dump (sens sûr) ; relancez kesh-dump.sh. Un verrou (dump/.verrou) empêche deux
# exécutions simultanées ; laissé par un arrêt brutal, il se supprime à la main
# (rmdir). Le dump n'est JAMAIS supprimé par ce script : c'est lui que copient
# Hyper Backup et les snapshots.
#
# Réglages (variables d'environnement ; défaut entre parenthèses) :
#   SAUVEGARDE_DOSSIER  dossier du compose (/volume1/docker/kesh), qui contient
#                       kesh-dump.cnf (compte de sauvegarde, 600, root)
#   SAUVEGARDE_BASE     la base de DATABASE_URL (kesh)
#   SAUVEGARDE_RESEAU   réseau Docker qui atteint la base (frontend)
#   SAUVEGARDE_IMAGE    image du client MariaDB (mariadb:10.11)
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
SAUVEGARDE_DOSSIER=$(cd "$SAUVEGARDE_DOSSIER" && pwd -P)

[ -f "$SAUVEGARDE_DOSSIER/kesh-dump.cnf" ] || { echo "kesh-dump : $SAUVEGARDE_DOSSIER/kesh-dump.cnf absent" >&2; exit 1; }
mkdir -p "$SAUVEGARDE_DOSSIER/dump"
chmod 700 "$SAUVEGARDE_DOSSIER/dump"
cd "$SAUVEGARDE_DOSSIER/dump"
mkdir .verrou 2>/dev/null || { echo "kesh-dump : un autre dump est en cours (sinon : rmdir $SAUVEGARDE_DOSSIER/dump/.verrou)" >&2; exit 1; }
trap 'rm -f kesh_pre_backup.sql.gz.tmp kesh_pre_backup.sql.gz.sha256.tmp; rmdir .verrou' EXIT

docker run --rm --network "$SAUVEGARDE_RESEAU" \
    -v "$SAUVEGARDE_DOSSIER/kesh-dump.cnf:/etc/kesh-dump.cnf:ro" \
    "$SAUVEGARDE_IMAGE" \
    mariadb-dump --defaults-extra-file=/etc/kesh-dump.cnf --default-character-set=utf8mb4 \
    --single-transaction --add-drop-database --databases "$SAUVEGARDE_BASE" \
    | gzip > kesh_pre_backup.sql.gz.tmp
gzip -t kesh_pre_backup.sql.gz.tmp
[ "$(gzip -dc kesh_pre_backup.sql.gz.tmp | grep -c '^CREATE TABLE')" -gt 0 ] \
    || { echo "kesh-dump : dump sans aucune table — refusé, celui de la veille est gardé" >&2; exit 1; }
gzip -dc kesh_pre_backup.sql.gz.tmp | tail -n 1 | grep -q '^-- Dump completed' \
    || { echo "kesh-dump : dump incomplet (pas de « -- Dump completed ») — refusé" >&2; exit 1; }
sha256sum kesh_pre_backup.sql.gz.tmp | sed 's/\.tmp$//' > kesh_pre_backup.sql.gz.sha256.tmp
mv kesh_pre_backup.sql.gz.sha256.tmp kesh_pre_backup.sql.gz.sha256
mv kesh_pre_backup.sql.gz.tmp kesh_pre_backup.sql.gz
echo "kesh-dump : $(date '+%Y-%m-%d %H:%M:%S') — base $SAUVEGARDE_BASE dans $SAUVEGARDE_DOSSIER/dump/kesh_pre_backup.sql.gz"
