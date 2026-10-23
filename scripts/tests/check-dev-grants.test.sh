#!/usr/bin/env bash
# check-dev-grants.test.sh — éprouve `scripts/check-dev-grants.sh` (le garde des
# droits du MariaDB de dev) sans jamais toucher au vrai `docker` ni au vrai
# conteneur, sur le poste comme en CI.
#
# Chaque cas bâtit sous `mktemp -d` un MINI-DÉPÔT : une copie du garde, un
# `scripts/mariadb-init/01-dev-grants.sql` de référence ÉCRIT ICI (contenu figé)
# et un `docker-compose.dev.yml` de référence portant un mot de passe
# distinctif. Le vrai fichier de droits n'est jamais lu : une branche qui le
# modifie légitimement ne fait pas rougir ce test (Story 31-2, AC3).
#
# `KESH_DOCKER` — la seule injection du garde — pointe vers un `docker` factice
# qui CONSIGNE son argv et la valeur de `$MYSQL_PWD`, rend une sortie choisie
# par le cas, et échoue bruyamment sur toute sous-commande imprévue.
#
# Seule l'assertion de câblage lit un fichier réel : `scripts/test-fast.sh`.
#
# Lancé par `crates/kesh-db/tests/check_dev_grants_script.rs` (donc par le gate
# nextest et par la CI) ; lançable seul : `bash scripts/tests/check-dev-grants.test.sh`.

set -euo pipefail

SCRIPTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
GARDE_SRC="$SCRIPTS_DIR/check-dev-grants.sh"
TEST_FAST="$SCRIPTS_DIR/test-fast.sh"
[ -f "$TEST_FAST" ] || { echo "test-fast.sh introuvable : $TEST_FAST" >&2; exit 2; }

WORK=$(mktemp -d "${TMPDIR:-/tmp}/check-dev-grants-test.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

# Rien de l'environnement de l'appelant ne doit atteindre le garde.
unset MYSQL_PWD KESH_DOCKER DATABASE_URL

MDP='mdp-distinctif-31-2-Zq7'                     # celui du compose de référence
FAUX='*FAUXCONDENSE0123456789ABCDEF0123456789AB'  # condensé factice des fixtures
CONTENEUR_ID='b449301de25d'

ECHECS=0
CAS=0
fail() { echo "    ✗ $*"; ECHECS=$((ECHECS + 1)); }
ok()   { echo "    ✓ $*"; }

# --- Le docker factice ---------------------------------------------------------
STUB="$WORK/bin/docker-factice"
mkdir -p "$WORK/bin"
cat > "$STUB" <<'FACTICE'
#!/usr/bin/env bash
echo "ARGV $*" >> "$STUB_LOG"
echo "MYSQL_PWD=${MYSQL_PWD-<absente>}" >> "$STUB_LOG"
case "${1:-}" in
  ps)
    if [ "${STUB_PS:-ok}" = echec ]; then
      echo "docker factice : ps en échec" >&2; exit 1
    fi
    [ "${STUB_PS:-ok}" = vide ] || echo "$STUB_CONTENEUR_ID"
    exit 0 ;;
  exec)
    if [ "${STUB_EXEC:-ok}" = echec ]; then
      echo "docker factice : exec en échec" >&2; exit 1
    fi
    cat "$STUB_GRANTS"
    exit 0 ;;
  *)
    echo "docker factice : sous-commande imprévue « $* »" >&2; exit 97 ;;
esac
FACTICE
chmod +x "$STUB"

# --- Les fichiers de référence ------------------------------------------------
# Le fichier de droits de référence exerce l'extraction : commentaires des trois
# formes (dont un `/* … */` sur plusieurs lignes et des GRANT commentés), une
# instruction GRANT sur deux lignes, des instructions qui ne sont pas des GRANT.
ecrire_droits_reference() {
    cat > "$1" <<'SQL'
-- Fichier de droits de RÉFÉRENCE du test (contenu figé).
-- GRANT ALL PRIVILEGES ON *.* TO 'kesh'@'%';   (commenté : ne compte pas)
# GRANT PROCESS ON *.* TO 'kesh'@'%';
/* Bloc sur plusieurs lignes :
   GRANT ALL PRIVILEGES ON `autre`.* TO 'kesh'@'%';
*/
GRANT ALL PRIVILEGES ON `\_sqlx\_test%`.* TO 'kesh'@'%';
GRANT ALL PRIVILEGES
    ON `kesh%`.* TO 'kesh'@'%';   -- sur deux lignes
FLUSH PRIVILEGES;
CREATE DATABASE IF NOT EXISTS kesh_e2e
  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;
SQL
}

