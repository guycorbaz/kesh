#!/usr/bin/env bash
# check-dev-grants.sh — garde des droits du MariaDB de DÉVELOPPEMENT.
#
# Compare l'ENSEMBLE des droits que l'utilisateur applicatif a réellement sur le
# conteneur `kesh-mariadb-dev` à celui que décrit le fichier versionné
# `scripts/mariadb-init/01-dev-grants.sql`. Rend 0 s'ils sont égaux, non nul
# sinon, en nommant chaque écart et la remise en état — qu'il n'exécute JAMAIS.
#
# Pourquoi : du 2026-10-11 au 2026-10-23, le conteneur a monté un répertoire
# vide (le worktree d'où il avait été créé n'existait plus) ; `kesh` n'avait
# plus que `kesh`.*, et des `GRANT` posés à la main par des agents ont masqué la
# panne — au point de rendre vertes des sondes qui exigeaient un privilège que
# la production n'a pas (Stories 29-4, 29-8). Rien ne signalait l'écart.
# *(Story 31-2, action C3 de la rétrospective de l'Epic 29.)*
#
# Lancé par `scripts/test-fast.sh` avant toute compilation ; utilisable seul :
#   scripts/check-dev-grants.sh
#
# Sources, lues RELATIVEMENT À L'EMPLACEMENT DE CE SCRIPT (`$(dirname "$0")/..`) :
#   - `scripts/mariadb-init/01-dev-grants.sql` : ses instructions `GRANT` ;
#   - `docker-compose.dev.yml` : `MARIADB_USER` (seule source du nom de compte),
#     `MARIADB_PASSWORD` et `MARIADB_DATABASE`.
# Seule injection : `KESH_DOCKER` (la commande `docker`, défaut `docker`), par
# laquelle le test passe un `docker` factice.
#
# Codes : 0 = droits égaux, ou garde SAUTÉ (docker introuvable, conteneur non en
# marche — chacun dit) ; 1 = écart ou acquisition impossible (cause nommée).
#
# Normalisation, minimale et explicite (rien d'autre) :
#   (a) le compte `'<MARIADB_USER>'@'%'` du fichier devient `` `<MARIADB_USER>`@`%` `` ;
#   (b) le `;` final des instructions du fichier est retiré (découpage) ;
#   (c) sur toute ligne `… ON *.* TO …`, réelle ou attendue, la clause
#       ` IDENTIFIED BY PASSWORD '<condensé>'` est retirée — la sortie du garde ne
#       contient donc jamais le condensé. Les antislashs sont CONSERVÉS : les
#       retirer rendrait `_sqlx_test%` non échappé (un joker) indiscernable du bon
#       droit `\_sqlx\_test%`.
# L'attendu est : les `GRANT` du fichier, plus `ALL PRIVILEGES` sur la base
# `MARIADB_DATABASE` et la ligne `USAGE ON *.*` que crée l'entrypoint — sauf si
# le fichier porte lui-même un droit `ON *.*`, qui sur MariaDB REMPLACE la ligne
# `USAGE` au lieu de s'y ajouter.
#
# ⚠️ TROIS ANGLES MORTS, écrits pour qu'on ne lise pas ce garde comme une
# garantie plus forte que la sienne :
#   (i)   les droits du rôle `PUBLIC` n'apparaissent pas dans `SHOW GRANTS` d'un
#         utilisateur ;
#   (ii)  l'acquisition passe par le socket du conteneur (`kesh@localhost`
#         s'il existait, `kesh@%` sinon), le gate par TCP (`kesh@%`) : un compte
#         `'kesh'@'172.%'` posé à la main resterait invisible ;
#   (iii) la règle « le conteneur se crée DEPUIS LE DÉPÔT PRINCIPAL » n'est PAS
#         vérifiée : un conteneur monté depuis un worktree encore présent, au
#         fichier identique, passe ; le garde rougit quand l'égalité des droits
#         est rompue (typiquement après suppression du worktree et redémarrage),
#         non au moment de la violation. Prévention : issue #699.
#
# Limites de l'extraction (revue de code P1, écrites plutôt que traitées) : le
# fichier de droits doit être écrit à la FORME CANONIQUE que rend le serveur
# (`GRANT ALL PRIVILEGES ON \`base\`.* TO 'user'@'%'`, mot-clé et privilèges en
# majuscules) — `grant all on …` y donnerait un faux rouge, bruyant ; un `--`,
# `#` ou `/*` À L'INTÉRIEUR d'une chaîne ou d'un identifiant fait REFUSER le
# fichier (bruyant) s'il est détecté sur sa ligne — une chaîne ouverte sur une
# ligne précédente échappe à ce contrôle ; les commentaires exécutables
# `/*! … */` sont ignorés comme des commentaires.
#
# Affichage : toute ligne affichée — droit relevé, attendu, instruction refusée
# — passe par `masquer`, FAIL-CLOSED : à partir du premier `identified` ou
# `using`, quelle qu'en soit la casse, rien n'est affiché (`<masqué>`), quels que
# soient les guillemets. Le condensé de la ligne globale en est retiré avant.
# `set +x` en tête : un `bash -x` ou un `SHELLOPTS=xtrace` hérité ne trace pas
# l'export du mot de passe.
#
# Remise en état (affichée, jamais exécutée — cf. docs/testing.md § « Base de
# dev jetable » et docs/agents-orchestration.md § 2) : attendre `healthy` si le
# conteneur vient de démarrer ; sinon recréation DEPUIS LE DÉPÔT PRINCIPAL, qui
# efface les bases de tous les agents — à demander à l'orchestrateur.
set +x
set -euo pipefail
export LC_ALL=C

