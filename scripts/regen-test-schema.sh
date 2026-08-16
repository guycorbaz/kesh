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
#   SQUASH_DB=_sqlx_test_ma_base scripts/regen-test-schema.sh
#
# Prérequis : MariaDB dev démarré, `DATABASE_URL` dans `.env` (ou l'environnement).
set -euo pipefail

cd "$(dirname "$0")/.."
OUT="crates/kesh-db/test-schema/0001_schema_squash.sql"

# ---------------------------------------------------------------------------
# 1. Coordonnées de connexion, dérivées de DATABASE_URL
#
# ⚠️ Le `|| true` sur le grep n'est pas de la négligence, c'est l'inverse : sous
# `pipefail`, un `.env` présent mais SANS la clé faisait rendre 1 au pipeline,
# donc `set -e` tuait le script AVANT le message d'erreur soigné ci-dessous.
# L'utilisateur recevait un exit 1 muet. *(Relevé en passe 1 de revue.)*
# ---------------------------------------------------------------------------
if [[ -z "${DATABASE_URL:-}" && -f .env ]]; then
    DATABASE_URL=$(grep -E '^DATABASE_URL=' .env | head -1 | cut -d= -f2- || true)
fi
if [[ -z "${DATABASE_URL:-}" ]]; then
    echo "✗ DATABASE_URL absent (ni environnement, ni .env)." >&2
    exit 1
fi

# Nettoyage de la valeur : retour chariot d'un fichier édité sous Windows, puis
# guillemets encadrants. Sans cela, `${DATABASE_URL#mysql://}` ne retire RIEN
# (le préfixe ne matche pas) et l'utilisateur devient `"mysql://kesh`.
DATABASE_URL="${DATABASE_URL%$'\r'}"
DATABASE_URL="${DATABASE_URL%\"}"; DATABASE_URL="${DATABASE_URL#\"}"
DATABASE_URL="${DATABASE_URL%\'}"; DATABASE_URL="${DATABASE_URL#\'}"