ecrire_compose_reference() {
    cat > "$1" <<YAML
services:
  mariadb:
    image: mariadb:10.11
    container_name: kesh-mariadb-dev
    environment:
      MARIADB_ROOT_PASSWORD: racine-jamais-lue
      MARIADB_DATABASE: kesh
      MARIADB_USER: kesh
      MARIADB_PASSWORD: $MDP
YAML
}

# Fixture des droits ÉGAUX : capturée le 2026-10-23 sur le conteneur réel
# (`mariadb -N -B -r -u kesh -e 'SHOW GRANTS FOR CURRENT_USER'`, MariaDB 10.11),
# condensé remplacé par le faux condensé distinctif.
L_USAGE="GRANT USAGE ON *.* TO \`kesh\`@\`%\` IDENTIFIED BY PASSWORD '$FAUX'"
L_KESH='GRANT ALL PRIVILEGES ON `kesh`.* TO `kesh`@`%`'
L_SQLX='GRANT ALL PRIVILEGES ON `\_sqlx\_test%`.* TO `kesh`@`%`'
L_KESHP='GRANT ALL PRIVILEGES ON `kesh%`.* TO `kesh`@`%`'

# --- Outillage des cas ---------------------------------------------------------
# nouveau_cas <titre> : bâtit le mini-dépôt $R et remet l'état du factice à zéro.
nouveau_cas() {
    CAS=$((CAS + 1))
    echo "• cas $CAS — $1"
    R="$WORK/cas$CAS"
    mkdir -p "$R/scripts/mariadb-init"
    cp "$GARDE_SRC" "$R/scripts/check-dev-grants.sh" 2>/dev/null || true
    ecrire_droits_reference "$R/scripts/mariadb-init/01-dev-grants.sql"
    ecrire_compose_reference "$R/docker-compose.dev.yml"
    GRANTS="$R/grants.txt"
    LOG="$R/factice.log"
    : > "$LOG"
    STUB_PS=ok STUB_EXEC=ok
    DOCKER_CMD="$STUB"
}

# droits <ligne>… : la sortie que rendra `docker exec` du factice.
droits() { printf '%s\n' "$@" > "$GRANTS"; }

lancer() {
    set +e
    OUT=$(env -u MYSQL_PWD KESH_DOCKER="$DOCKER_CMD" STUB_LOG="$LOG" STUB_GRANTS="$GRANTS" \
            STUB_PS="$STUB_PS" STUB_EXEC="$STUB_EXEC" STUB_CONTENEUR_ID="$CONTENEUR_ID" \
            bash "$R/scripts/check-dev-grants.sh" 2>&1)
    RC=$?
    set -e
}

expect_rc() {
    if [ "$1" = nonnul ]; then
        if [ "$RC" -ne 0 ]; then ok "code non nul ($RC)"; else fail "code 0, attendu non nul — sortie :"; echo "$OUT" | sed 's/^/      | /'; fi
    elif [ "$RC" -eq "$1" ]; then ok "code $RC"
    else fail "code $RC, attendu $1 — sortie :"; echo "$OUT" | sed 's/^/      | /'
    fi
}
expect_out()    { if grep -qF -- "$1" <<<"$OUT"; then ok "sortie contient « $1 »"; else fail "sortie sans « $1 »"; fi; }
expect_no_out() { if grep -qF -- "$1" <<<"$OUT"; then fail "sortie contient « $1 »"; else ok "sortie sans « $1 »"; fi; }

# Assertions transversales d'un cas en échec : aucune fuite.
sans_fuite() {
    expect_no_out "IDENTIFIED BY PASSWORD"
    expect_no_out "$FAUX"
    expect_no_out "$MDP"
}