RACINE="$(cd "$(dirname "$0")/.." && pwd)"
FICHIER_DROITS="$RACINE/scripts/mariadb-init/01-dev-grants.sql"
COMPOSE="$RACINE/docker-compose.dev.yml"
CONTENEUR="kesh-mariadb-dev"
DOCKER="${KESH_DOCKER:-docker}"

echec() { echo "⛔ garde des droits : $*" >&2; }

remise_en_etat() {
    cat >&2 <<'TEXTE'
   Remise en état (le garde ne l'exécute pas) :
   Si le conteneur vient de démarrer, attendre l'état healthy (docker ps) et relancer.
   Sinon, depuis le dépôt principal (jamais un worktree) :
     docker compose -f docker-compose.dev.yml up -d --force-recreate mariadb
   puis sqlx migrate run --source crates/kesh-db/migrations et scripts/seed-dev-db.sql.
   ⚠️ Cela efface les bases de tous les agents (tmpfs) : à demander à l'orchestrateur,
   jamais d'initiative ; un simple restart ne répare pas un montage périmé.
TEXTE
}

# --- (1) docker présent ? ------------------------------------------------------
if ! command -v "$DOCKER" >/dev/null 2>&1; then
    echo "garde des droits sauté : docker introuvable ($DOCKER)"
    exit 0
fi

# --- (2) conteneur en marche ? ------------------------------------------------
# L'état se lit au code de retour et à la sortie de `docker ps`, jamais au texte
# d'une erreur : un démon arrêté n'est pas un conteneur absent.
rc=0
en_marche=$("$DOCKER" ps --filter "name=^${CONTENEUR}\$" --filter status=running -q) || rc=$?
if [ "$rc" -ne 0 ]; then
    echec "docker ps a échoué (code $rc) : démon arrêté, socket inaccessible ou utilisateur hors du groupe docker ?"
    exit 1
fi
if [ -z "$en_marche" ]; then
    echo "garde des droits sauté : conteneur $CONTENEUR non en marche"
    exit 0
fi

# --- (3) sources : compose et fichier de droits -------------------------------
# cle_compose <clé> : valeur de `<clé>:` dans le compose de dev (une seule
# occurrence admise) ; refuse une clé absente, vide ou interpolée.
cle_compose() {
    local cle=$1 lignes n valeur
    [ -f "$COMPOSE" ] || { echec "compose de dev introuvable : $COMPOSE"; exit 1; }
    lignes=$(grep -E "^[[:space:]]*${cle}:" "$COMPOSE" || true)
    n=$(printf '%s' "$lignes" | grep -c . || true)
    if [ "$n" -ne 1 ]; then
        echec "clé $cle : $n occurrence(s) dans docker-compose.dev.yml, une seule attendue"
        exit 1
    fi
    valeur=$(printf '%s\n' "$lignes" | sed -E "s/^[[:space:]]*${cle}:[[:space:]]*//; s/[[:space:]]+#.*$//; s/[[:space:]]*$//")
    valeur=${valeur#\"}; valeur=${valeur%\"}; valeur=${valeur#\'}; valeur=${valeur%\'}
    if [ -z "$valeur" ]; then
        echec "clé $cle vide dans docker-compose.dev.yml"
        exit 1
    fi
    case "$valeur" in
        *'$'*) echec "clé $cle interpolée dans docker-compose.dev.yml (\${…}) : valeur non lisible par le garde"; exit 1 ;;
    esac
    printf '%s' "$valeur"
}

UTILISATEUR=$(cle_compose MARIADB_USER)
MOT_DE_PASSE=$(cle_compose MARIADB_PASSWORD)
BASE=$(cle_compose MARIADB_DATABASE)

if [ ! -f "$FICHIER_DROITS" ]; then
    echec "fichier de droits introuvable : $FICHIER_DROITS (01-dev-grants.sql)"
    exit 1
fi

# (c) — retrait de la clause du condensé sur toute ligne de niveau global
# (comparaison seulement).
sans_condense() { sed -E "/ ON \\*\\.\\* TO /s/ IDENTIFIED BY PASSWORD '[^']*'//"; }
# `masquer` (affichage) — FAIL-CLOSED (revue de code P2) : toute ligne qui
# contient `identified` ou `using`, quelle qu'en soit la casse, est tronquée à ce
# mot-clé et suivie de `<masqué>` ; rien de ce qui suit n'est affiché, quels que
# soient les guillemets. Rien n'est affiché sans passer par ici. Effet assumé :
# une base dont le nom contiendrait l'un de ces mots serait tronquée elle aussi.
masquer() {
    awk '{
        l = tolower($0); i = index(l, "identified"); u = index(l, "using")
        if (u > 0 && (i == 0 || u < i)) i = u
        if (i > 0) print substr($0, 1, i - 1) "<masqué>"; else print
    }'
}

