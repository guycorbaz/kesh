#!/bin/bash
# kesh-dump.sh — dump de la base de Kesh, pour une installation Synology sur
# docker-compose.prod.yml (manuel d'administration, « Backup natif sur Synology DSM »).
#
# Lancé par une tâche du Planificateur de tâches de DSM (utilisateur root), AVANT
# la tâche Hyper Backup : Hyper Backup n'a pas de pré-script. Écrit, dans le dossier
# cible (par défaut $SAUVEGARDE_DOSSIER/dump/, forcé en 700) :
#   kesh_pre_backup.sql.gz          le dump (600), --single-transaction, utf8mb4
#   kesh_pre_backup.sql.gz.sha256   son empreinte (600)
# Le dump s'écrit d'abord en .tmp, vérifié (gzip -t, au moins un CREATE TABLE,
# ligne finale « -- Dump completed »), puis renommé : un dump raté ou vide ne
# remplace jamais le précédent et ne laisse aucun fichier (trap, signaux compris).
# L'empreinte est renommée AVANT le dump : une interruption entre les deux
# renommages laisse l'empreinte neuve à côté du dump précédent, que kesh-restore.sh
# refuse (sens sûr) — relancez kesh-dump.sh. Un verrou (.verrou dans le dossier
# cible) empêche deux dumps simultanés ; kesh-restore.sh prend le même verrou
# pendant toute une restauration. Laissé par un arrêt brutal (ou rendu par un
# snapshot restauré), il se supprime à la main : rmdir <dossier>/.verrou. Le dump
# n'est JAMAIS supprimé par ce script : c'est lui que copient Hyper Backup et les
# snapshots.
#
# Réglages (variables d'environnement ; défaut entre parenthèses) :
#   SAUVEGARDE_DOSSIER  dossier du compose (/volume1/docker/kesh), qui contient
#                       kesh-dump.cnf (compte de sauvegarde, 600, root)
#   SAUVEGARDE_BASE     la base de DATABASE_URL (kesh)
#   SAUVEGARDE_RESEAU   réseau Docker qui atteint la base (frontend)
#   SAUVEGARDE_IMAGE    image du client MariaDB (mariadb:10.11)
# Réglages internes, posés par kesh-restore.sh pour son dump de sécurité (un
# exploitant n'a pas à les toucher) :
#   DUMP_CIBLE   dossier où écrire ($SAUVEGARDE_DOSSIER/dump)
#   DUMP_COMPTE  fichier d'identifiants, dans $SAUVEGARDE_DOSSIER (kesh-dump.cnf)
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
DUMP_CIBLE=${DUMP_CIBLE:-$SAUVEGARDE_DOSSIER/dump}
DUMP_COMPTE=${DUMP_COMPTE:-kesh-dump.cnf}

[ -f "$SAUVEGARDE_DOSSIER/$DUMP_COMPTE" ] || { echo "kesh-dump : $SAUVEGARDE_DOSSIER/$DUMP_COMPTE absent" >&2; exit 1; }
mkdir -p "$DUMP_CIBLE"
chmod 700 "$DUMP_CIBLE"
cd "$DUMP_CIBLE"
mkdir .verrou 2>/dev/null || { echo "kesh-dump : une sauvegarde ou une restauration est en cours (sinon : rmdir $DUMP_CIBLE/.verrou)" >&2; exit 1; }
trap 'rm -f kesh_pre_backup.sql.gz.tmp kesh_pre_backup.sql.gz.sha256.tmp; rmdir .verrou' EXIT
trap 'exit 1' HUP INT TERM

docker run --rm --network "$SAUVEGARDE_RESEAU" \
    -v "$SAUVEGARDE_DOSSIER/$DUMP_COMPTE:/etc/compte.cnf:ro" \
    "$SAUVEGARDE_IMAGE" \
    mariadb-dump --defaults-extra-file=/etc/compte.cnf --default-character-set=utf8mb4 \
    --single-transaction --add-drop-database --databases "$SAUVEGARDE_BASE" \
    | gzip > kesh_pre_backup.sql.gz.tmp
gzip -t kesh_pre_backup.sql.gz.tmp
[ "$(gzip -dc kesh_pre_backup.sql.gz.tmp | grep -c '^CREATE TABLE')" -gt 0 ] \
    || { echo "kesh-dump : dump sans aucune table — refusé, le précédent est gardé" >&2; exit 1; }
gzip -dc kesh_pre_backup.sql.gz.tmp | tail -n 1 | grep -q '^-- Dump completed' \
    || { echo "kesh-dump : dump incomplet (pas de « -- Dump completed ») — refusé" >&2; exit 1; }
sha256sum kesh_pre_backup.sql.gz.tmp | sed 's/\.tmp$//' > kesh_pre_backup.sql.gz.sha256.tmp
mv kesh_pre_backup.sql.gz.sha256.tmp kesh_pre_backup.sql.gz.sha256
mv kesh_pre_backup.sql.gz.tmp kesh_pre_backup.sql.gz
echo "kesh-dump : $(date '+%Y-%m-%d %H:%M:%S') — base $SAUVEGARDE_BASE dans $DUMP_CIBLE/kesh_pre_backup.sql.gz"
