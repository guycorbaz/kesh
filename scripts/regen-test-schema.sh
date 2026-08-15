#!/usr/bin/env bash
#
# Régénère le SQUASH DU SCHÉMA DE TEST — Story 22-5 (#251).
#
# Le squash (`crates/kesh-db/test-schema/0001_schema_squash.sql`) est l'unique
# migration que rejouent les bases éphémères de `#[sqlx::test]` : un batch DDL
# au lieu des 61 cycles INSERT/DDL/UPDATE du vrai `MIGRATOR`.
#
# ⚠️ CE FICHIER SE RÉGÉNÈRE, IL NE S'ÉDITE JAMAIS. Le garde-fou
# `crates/kesh-db/tests/test_schema_guard.rs` compare, à chaque gate, le schéma
# monté par le squash à celui monté par le vrai `MIGRATOR` : toute migration
# ajoutée sans passer ce script fait rougir le gate en nommant la divergence.
#
# Usage :
#   scripts/regen-test-schema.sh                 # base jetable auto-nommée
#   SQUASH_DB=ma_base scripts/regen-test-schema.sh
#
# Prérequis : MariaDB dev démarré, `DATABASE_URL` dans `.env` (ou l'environnement).
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$PWD"
OUT="crates/kesh-db/test-schema/0001_schema_squash.sql"

# ---------------------------------------------------------------------------
# 1. Coordonnées de connexion, dérivées de DATABASE_URL
# ---------------------------------------------------------------------------
if [[ -z "${DATABASE_URL:-}" && -f .env ]]; then
    DATABASE_URL=$(grep -E '^DATABASE_URL=' .env | head -1 | cut -d= -f2-)
fi
if [[ -z "${DATABASE_URL:-}" ]]; then
    echo "✗ DATABASE_URL absent (ni environnement, ni .env)." >&2
    exit 1
fi

# mysql://user:pass@host:port/base
proto_stripped="${DATABASE_URL#mysql://}"
creds="${proto_stripped%%@*}"
hostpart="${proto_stripped#*@}"
DB_USER="${creds%%:*}"
DB_PASS="${creds#*:}"
hostport="${hostpart%%/*}"
DB_HOST="${hostport%%:*}"
DB_PORT="${hostport#*:}"
[[ "$DB_PORT" == "$DB_HOST" ]] && DB_PORT=3306

# Base JETABLE : jamais la base dev, que ce script ne touche pas.
#
# ⚠️ Le préfixe `_sqlx_test_` n'est pas décoratif : l'utilisateur applicatif n'a
# de droits de création que sur `kesh`, `kesh_e2e`, les bases de gate… et
# `_sqlx_test%` — c'est CE grant qui fait vivre `#[sqlx::test]`. S'y ranger rend
# le script exécutable partout où le harnais de test tourne, sans exiger root.
# Cette base n'est pas inscrite au registre `_sqlx_test_databases`, donc le
# ménage automatique de sqlx ne la voit pas ; le `trap` ci-dessous s'en charge.
SQUASH_DB="${SQUASH_DB:-_sqlx_test_squashgen_$$}"

# ---------------------------------------------------------------------------
# 2. Client de dump — `mariadb-dump` OU `mysqldump` selon les machines
# ---------------------------------------------------------------------------
if command -v mariadb-dump >/dev/null 2>&1; then
    DUMP_BIN=mariadb-dump
elif command -v mysqldump >/dev/null 2>&1; then
    DUMP_BIN=mysqldump
else
    echo "✗ ni mariadb-dump ni mysqldump : installez le paquet client MariaDB." >&2
    exit 1
fi
if command -v mariadb >/dev/null 2>&1; then
    CLI_BIN=mariadb
elif command -v mysql >/dev/null 2>&1; then
    CLI_BIN=mysql
else
    echo "✗ ni mariadb ni mysql (client CLI) : installez le paquet client MariaDB." >&2
    exit 1
fi