# Extraction des GRANT du fichier : commentaires retirés (`-- …` et `# …`
# jusqu'en fin de ligne, `/* … */` y compris sur plusieurs lignes), découpage en
# instructions par `;`, blancs ramenés à une espace. L'extraction ne comprend
# pas tout le SQL ; ce qu'elle ne comprend pas, elle le REFUSE, bruyamment,
# au lieu de l'ignorer :
#   - code 3 (`REFUS<tab><instr>`) : une instruction contient GRANT sans
#     commencer par lui ;
#   - code 4 (`CHAINE<tab><ligne>`) : un marqueur de commentaire (`#`, `--`,
#     `/*`) suit, sur sa ligne, un nombre IMPAIR d'apostrophes ou de backticks —
#     il est donc dans une chaîne ou un identifiant, et le prendre pour un
#     commentaire tronquerait l'instruction (revue de code P2). Ce contrôle ne
#     voit qu'une ligne : une chaîne ouverte sur une ligne précédente lui échappe.
# awk POSIX (la CI a mawk, le poste gawk).
rc=0
grants_fichier=$(awk '
    function impair(t, c,   n) { n = gsub(c, "", t); return n % 2 }
    BEGIN { dans_bloc = 0; tampon = ""; chaine = "" }
    {
        ligne = $0; sortie = ""
        while (length(ligne) > 0) {
            if (dans_bloc) {
                i = index(ligne, "*/")
                if (i == 0) { ligne = ""; break }
                ligne = substr(ligne, i + 2); dans_bloc = 0; continue
            }
            b = index(ligne, "/*")
            d = match(ligne, /--([ \t]|$)/)
            h = index(ligne, "#")
            # Fin de ligne commentée : le plus tôt de `--` et `#`.
            f = d
            if (h > 0 && (f == 0 || h < f)) f = h
            # Le marqueur retenu (bloc ou fin de ligne) est-il dans une chaîne ?
            m = f
            if (b > 0 && (m == 0 || b < m)) m = b
            if (m > 0) {
                avant = sortie substr(ligne, 1, m - 1)
                if (impair(avant, "\047") || impair(avant, "`")) { chaine = $0; exit 4 }
            }
            if (b > 0 && (f == 0 || b < f)) {
                sortie = sortie substr(ligne, 1, b - 1) " "
                ligne = substr(ligne, b + 2); dans_bloc = 1; continue
            }
            if (f > 0) { sortie = sortie substr(ligne, 1, f - 1); ligne = ""; break }
            sortie = sortie ligne; ligne = ""
        }
        tampon = tampon " " sortie
    }
    END {
        if (chaine != "") { print "CHAINE\t" chaine; exit 4 }
        n = split(tampon, instr, ";")
        for (k = 1; k <= n; k++) {
            s = instr[k]
            gsub(/[ \t\r]+/, " ", s); sub(/^ /, "", s); sub(/ $/, "", s)
            if (s == "") continue
            if (toupper(substr(s, 1, 6)) == "GRANT ") { print s; continue }
            if (index(toupper(s), "GRANT") > 0) { print "REFUS\t" s; exit 3 }
        }
    }
' "$FICHIER_DROITS") || rc=$?
if [ "$rc" -eq 3 ]; then
    refusee=$(printf '%s\n' "$grants_fichier" | sed -n 's/^REFUS\t//p' | masquer)
    echec "instruction de 01-dev-grants.sql non comprise (contient GRANT sans commencer par lui) : $refusee"
    exit 1
fi
if [ "$rc" -eq 4 ]; then
    refusee=$(printf '%s\n' "$grants_fichier" | sed -n 's/^CHAINE\t//p' | masquer)
    echec "ligne de 01-dev-grants.sql non comprise (marqueur de commentaire dans une chaîne ou un identifiant) : $refusee"
    exit 1
fi
if [ "$rc" -ne 0 ]; then
    echec "extraction des GRANT de 01-dev-grants.sql impossible (code $rc)"
    exit 1
fi
if [ -z "$grants_fichier" ]; then
    echec "01-dev-grants.sql ne porte aucune instruction GRANT : attendu refusé (un fichier vide ou tout commenté décrirait l'état défaillant)"
    exit 1
fi

# (a) — compte du fichier à la forme du serveur.
attendu=$(printf '%s\n' "$grants_fichier" \
    | sed "s/'${UTILISATEUR}'@'%'/\`${UTILISATEUR}\`@\`%\`/g")
attendu+=$'\n'"GRANT ALL PRIVILEGES ON \`${BASE}\`.* TO \`${UTILISATEUR}\`@\`%\`"
if ! grep -qF ' ON *.* TO ' <<<"$attendu"; then
    attendu+=$'\n'"GRANT USAGE ON *.* TO \`${UTILISATEUR}\`@\`%\`"
fi
attendu=$(printf '%s\n' "$attendu" | sans_condense | sort -u)

# --- (4) acquisition ---------------------------------------------------------
# Le mot de passe est EXPORTÉ, puis transmis par son seul nom (`-e MYSQL_PWD`) :
# ni dans l'argv de docker, ni dans celle du client — donc absent de `ps`.
export MYSQL_PWD="$MOT_DE_PASSE"
rc=0
releve=$("$DOCKER" exec -e MYSQL_PWD "$CONTENEUR" \
    mariadb -N -B -r -u "$UTILISATEUR" -e 'SHOW GRANTS FOR CURRENT_USER') || rc=$?
unset MYSQL_PWD
if [ "$rc" -ne 0 ]; then
    echec "docker exec a échoué (code $rc) : authentification de $UTILISATEUR refusée, ou client mariadb absent du conteneur ?"
    remise_en_etat
    exit 1
fi
# `\r` et lignes vides retirés : ils feraient diverger les ensembles à tort.
releve=$(printf '%s\n' "$releve" | tr -d '\r' | sans_condense | grep -v '^[[:space:]]*$' | sort -u || true)

# --- (5) comparaison d'ensembles ---------------------------------------------
if [ "$attendu" = "$releve" ]; then
    echo "garde des droits : droits de $UTILISATEUR conformes à 01-dev-grants.sql"
    exit 0
fi

global_attendu=$(grep -F ' ON *.* TO ' <<<"$attendu" || true)
global_releve=$(grep -F ' ON *.* TO ' <<<"$releve" || true)
echec "les droits de $UTILISATEUR sur $CONTENEUR ne sont pas ceux de scripts/mariadb-init/01-dev-grants.sql"
if [ "$global_attendu" != "$global_releve" ]; then
    echo "   droits globaux : attendu ${global_attendu:-<aucun>}, relevé ${global_releve:-<aucun>}" | masquer >&2
fi
comm -23 <(grep -vF ' ON *.* TO ' <<<"$attendu" || true) <(grep -vF ' ON *.* TO ' <<<"$releve" || true) \
    | masquer | sed 's/^/   droit manquant : /' >&2
comm -13 <(grep -vF ' ON *.* TO ' <<<"$attendu" || true) <(grep -vF ' ON *.* TO ' <<<"$releve" || true) \
    | masquer | sed 's/^/   droit en plus : /' >&2
remise_en_etat
exit 1