# Assertions transversales d'un cas qui atteint `docker exec`.
acquisition_conforme() {
    local argv_exec
    argv_exec=$(grep '^ARGV exec' "$LOG" || true)
    if [ -z "$argv_exec" ]; then fail "aucun appel « exec » consigné"; return; fi
    if grep -qF -- '-N -B -r' <<<"$argv_exec"; then ok "argv exec : -N -B -r"; else fail "argv exec sans « -N -B -r » : $argv_exec"; fi
    if grep -qF -- 'kesh-mariadb-dev' <<<"$argv_exec"; then ok "argv exec : kesh-mariadb-dev"; else fail "argv exec sans le nom du conteneur"; fi
    if grep -qE -- '-e MYSQL_PWD( |$)' <<<"$argv_exec"; then ok "argv exec : -e MYSQL_PWD sans valeur"; else fail "argv exec sans « -e MYSQL_PWD » nu : $argv_exec"; fi
    if grep -qF -- "$MDP" "$LOG" && grep -F -- "$MDP" "$LOG" | grep -q '^ARGV'; then
        fail "le mot de passe figure dans un argv consigné"
    else ok "mot de passe absent de tout argv"
    fi
    local pwd_exec
    pwd_exec=$(grep -A1 '^ARGV exec' "$LOG" | grep '^MYSQL_PWD=' | head -1)
    if [ "$pwd_exec" = "MYSQL_PWD=$MDP" ]; then ok "MYSQL_PWD transmis = mot de passe du compose"
    else fail "MYSQL_PWD vu par exec : « $pwd_exec », attendu celui du compose"; fi
    if grep -qE -- '^ARGV ps --filter name=\^kesh-mariadb-dev\$ --filter status=running -q$' "$LOG"; then
        ok "argv ps : filtre nom exact + en marche"
    else fail "argv ps inattendu : $(grep '^ARGV ps' "$LOG" || echo '<aucun>')"; fi
}

[ -f "$GARDE_SRC" ] || { echo "⛔ garde introuvable : $GARDE_SRC"; echo "⛔ 1 assertion(s) en échec sur 0 cas."; exit 1; }

# --- Les cas -------------------------------------------------------------------
nouveau_cas "droits égaux (fixture capturée) → 0"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc 0
acquisition_conforme
expect_no_out "IDENTIFIED BY PASSWORD"
expect_no_out "$FAUX"

nouveau_cas "droits égaux, ordre différent → 0 (comparaison d'ensembles)"
droits "$L_KESHP" "$L_SQLX" "$L_USAGE" "$L_KESH"
lancer
expect_rc 0
acquisition_conforme

nouveau_cas "relevé avec lignes vides et fins de ligne \\r → 0"
printf '%s\r\n\n%s\r\n   \n%s\r\n%s\r\n\n' "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP" > "$GRANTS"
lancer
expect_rc 0
acquisition_conforme

nouveau_cas "droit en moins (\`kesh\`.* seul) → non nul, étiqueté « droit manquant »"
droits "$L_USAGE" "$L_KESH"
lancer
expect_rc nonnul
expect_out 'droit manquant : GRANT ALL PRIVILEGES ON `kesh%`.* TO `kesh`@`%`'
expect_no_out 'droit en plus :'
acquisition_conforme
sans_fuite

nouveau_cas "droit global en trop : ALL PRIVILEGES ON *.* (remplace USAGE) → non nul"
droits "GRANT ALL PRIVILEGES ON *.* TO \`kesh\`@\`%\` IDENTIFIED BY PASSWORD '$FAUX'" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out 'GRANT ALL PRIVILEGES ON *.* TO `kesh`@`%`'
acquisition_conforme
sans_fuite

nouveau_cas "droit global en trop : PROCESS ON *.* (remplace USAGE) → non nul"
droits "GRANT PROCESS ON *.* TO \`kesh\`@\`%\` IDENTIFIED BY PASSWORD '$FAUX'" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out 'GRANT PROCESS ON *.* TO `kesh`@`%`'
acquisition_conforme
sans_fuite

nouveau_cas "droit superflu hors niveau global (ALL ON autre.*, USAGE intact) → non nul, étiqueté « droit en plus »"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP" 'GRANT ALL PRIVILEGES ON `autre`.* TO `kesh`@`%`'
lancer
expect_rc nonnul
expect_out 'droit en plus : GRANT ALL PRIVILEGES ON `autre`.* TO `kesh`@`%`'
expect_no_out 'droit manquant :'
acquisition_conforme
sans_fuite