# Les identifiants passent par un fichier de config temporaire (mode 600) :
# ni `-p` en clair dans `ps`, ni warning client à filtrer — donc AUCUNE erreur
# n'a besoin d'être masquée, et le script échoue bruyamment quand il échoue.
CNF=$(mktemp)
chmod 600 "$CNF"
cat > "$CNF" <<CNF_EOF
[client]
host=$DB_HOST
port=$DB_PORT
user=$DB_USER
password=$DB_PASS
CNF_EOF

cli() { "$CLI_BIN" --defaults-extra-file="$CNF" --skip-column-names --batch "$@"; }

cleanup() {
    cli -e "DROP DATABASE IF EXISTS \`$SQUASH_DB\`;" >/dev/null 2>&1 || true
    rm -f "$CNF"
}
trap cleanup EXIT

echo "▶ base jetable : $SQUASH_DB (client : $DUMP_BIN)"
cli -e "DROP DATABASE IF EXISTS \`$SQUASH_DB\`; CREATE DATABASE \`$SQUASH_DB\`;"

# ---------------------------------------------------------------------------
# 3. Le VRAI migrator, en entier, sur la base jetable
# ---------------------------------------------------------------------------
echo "▶ application des migrations réelles…"
MIGRATION_COUNT=0
for f in crates/kesh-db/migrations/*.sql; do
    "$CLI_BIN" --defaults-extra-file="$CNF" "$SQUASH_DB" < "$f"
    MIGRATION_COUNT=$((MIGRATION_COUNT + 1))
done
echo "  $MIGRATION_COUNT migrations appliquées"

# ---------------------------------------------------------------------------
# 4. Garde-fou : le squash ne sait pas porter vues / triggers / routines
#
# Un dump `--routines`/`--triggers` émet des blocs `DELIMITER ;;` — directive
# CLIENT que le serveur rejette : le squash deviendrait inchargeable. Plutôt
# qu'émettre un fichier cassé, on échoue ici, en le disant.
# ---------------------------------------------------------------------------
EXOTIC=$(cli -e "
    SELECT
      (SELECT COUNT(*) FROM information_schema.VIEWS   WHERE TABLE_SCHEMA = '$SQUASH_DB')
    + (SELECT COUNT(*) FROM information_schema.TRIGGERS WHERE TRIGGER_SCHEMA = '$SQUASH_DB')
    + (SELECT COUNT(*) FROM information_schema.ROUTINES WHERE ROUTINE_SCHEMA = '$SQUASH_DB');")
if [[ "${EXOTIC:-0}" -ne 0 ]]; then
    echo "✗ le schéma porte désormais $EXOTIC vue(s)/trigger(s)/routine(s)." >&2
    echo "  Le squash ne sait pas les porter : un dump --routines/--triggers émet des" >&2
    echo "  blocs DELIMITER, directive client que le serveur rejette. Étendre CE script" >&2
    echo "  d'abord (story 22-5, #251), puis régénérer." >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# 5. Dump DDL — sans données, SANS `_sqlx_migrations`
#
# ⚠️ L'exclusion de `_sqlx_migrations` n'est pas cosmétique, et son mode
# d'échec est MUET : sqlx crée cette table AVANT d'appliquer la migration et y
# insère sa ligne de suivi ; un `DROP TABLE IF EXISTS` du dump (émis par défaut)
# la détruirait, le `CREATE` la recréerait vide, et l'`UPDATE success=TRUE`
# final affecterait ZÉRO ligne — sans erreur. Le schéma serait identique, seul
# le suivi serait perdu. C'est la 3ᵉ assertion du garde-fou qui l'attrape.
# ---------------------------------------------------------------------------
echo "▶ dump DDL…"
RAW=$(mktemp)
"$DUMP_BIN" --defaults-extra-file="$CNF" \
    --no-data --skip-triggers --skip-comments \
    --ignore-table="$SQUASH_DB._sqlx_migrations" \
    "$SQUASH_DB" > "$RAW"

# ---------------------------------------------------------------------------
# 6. Normalisation — LISTE EXACTE, que le garde-fou exclut symétriquement
#
#   (a) `AUTO_INCREMENT=<n>` — volatil (dépend des insertions du run) ;
#   (b) lignes de commentaire `--` résiduelles et lignes vides multiples.
#
# Rien d'autre n'est retouché : les directives `/*!40101 … */` d'en-tête et de
# pied (dont la sauvegarde/restauration de FOREIGN_KEY_CHECKS et de SQL_MODE)
# sont CONSERVÉES telles quelles — c'est ce qui rend le rejeu insensible à
# l'ordre alphabétique des tables face aux clés étrangères.
# ---------------------------------------------------------------------------
NORM=$(mktemp)
sed -E 's/ AUTO_INCREMENT=[0-9]+//g' "$RAW" | grep -vE '^--' | cat -s > "$NORM"

# ---------------------------------------------------------------------------
# 7. Réinjection de la ligne `_kesh_version`
#
# ⚠️ Elle se LIT dans la base migrée, jamais ne se code en dur : deux migrations
# réécrivent déjà `kesh_version_min_required` (0.7.0 puis 0.10.0), et chaque
# futur bump P2 la déplacerait encore. `applied_at` est OMIS (défaut serveur).
# Sans cette ligne, `check_downgrade_protection` et le verrou d'installation
# (`FOR UPDATE` sur id=1) changeraient de comportement dans les tests.
# ---------------------------------------------------------------------------
# Trois requêtes scalaires plutôt qu'un CONCAT_WS découpé : le passage par un
# séparateur est une source d'erreur silencieuse de plus (l'échappement du \t
# traverse deux couches de quoting avant d'atteindre le serveur), et ce script
# a déjà payé une fois le prix d'un échec muet.
KV_ID=$(cli "$SQUASH_DB" -e "SELECT id FROM _kesh_version WHERE id = 1;")
KV_MIN=$(cli "$SQUASH_DB" -e "SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1;")
KV_LAST=$(cli "$SQUASH_DB" -e "SELECT kesh_version_last_applied FROM _kesh_version WHERE id = 1;")
if [[ -z "$KV_ID" || -z "$KV_MIN" || -z "$KV_LAST" ]]; then
    echo "✗ _kesh_version ne porte aucune ligne id=1 exploitable après migration." >&2
    exit 1
fi

mkdir -p "$(dirname "$OUT")"
{
    echo "-- SQUASH DU SCHÉMA DE TEST — Story 22-5 (#251). GÉNÉRÉ, NE PAS ÉDITER."
    echo "-- Régénérer : scripts/regen-test-schema.sh"
    echo "-- Équivalent des $MIGRATION_COUNT migrations de crates/kesh-db/migrations/,"
    echo "-- rejouées en UN batch DDL par base éphémère de test."
    echo "--"
    echo "-- Le garde-fou crates/kesh-db/tests/test_schema_guard.rs compare ce schéma"
    echo "-- au vrai à chaque gate : une migration ajoutée sans régénération rougit."
    echo ""
    cat "$NORM"
    echo ""
    echo "-- Ligne d'installation, RELEVÉE dans la base migrée (jamais codée en dur :"
    echo "-- chaque bump P2 de kesh_version_min_required la déplace)."
    echo "INSERT INTO \`_kesh_version\` (\`id\`, \`kesh_version_min_required\`, \`kesh_version_last_applied\`)"
    echo "VALUES ($KV_ID, '$KV_MIN', '$KV_LAST');"
} > "$OUT"

rm -f "$RAW" "$NORM"

TABLES=$(grep -c '^CREATE TABLE' "$OUT" || true)
echo "✓ $OUT — $TABLES tables, _kesh_version = ($KV_ID, '$KV_MIN', '$KV_LAST'), $(wc -c < "$OUT") octets"