if [[ "$DATABASE_URL" != mysql://* && "$DATABASE_URL" != mariadb://* ]]; then
    echo "✗ DATABASE_URL n'a pas la forme attendue mysql://user:pass@host:port/base" >&2
    echo "  reçu : $DATABASE_URL" >&2
    exit 1
fi

# mysql://user:pass@host:port/base — le découpage coupe au DERNIER `@`, un mot
# de passe pouvant légitimement en contenir un (encodé ou non).
proto_stripped="${DATABASE_URL#*://}"
creds="${proto_stripped%@*}"
hostpart="${proto_stripped##*@}"
DB_USER="${creds%%:*}"
if [[ "$creds" == *:* ]]; then
    DB_PASS="${creds#*:}"
else
    # Sans `:`, `${creds#*:}` rendait la chaîne inchangée — donc le NOM
    # D'UTILISATEUR en guise de mot de passe. *(Relevé en passe 1.)*
    DB_PASS=""
fi
hostport="${hostpart%%/*}"
DB_HOST="${hostport%%:*}"
DB_PORT="${hostport#*:}"
[[ "$DB_PORT" == "$DB_HOST" ]] && DB_PORT=3306

# sqlx percent-décode les identifiants d'une URL ; ce script doit en faire
# autant, sans quoi il s'authentifie avec une chaîne différente de celle de
# l'application dès qu'un mot de passe contient `@`, `:` ou `/`.
percent_decode() { printf '%b' "${1//%/\\x}"; }
DB_USER=$(percent_decode "$DB_USER")
DB_PASS=$(percent_decode "$DB_PASS")

# Base JETABLE : jamais la base dev.
#
# ⚠️ Le préfixe `_sqlx_test_` n'est pas décoratif — il est CONTRÔLÉ ci-dessous.
# C'est l'espace de noms des bases jetables du harnais : s'y ranger garde le
# script exécutable là où l'utilisateur n'a de droits de création que sur ce
# préfixe (le cas dès que le grant de dev est resserré), et surtout il interdit
# à l'override `SQUASH_DB=` de viser une base réelle — le premier statement
# exécuté plus bas est un `DROP DATABASE`. *(Garde ajoutée en passe 1 de revue :
# `SQUASH_DB=kesh` détruisait la base de dev, contre la promesse du commentaire
# qui occupait ces lignes.)*
# Cette base n'est pas inscrite au registre `_sqlx_test_databases`, donc le
# ménage automatique de sqlx ne la voit pas ; le `trap` ci-dessous s'en charge.
SQUASH_DB="${SQUASH_DB:-_sqlx_test_squashgen_$$}"
if [[ ! "$SQUASH_DB" =~ ^_sqlx_test_[A-Za-z0-9_]+$ ]]; then
    echo "✗ SQUASH_DB doit correspondre à ^_sqlx_test_[A-Za-z0-9_]+$ — reçu « $SQUASH_DB »." >&2
    echo "  Ce script commence par DROP DATABASE : le préfixe est ce qui garantit" >&2
    echo "  qu'il ne peut viser ni kesh, ni kesh_e2e, ni une base de gate." >&2
    exit 1
fi

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

# Les identifiants passent par un fichier de config temporaire (mode 600) : ni
# `-p` en clair dans `ps`, ni warning client à filtrer.
#
# ⚠️ La valeur est GUILLEMETÉE : le format des fichiers d'options MariaDB traite
# `#` comme un début de commentaire et interprète `\b \t \n \s \\`. Un mot de
# passe contenant `#` était tronqué en silence. Un `"` reste impossible à
# transmettre — on le refuse plutôt que d'échouer plus loin sans motif.
# *(Relevé en passe 1 de revue.)*
if [[ "$DB_PASS" == *'"'* ]]; then
    echo "✗ mot de passe contenant un guillemet double : intransmissible par" >&2
    echo "  --defaults-extra-file. Utilisez un autre compte pour régénérer." >&2
    exit 1
fi
CNF=$(mktemp)
chmod 600 "$CNF"
cat > "$CNF" <<CNF_EOF
[client]
host=$DB_HOST
port=$DB_PORT
user="$DB_USER"
password="$DB_PASS"
CNF_EOF

cli() { "$CLI_BIN" --defaults-extra-file="$CNF" --skip-column-names --batch "$@"; }

# Le ménage est best-effort ET silencieux : à ce stade la sortie est déjà en
# cours, et un second message d'erreur masquerait le premier. C'est la SEULE
# chose que ce script tait.
#
# `RAW`/`NORM`/`TMP_OUT` y figurent : sous `set -e`, tout échec entre leur
# création et la fin les laissait dans /tmp. Et le `trap` couvre les signaux,
# pas seulement EXIT — `$CNF` contient le mot de passe de la base.
# *(Relevé par deux lentilles en passe 1.)*
RAW=""; NORM=""; TMP_OUT=""; VERIFY_DB=""
cleanup() {
    cli -e "DROP DATABASE IF EXISTS \`$SQUASH_DB\`;" >/dev/null 2>&1 || true
    [[ -n "$VERIFY_DB" ]] && cli -e "DROP DATABASE IF EXISTS \`$VERIFY_DB\`;" >/dev/null 2>&1 || true
    rm -f "$CNF" "$RAW" "$NORM" "$TMP_OUT"
}
trap cleanup EXIT INT TERM HUP

echo "▶ base jetable : $SQUASH_DB (client : $DUMP_BIN)"
cli -e "DROP DATABASE IF EXISTS \`$SQUASH_DB\`; CREATE DATABASE \`$SQUASH_DB\`;"

# ---------------------------------------------------------------------------
# 3. Le VRAI migrator, en entier, sur la base jetable
#
# `LC_ALL=C` fige l'ordre du glob : sqlx trie sur la version numérique, et le
# tri du shell dépend sinon de la locale. Les deux ordres coïncident en C, pas
# nécessairement ailleurs. *(Relevé en passe 1.)*
# ---------------------------------------------------------------------------
echo "▶ application des migrations réelles…"
MIGRATION_COUNT=0
while IFS= read -r f; do
    "$CLI_BIN" --defaults-extra-file="$CNF" "$SQUASH_DB" < "$f"
    MIGRATION_COUNT=$((MIGRATION_COUNT + 1))
done < <(LC_ALL=C ls -1 crates/kesh-db/migrations/*.sql)
echo "  $MIGRATION_COUNT migrations appliquées"

# ---------------------------------------------------------------------------
# 4. Garde-fou : le squash ne sait porter que des TABLES
#
# Un dump `--routines`/`--triggers` émet des blocs `DELIMITER ;;` — directive
# CLIENT que le serveur rejette : le squash deviendrait inchargeable. Les
# `EVENTS` seraient, eux, silencieusement OMIS (pas de `--events`), et une
# `SEQUENCE` échapperait au relevé du garde-fou, qui filtre sur
# `TABLE_TYPE = 'BASE TABLE'`. Plutôt qu'émettre un fichier incomplet, on
# échoue ici, en le disant. *(EVENTS et SEQUENCE ajoutés en passe 1 de revue.)*
# ---------------------------------------------------------------------------
EXOTIC=$(cli -e "
    SELECT
      (SELECT COUNT(*) FROM information_schema.VIEWS    WHERE TABLE_SCHEMA   = '$SQUASH_DB')
    + (SELECT COUNT(*) FROM information_schema.TRIGGERS WHERE TRIGGER_SCHEMA = '$SQUASH_DB')
    + (SELECT COUNT(*) FROM information_schema.ROUTINES WHERE ROUTINE_SCHEMA = '$SQUASH_DB')
    + (SELECT COUNT(*) FROM information_schema.EVENTS   WHERE EVENT_SCHEMA   = '$SQUASH_DB')
    + (SELECT COUNT(*) FROM information_schema.TABLES   WHERE TABLE_SCHEMA   = '$SQUASH_DB'
                                                          AND TABLE_TYPE     = 'SEQUENCE');")
# Un résultat VIDE n'est pas un zéro : c'est une requête qui n'a pas répondu.
# Le lire comme « rien d'exotique » rendait ce garde-fou muet dans le seul cas
# où il compte. *(Relevé en passe 1.)*
if [[ -z "$EXOTIC" ]]; then
    echo "✗ le décompte des objets non-table n'a rien rendu — requête en échec ?" >&2
    exit 1
fi
if [[ "$EXOTIC" -ne 0 ]]; then
    echo "✗ le schéma porte désormais $EXOTIC objet(s) non-table (vue, trigger," >&2
    echo "  routine, event ou séquence)." >&2
    echo "  Le squash ne sait pas les porter : --routines/--triggers émet des blocs" >&2
    echo "  DELIMITER que le serveur rejette, et les events seraient omis en silence." >&2
    echo "  Étendre CE script d'abord (story 22-5, #251), puis régénérer." >&2
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

# ⚠️ On écrit dans un TEMPORAIRE, et `$OUT` n'est remplacé qu'au `mv` final.
# Une redirection directe tronquait l'artefact VERSIONNÉ dès l'ouverture du
# bloc : un échec en cours laissait un squash coupé là où le run précédent était
# bon, sans que rien ne le signale. *(Relevé en passe 1 de revue.)*
mkdir -p "$(dirname "$OUT")"
TMP_OUT=$(mktemp)
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
} > "$TMP_OUT"

# ---------------------------------------------------------------------------
# 8. Le fichier produit doit S'APPLIQUER — et rendre le même nombre de tables
#
# Sans cette étape, le script affichait « ✓ » sur un squash potentiellement
# inchargeable, et la découverte était renvoyée au gate suivant… où elle casse
# 1102 tests d'un coup. Le contrôle coûte une base jetable de plus.
# *(Relevé par deux lentilles en passe 1 de revue.)*
# ---------------------------------------------------------------------------
echo "▶ vérification : rejeu du squash sur une base neuve…"
EXPECTED_TABLES=$(cli -e "SELECT COUNT(*) FROM information_schema.TABLES \
    WHERE TABLE_SCHEMA = '$SQUASH_DB' AND TABLE_TYPE = 'BASE TABLE';")
VERIFY_DB="${SQUASH_DB}_verify"
cli -e "DROP DATABASE IF EXISTS \`$VERIFY_DB\`; CREATE DATABASE \`$VERIFY_DB\`;"
if ! "$CLI_BIN" --defaults-extra-file="$CNF" "$VERIFY_DB" < "$TMP_OUT"; then
    echo "✗ le squash produit ne s'applique PAS. Fichier laissé intact ; rien n'a" >&2
    echo "  été écrit dans $OUT." >&2
    exit 1
fi
GOT_TABLES=$(cli -e "SELECT COUNT(*) FROM information_schema.TABLES \
    WHERE TABLE_SCHEMA = '$VERIFY_DB' AND TABLE_TYPE = 'BASE TABLE';")
if [[ "$GOT_TABLES" -ne "$EXPECTED_TABLES" ]]; then
    echo "✗ le squash rejoué rend $GOT_TABLES tables au lieu de $EXPECTED_TABLES." >&2
    echo "  Rien n'a été écrit dans $OUT." >&2
    exit 1
fi
if [[ "$EXPECTED_TABLES" -lt 30 ]]; then
    echo "✗ plancher : $EXPECTED_TABLES tables seulement — le dump est tronqué." >&2
    exit 1
fi

mv "$TMP_OUT" "$OUT"
TMP_OUT=""

echo "✓ $OUT — $GOT_TABLES tables (rejeu vérifié), _kesh_version = ($KV_ID, '$KV_MIN', '$KV_LAST'), $(wc -c < "$OUT") octets"