nouveau_cas "\`_sqlx_test%\` non échappé → non nul"
droits "$L_USAGE" "$L_KESH" 'GRANT ALL PRIVILEGES ON `_sqlx_test%`.* TO `kesh`@`%`' "$L_KESHP"
lancer
expect_rc nonnul
expect_out '`_sqlx_test%`'
acquisition_conforme
sans_fuite

nouveau_cas "compte kesh@localhost (ligne remplaçante) → non nul"
droits "GRANT USAGE ON *.* TO \`kesh\`@\`localhost\` IDENTIFIED BY PASSWORD '$FAUX'" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out '`kesh`@`localhost`'
acquisition_conforme
sans_fuite

nouveau_cas "USAGE … WITH GRANT OPTION → non nul"
droits "$L_USAGE WITH GRANT OPTION" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out 'WITH GRANT OPTION'
acquisition_conforme
sans_fuite

# C-31-2-16 : un droit `ON *.*` dans le FICHIER remplace la ligne `USAGE` attendue.
nouveau_cas "fichier portant un droit ON *.* (C-31-2-16), relevé conforme → 0"
printf -- "GRANT SELECT ON *.* TO 'kesh'@'%%';\n" >> "$R/scripts/mariadb-init/01-dev-grants.sql"
droits "GRANT SELECT ON *.* TO \`kesh\`@\`%\` IDENTIFIED BY PASSWORD '$FAUX'" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc 0
acquisition_conforme

nouveau_cas "fichier portant un droit ON *.* (C-31-2-16), relevé avec une ligne USAGE en trop → non nul"
printf -- "GRANT SELECT ON *.* TO 'kesh'@'%%';\n" >> "$R/scripts/mariadb-init/01-dev-grants.sql"
droits "GRANT SELECT ON *.* TO \`kesh\`@\`%\` IDENTIFIED BY PASSWORD '$FAUX'" "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out 'droits globaux : attendu GRANT SELECT ON *.* TO `kesh`@`%`'
acquisition_conforme
sans_fuite

nouveau_cas "commentaire # en milieu de ligne dans le fichier de droits → reconnu, 0"
printf -- "FLUSH PRIVILEGES; # GRANT ALL ON *.* TO 'kesh'@'%%' (commentaire de fin de ligne)\n" >> "$R/scripts/mariadb-init/01-dev-grants.sql"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc 0
acquisition_conforme

nouveau_cas "GRANT en minuscules dans le fichier → non nul (forme canonique exigée, faux rouge assumé)"
printf -- "grant all privileges on \`autre\`.* to 'kesh'@'%%';\n" >> "$R/scripts/mariadb-init/01-dev-grants.sql"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP" 'GRANT ALL PRIVILEGES ON `autre`.* TO `kesh`@`%`'
lancer
expect_rc nonnul
expect_out 'droit manquant : grant all privileges on `autre`.*'
acquisition_conforme
sans_fuite

nouveau_cas "secret en clair dans le fichier de droits → jamais affiché"
printf -- "GRANT SELECT ON \`autre\`.* TO 'kesh'@'%%' IDENTIFIED BY 'secret-clair-31-2';\n" >> "$R/scripts/mariadb-init/01-dev-grants.sql"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out 'droit manquant : GRANT SELECT ON `autre`.*'
expect_no_out 'secret-clair-31-2'
sans_fuite

nouveau_cas "instruction refusée portant un secret → refusée, secret masqué"
printf -- "ALTER USER 'kesh'@'%%' IDENTIFIED BY 'secret-clair-31-2' WITH GRANT OPTION;\n" >> "$R/scripts/mariadb-init/01-dev-grants.sql"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out "non comprise"
expect_no_out 'secret-clair-31-2'
sans_fuite

nouveau_cas "lancé sous bash -x : ni le mot de passe ni le condensé dans la trace"
droits "$L_USAGE" "$L_KESH"
set +e
OUT=$(env -u MYSQL_PWD KESH_DOCKER="$DOCKER_CMD" STUB_LOG="$LOG" STUB_GRANTS="$GRANTS" \
        STUB_PS="$STUB_PS" STUB_EXEC="$STUB_EXEC" STUB_CONTENEUR_ID="$CONTENEUR_ID" \
        bash -x "$R/scripts/check-dev-grants.sh" 2>&1)
RC=$?
set -e
expect_rc nonnul
sans_fuite

nouveau_cas "docker introuvable (KESH_DOCKER vers un chemin inexistant) → garde sauté, 0"
DOCKER_CMD="$WORK/inexistant/docker"
lancer
expect_rc 0
expect_out "garde des droits sauté : docker introuvable"
MSG_SANS_DOCKER="$OUT"

nouveau_cas "conteneur non en marche (ps vide) → garde sauté, 0, message distinct"
STUB_PS=vide
lancer
expect_rc 0
expect_out "garde des droits sauté : conteneur kesh-mariadb-dev non en marche"
if [ "$OUT" != "$MSG_SANS_DOCKER" ]; then ok "message distinct de « docker introuvable »"
else fail "même message que « docker introuvable »"; fi
if grep -q '^ARGV exec' "$LOG"; then fail "exec appelé alors que le conteneur n'est pas en marche"; else ok "aucun exec"; fi

nouveau_cas "docker ps en échec → non nul, cause nommée"
STUB_PS=echec
lancer
expect_rc nonnul
expect_out "docker ps a échoué"
expect_no_out "garde des droits sauté"
sans_fuite

nouveau_cas "docker exec en échec → non nul, cause nommée"
STUB_EXEC=echec
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out "docker exec a échoué"
expect_no_out "garde des droits sauté"
acquisition_conforme
sans_fuite

nouveau_cas "fichier de droits sans aucun GRANT → non nul"
printf -- "-- GRANT ALL ON *.* TO 'kesh'@'%%';\nFLUSH PRIVILEGES;\n" > "$R/scripts/mariadb-init/01-dev-grants.sql"
droits "GRANT USAGE ON *.* TO \`kesh\`@\`%\` IDENTIFIED BY PASSWORD '$FAUX'" "$L_KESH"
lancer
expect_rc nonnul
expect_out "aucune instruction GRANT"
sans_fuite

nouveau_cas "fichier de droits absent → non nul"
rm "$R/scripts/mariadb-init/01-dev-grants.sql"
droits "$L_USAGE" "$L_KESH"
lancer
expect_rc nonnul
expect_out "fichier de droits introuvable"
sans_fuite

nouveau_cas "instruction qui contient GRANT sans commencer par lui → non nul"
printf -- "REVOKE GRANT OPTION ON *.* FROM 'kesh'@'%%';\n" >> "$R/scripts/mariadb-init/01-dev-grants.sql"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out "REVOKE GRANT OPTION"
sans_fuite

nouveau_cas "MARIADB_USER interpolé → non nul, clé nommée"
sed -i 's/^      MARIADB_USER: kesh$/      MARIADB_USER: ${KESH_DB_USER}/' "$R/docker-compose.dev.yml"
grep -qF 'MARIADB_USER: ${KESH_DB_USER}' "$R/docker-compose.dev.yml" || fail "montage du cas : substitution non faite"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out "MARIADB_USER"
if grep -q '^ARGV exec' "$LOG"; then fail "exec appelé avec un compte interpolé"; else ok "aucun exec"; fi
sans_fuite

nouveau_cas "MARIADB_USER absent → non nul, clé nommée"
sed -i '/^      MARIADB_USER:/d' "$R/docker-compose.dev.yml"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out "clé MARIADB_USER : 0 occurrence"
sans_fuite

nouveau_cas "MARIADB_USER en double → non nul, occurrences comptées"
printf '      MARIADB_USER: autre\n' >> "$R/docker-compose.dev.yml"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out "clé MARIADB_USER : 2 occurrence"
sans_fuite

nouveau_cas "MARIADB_DATABASE vide → non nul, clé nommée"
sed -i 's/^      MARIADB_DATABASE: kesh$/      MARIADB_DATABASE:/' "$R/docker-compose.dev.yml"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out "clé MARIADB_DATABASE vide"
sans_fuite

nouveau_cas "compose de dev absent → non nul, fichier nommé"
rm "$R/docker-compose.dev.yml"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc nonnul
expect_out "compose de dev introuvable"

nouveau_cas "mot de passe entre guillemets suivi d'un commentaire → valeur nue transmise, 0"
sed -i "s/^      MARIADB_PASSWORD: .*/      MARIADB_PASSWORD: \"$MDP\"   # mot de passe de dev/" "$R/docker-compose.dev.yml"
grep -qF "MARIADB_PASSWORD: \"$MDP\"   #" "$R/docker-compose.dev.yml" || fail "montage du cas : substitution non faite"
droits "$L_USAGE" "$L_KESH" "$L_SQLX" "$L_KESHP"
lancer
expect_rc 0
acquisition_conforme

nouveau_cas "message d'échec : remise en état en texte arrêté, jamais exécutée"
droits "$L_USAGE" "$L_KESH"
lancer
expect_rc nonnul
acquisition_conforme
sans_fuite
expect_out "attendre l'état healthy"
expect_out "depuis le dépôt principal"
expect_out "docker compose -f docker-compose.dev.yml up -d --force-recreate mariadb"
expect_out "efface les bases de tous les agents"
if grep -qE '^ARGV (compose|restart|stop|start|rm)' "$LOG"; then fail "le garde a lancé une remise en état"; else ok "aucune remise en état lancée"; fi

# --- Câblage dans test-fast.sh (lit le VRAI fichier) -------------------------
CAS=$((CAS + 1))
echo "• cas $CAS — câblage : une seule invocation non commentée, nue, au premier niveau, entre cargo-nextest et le bloc lint"
INVOC=$(grep -nE '^[[:space:]]*[^#[:space:]].*check-dev-grants\.sh' "$TEST_FAST" || true)
N_INVOC=$(printf '%s' "$INVOC" | grep -c . || true)
# Bornes : sur leurs seules lignes de CODE (non commentées) ; une borne disparue
# se dit, au lieu d'arrêter le test sans diagnostic (`set -e`).
L_NEXTEST=$(grep -nE '^[^#]*command -v cargo-nextest' "$TEST_FAST" | head -1 | cut -d: -f1 || true)
L_LINT=$(grep -nE '^[^#]*if \[ "\$RUN_LINT" = 1 \]' "$TEST_FAST" | head -1 | cut -d: -f1 || true)
[ -n "$L_NEXTEST" ] || fail "borne introuvable dans test-fast.sh : « command -v cargo-nextest »"
[ -n "$L_LINT" ] || fail "borne introuvable dans test-fast.sh : « if [ \"\$RUN_LINT\" = 1 ] »"
if [ "$N_INVOC" = 1 ]; then ok "une seule ligne d'invocation"; else fail "$N_INVOC ligne(s) d'invocation non commentée(s) : $INVOC"; fi
if [ "$N_INVOC" = 1 ] && [ -n "$L_NEXTEST" ] && [ -n "$L_LINT" ]; then
    L_INVOC=${INVOC%%:*}
    if [ "$L_INVOC" -gt "$L_NEXTEST" ] && [ "$L_INVOC" -lt "$L_LINT" ]; then
        ok "ligne $L_INVOC entre cargo-nextest ($L_NEXTEST) et le bloc lint ($L_LINT)"
    else fail "ligne $L_INVOC hors de ]$L_NEXTEST ; $L_LINT["; fi
    # Nue ET au premier niveau (colonne 0) : dans `test-fast.sh`, tout code
    # conditionnel est indenté. Angle mort assumé : un `if false; then` lui-même
    # non indenté juste au-dessus passerait — le `set -e` du script et la revue
    # restent le filet.
    if grep -qE '^(bash )?(\./)?scripts/check-dev-grants\.sh[[:space:]]*$' <<<"${INVOC#*:}"; then
        ok "invocation nue, au premier niveau"
    else fail "invocation non nue ou indentée : ${INVOC#*:}"; fi
fi
if grep -qE 'echo.*check-dev-grants' "$TEST_FAST"; then fail "un echo de test-fast.sh nomme le fichier du garde"; else ok "aucun echo ne nomme le fichier"; fi

echo
if [ "$ECHECS" -gt 0 ]; then
    echo "⛔ $ECHECS assertion(s) en échec sur $CAS cas."
    exit 1
fi
echo "✅ $CAS cas, aucune assertion en échec."
