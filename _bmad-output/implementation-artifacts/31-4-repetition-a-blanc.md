# Story 31.4 : La répétition à blanc d'une clôture (ex-29-7, documentation exécutée)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie. Registre : `epic-31-choix-autonomes-314.md`. Aucune issue GitHub ne porte
cette story (la 29-7 est née en validation P2 de la 29-1 ; son sort a été tranché par Guy à la rétrospective de l'Epic
29, Q1 : reportée **après #503**, avant la clôture de l'exercice 2026). Cahier des charges d'origine : commentaire de la
clé `29-7-repetition-a-blanc` de `sprint-status.yaml`, que cette fiche reprend **sans l'affaiblir** et complète de ce que
la 31-1 a changé.*

## Story

En tant que Guy, qui tient désormais ses livres réels dans Kesh et clôturera l'exercice 2026 sur le NAS, je veux une
**procédure écrite au manuel d'administration** pour **répéter la clôture sur une copie isolée** de mon installation —
restaurée depuis une sauvegarde complète, hors du NAS —, **telle qu'elle a été réellement jouée sur la station**, afin de
découvrir sur la copie, et non sur mes livres, ce qui coince : la restauration, les contrôles préalables, la
contre-passation des transitoires, la clôture, la réouverture.

## Origine de la décision

- **29-1, validation P1/P2** (`29-1-preparer-cloture.md`, ancienne AC19) : la répétition à blanc était écrite à la
  relecture ; chaque correction de sa marche a produit le défaut suivant, dont **R1 HIGH : la procédure pouvait remplacer
  la base de production**. Retirée de la 29-1b, devenue la 29-7 (C-29-1-23), « à écrire **en l'exécutant réellement** ».
- **Rétrospective de l'Epic 29**, § 4.1 (l'incident) et § 8/§ 10, Q1 : reportée **après #503**, parce que la répétition
  importe une sauvegarde et doit éprouver **la sauvegarde complète**, non l'ancienne. Décision de Guy du 2026-10-23.
- **Fichier d'epic** (`epic-31-tenir-les-livres.md`, § 31-4) : cahier des charges inchangé, « exécutée sur une copie avec
  une sauvegarde synthétique complète ». **Dépendance satisfaite** : la sauvegarde complète (31-1a, `a3e4bd62`) et la
  restauration depuis l'assistant (31-1b, `e71cd351`) sont sur `main`.
- **Indépendance de la 31-1c** (sauvegarde automatique, en développement en parallèle) : la répétition prend **un fichier
  `.keshbackup`**, quelle que soit son origine (export manuel ou, plus tard, sauvegarde automatique : même format 2). Elle
  **ne dépend pas** de la 31-1c et n'en parle pas ; si la 31-1c est mergée avant le développement, le dev peut ajouter
  une seule phrase au manuel (« ce fichier peut aussi être une sauvegarde automatique »), sous réserve de l'avoir vue
  fonctionner (C-31-4-9).

## État des lieux (établi au code et à la station le 2026-10-24)

| Fait | Constat |
|---|---|
| Compose autonome | `docker-compose.yml` : services `mariadb` (`mariadb:10.11`) **et** `kesh-api`. `DATABASE_URL` y est **construit** : `mysql://${MARIADB_USER:-kesh}:${MARIADB_PASSWORD:?…}@mariadb:3306/${MARIADB_DATABASE:-kesh}` — le `DATABASE_URL` du `.env` **n'y est pas lu**. Relevé par `docker compose config` sur le fichier du tag `v0.14.0` (voir ci-dessous) |
| Compose Synology | `docker-compose.prod.yml` : **un seul service** (`kesh-api`), `DATABASE_URL: ${DATABASE_URL}` **lu dans le `.env`**, réseau externe `frontend`, montages `./log`, `./inbox`, `./documents`, `./backup`. Le manuel Synology l'enregistre sous le nom **`docker-compose.yml`** (`admin-manual.tex` l. 574 : `curl … "$DEPOT/docker-compose.prod.yml" -o docker-compose.yml`) : **le nom du fichier ne dit pas lequel c'est** — R1 |
| Montages absolus (R2) | `docker-compose.yml` l. 76-78 : `${KESH_LOG_HOST_DIR:-./log}`, `${KESH_INBOX_HOST_DIR:-./inbox}`, `${KESH_DOCUMENTS_HOST_DIR:-./documents}`, `./backup`. Posées (dans le `.env` **ou dans l'environnement du shell**, qui l'emporte sur le `.env`), ces variables rendent les montages absolus : un *Importer* dans la copie viderait/réécrirait le dossier **de la production** |
| Noms fixes | `container_name: kesh-mariadb` et `kesh-api` en dur dans `docker-compose.yml` ; réseau `kesh-network`, volume `kesh-mariadb-data` (préfixés par le **nom du projet** Compose = nom du dossier). Sur un hôte qui porte déjà la production, une seconde pile **entre en collision** (refus de Compose, dans le meilleur cas) |
| Port | `docker-compose.yml` publie `"80:80"` (toutes interfaces) ; `KESH_PORT` change le port **dans** le conteneur, pas la publication. Station au 2026-10-24 : port 80 libre (`ss -ltn`), 3306 pris par `kesh-mariadb-dev` (127.0.0.1) |
| Compose au tag | `curl -fsSL https://raw.githubusercontent.com/guycorbaz/kesh/v0.14.0/docker-compose.yml` → 200. **Écart avec `main`** (`git diff v0.14.0 HEAD -- docker-compose*.yml .env.example`) : quatre lignes de **commentaires**, plus `Dockerfile` (1.96 → 1.97) ; aucune ligne exécutable du compose n'a changé |
| Essai à blanc de `docker compose config` | sur le compose et le `.env.example` du tag `v0.14.0`, avec `MARIADB_ROOT_PASSWORD`, `MARIADB_PASSWORD`, `KESH_JWT_SECRET` posés : `DATABASE_URL: mysql://kesh:…@mariadb:3306/kesh` ✔ ; deux services `mariadb`, `kesh-api` ✔ ; montages `source:` résolus **sous le dossier courant** ✔ ; un `docker-compose.override.yml` portant `ports: !override [ "127.0.0.1:18314:80" ]` et `image:` est pris en compte (`host_ip: 127.0.0.1`, `published: "18314"`) ✔ (Compose 2.40.3) |
| `.env.example` | lignes **vides de valeur utile** : `DATABASE_URL=<EDIT: …>` (ignorée par `docker-compose.yml`, exigée par le Synology), `KESH_JWT_SECRET=<GENERATE_ME: …>` (**refusée au démarrage** depuis la 0.13.0 tant qu'elle n'est pas remplacée), `MARIADB_ROOT_PASSWORD` / `MARIADB_PASSWORD` **commentées** (Compose refuse de démarrer sans) ; `KESH_COOKIE_SECURE=true` (cookie `Secure` refusé sur HTTP : la copie, en `http://127.0.0.1`, pose `false`) |
| Manuel : existant | `admin-manual.tex` § *Sauvegarde et restauration* : `Test de restauration périodique` (l. 1598, OLICo art. 10 al. 1 : « tester effectivement la restauration … sur un environnement de test isolé », sans montage ni garde) ; `Export/import d'installation via l'interface Kesh` (l. 1851) ; `Restaurer sur une nouvelle machine` (l. 2013, livrée par la 31-1b : la marche *installation neuve → /setup → « Restaurer une sauvegarde » → import → vérifier*). **La répétition la réutilise** au lieu de la recopier (DRY) |
| Manuel : gestes de clôture | `user-manual.tex` : `Préparer la clôture` (`sec:preparer-cloture`, l. 792), `Les transitoires de fin d'exercice` (`sec:transitoires`, l. 685 ; bouton « Contre-passer au début de l'exercice suivant »), `Clôture d'un exercice` (l. 819), `Réouverture d'un exercice clôturé` (`sec:reouverture-exercice`, l. 843) |
| Parcours déjà automatisé | `frontend/tests/e2e/fiscal-year-closing-journey.spec.ts` (29-1b) joue : exercices 2025 et 2026, comptes `1300`/`1020`, compte bancaire lié, relevé CAMT `camt053_v04_year_end.xml`, préparation (K2 et K3 en attention), rapprochement manuel, régularisation de 250.00 au 31.12.2025, contre-passation, clôture, réouverture. **La répétition en est la version « copie du NAS »**, pas un nouveau test fonctionnel |
| Garde G19 | `crates/kesh-api/tests/textes_coherents.rs` l. 1188 : toute URL `raw.githubusercontent.com/guycorbaz/kesh/<réf>/…` des `.tex`, du README et de `website/` doit porter `v<\keshVersion>` (au 2026-10-24 : **`v0.14.0`**, `kesh-style.sty` l. 64). Au bump de release (0.15.0), elle rougit tant que les URL n'ont pas suivi — **ce qui est voulu** et suffit à tenir l'URL de la répétition |
| Droits de la station | `sudo -n true` → *interactive authentication is required* : **un agent n'a pas `sudo`**. Les dossiers `log/`, `inbox/`, `documents/`, `backup/` d'une copie sont créés **par Docker, en root** (le conteneur tourne en root) : leur suppression est le point où la procédure doit offrir un chemin **sans `sudo`** (C-31-4-5) |
| Station, images | `gcorbaz/kesh:0.13.2`, `0.13.3` et d'autres anciennes présentes ; **aucune image portant le format 2** (la 0.15.0 n'est pas publiée). `docker build` possible depuis `main` (`Dockerfile`, Rust 1.97) |
| Santé de la version | `GET /health` → `{"status":"ok","db":true,"version":"…"}` (`crates/kesh-api/src/routes/health.rs` l. 38-41) : c'est **là** que « la version relevée » se lit |

## Prémisses de l'issue / du cahier des charges : établies, corrigées, ajoutées

1. **R1 (HIGH) — établie.** Le danger est réel **pour le compose Synology** : sa `DATABASE_URL` vient du `.env`, et un
   `.env` recopié de la production fait viser à la « copie » la base de la production. Le compose *autonome* est, lui,
   **sans ce risque** (il construit sa `DATABASE_URL` vers `@mariadb:3306`). **Précision** : la procédure ne peut donc pas
   se contenter de « prendre le compose autonome » — elle doit **contrôler** ce qu'elle a obtenu, parce que le fichier
   porte le même nom dans les deux installations. Le contrôle est `docker compose config`, **avant tout `up`** (AC3).
2. **R2 — établie, avec un cas que le cahier des charges ne nomme pas** : les variables `KESH_*_HOST_DIR` peuvent venir
   **de l'environnement du shell** et non du `.env` ; l'absence de ces lignes au `.env` de la copie ne suffit donc pas.
   Le contrôle est le même : la sortie de `docker compose config` ne doit montrer **aucun** `source:` hors du dossier
   de la copie (AC3), et `env | grep '^KESH_'` ne doit rien rendre (AC3).
3. **F5 — établie.** Les dossiers créés par Docker appartiennent à root. Le cahier des charges ne prévoit que `sudo` ;
   **la station d'un agent n'en a pas** (relevé ci-dessus), et le manuel s'adresse aussi à des lecteurs sans `sudo` : la
   procédure donne les **deux** chemins, dont l'un par un conteneur jetable (C-31-4-5), **vérifié** au développement.
4. **F L9 — établie.** Le message du compose « reprendre la valeur avec laquelle la base a été créée, jamais une neuve »
   vaut pour une mise à jour ; **dans la copie, volume neuf, une valeur neuve de `MARIADB_PASSWORD` est la bonne**. Le
   manuel le dit, avec la raison, pour qu'on ne recopie pas le mot de passe de la production par scrupule.
5. **G19 — établie** (ci-dessus). L'URL est **écrite en dur** dans un `lstlisting` (la macro ne s'y étend pas) ; la garde la
   tient égale à la macro.
6. **Ajoutée — la collision de noms (`container_name` en dur).** Absente du cahier des charges. Conséquence : **la
   répétition ne se joue pas sur le NAS ni sur aucun hôte qui porte la production** ; elle se joue sur **une autre
   machine** (la station de Guy). Le manuel pose le contrôle `docker ps -a` « aucun `kesh-api` / `kesh-mariadb` » avant
   de commencer, et la raison (C-31-4-3). Le défaut lui-même (noms et port en dur, aucune pile jumelle possible) est
   **demandé en issue** (voir *Issues à demander*, n° 1).
7. **Ajoutée — la version de la copie.** « Compose de la copie téléchargé **au tag de la version relevée** » : ce compose
   référence `image: gcorbaz/kesh:latest`, qui n'est pas « la version relevée » dès qu'une version plus récente existe.
   La procédure **épingle** l'image par un `docker-compose.override.yml` (`image: gcorbaz/kesh:<version relevée>`) — c'est
   aussi ce qui permet, **au développement**, de substituer l'image construite depuis `main` (la 0.15.0 n'est pas
   publiée, C-31-4-4). Une copie **plus ancienne** que la sauvegarde est refusée par la protection anti-downgrade
   (`kesh_version_min_required`) : l'épinglage ne descend jamais sous la version de production.
8. **Ajoutée — la sauvegarde de sécurité de la copie.** L'import crée `backup/kesh-pre-import-*.keshbackup` (secret, 0600,
   root). Au **premier** import, elle ne porte qu'une installation presque vide (celle de `/setup`) ; au **second**
   (rejeu), elle porte **les données importées** — c'est le sens du « F5 » du cahier des charges (« la sauvegarde
   pré-import ne porte les secrets de la production qu'à partir d'un second import »). Elle fait partie de **ce que la
   destruction finale détruit** (AC7).
9. **Corrigée — « ex-29-7 : après la 29-1b ».** Satisfaite (29-1a, 29-1b `done`) ; la répétition **rejoue** donc les
   gestes de la 29-1a (préparation) et de la 29-1b (contre-passation datée) tels qu'ils sont à `main`.

## Périmètre

**Dedans** : une sous-section du manuel d'administration (`.tex` **et** PDF) ; un script de **fixture synthétique**
versionné (`scripts/repetition-fixture.sh`) qui fabrique la sauvegarde servant à écrire et à rejouer la procédure ; une
**garde** de texte (G22) qui épingle les trois contrôles de la copie ; l'entrée `CHANGELOG`. **La procédure est jouée
réellement au développement**, journal à l'appui.

**Dehors** (voir *Hors périmètre*) : toute modification du code du produit, des compose ou de `.env.example` (les noms
fixes sont **signalés**, non corrigés, n° 1) ; la répétition **sur le NAS** (impossible, ci-dessous) ; le manuel
utilisateur, les trois autres langues (le manuel d'administration n'existe qu'en français, `docs/manual/{de,en,it}/` ne
porte pas de traduction de ce manuel) ; la sauvegarde automatique (31-1c).

## Critères d'acceptation

**AC1 — une sous-section au manuel d'administration, jouée avant d'être écrite.** `docs/manual/fr/admin-manual.tex`
gagne `\subsection{Répéter une clôture sur une copie}\label{sec:repetition-cloture}`, placée dans
*Sauvegarde et restauration* **juste après** `Test de restauration périodique` (§ 1598), qui y renvoie (la répétition
est aussi un test de restauration au sens de l'OLICo art. 10 al. 1) ; `Quelle méthode de sauvegarde choisir ?` n'est pas
touché. Le PDF est **régénéré** (`make fr`, sous verrou, § 5 de `docs/agents-orchestration.md`) et commité. **Chaque
commande du manuel a été exécutée** sur la station, dans l'ordre où le manuel l'écrit, et le journal de cette exécution
est versé hors dépôt (AC9). Un pas qui n'a pas pu être exécuté **ne figure pas au manuel comme acquis** : il porte la
mention « non éprouvé sur la station » et son motif (AC10).

**AC2 — le but et la portée, dits avant la première commande.** La sous-section ouvre sur : *ce qu'elle fait* (jouer sur
une copie, restaurée depuis une sauvegarde complète, les gestes de la clôture) ; *ce qu'elle ne fait pas* (elle ne
touche jamais l'installation de production ; elle ne la **remplace** pas par la copie) ; *où elle se joue* (une machine
qui ne porte pas la production, avec Docker et Compose v2 ; **jamais le NAS** : noms de conteneurs fixes, § État des
lieux) ; *ce qu'elle suppose* (une installation en **0.15.0 ou plus** : le fichier `.keshbackup` est de format 2 et
emporte les pièces ; une sauvegarde de format 1 se restaure aussi, ses pièces manquent, ce que la copie montrera) ;
*que la copie contient les données réelles et se détruit à la fin* (AC7) ; et que **le fichier `.keshbackup` est un
secret** (mots de passe hachés, jetons de session — rappel de `Exporter`, sans le recopier).

**AC3 — les contrôles de la copie, tous AVANT le premier `up` (R1, R2, F5).** Dans l'ordre, chacun avec sa **sortie
attendue** écrite :
 1. **Hôte sans production.** `docker ps -a --format '{{.Names}}' | grep -E '^kesh-(api|mariadb)$'` ne rend rien ; sinon
    **arrêt** (la pile jumelle entrerait en collision, et le compose Synology partage le nom `kesh-api`).
 2. **Dossier neuf et nom de projet propre.** Un dossier **vide, créé pour l'occasion** (pas le dossier du compose de la
    production, pas celui d'une autre copie) ; `COMPOSE_PROJECT_NAME=kesh-repetition` posé **dans le `.env` de la
    copie**, de sorte que `docker compose down -v` ne vise jamais que les volumes de la copie.
 3. **Compose obtenu par téléchargement au tag.** `DEPOT=https://raw.githubusercontent.com/guycorbaz/kesh/v0.14.0`
    (littéral, **tag = `v<\keshVersion>`**, G19 ; le manuel dit de le remplacer par le tag de la version **relevée** sur
    la production par `GET /health`), `curl -fsSL "$DEPOT/docker-compose.yml"` — **et non** le fichier du dossier de la
    production, ni le `docker-compose.prod.yml`. Le nom `docker-compose.yml` ne prouvant rien, vient le contrôle 4.
 4. **Le compose embarque MariaDB et vise `@mariadb:3306`.** `docker compose config --services` rend **exactement**
    `mariadb` et `kesh-api` ; `docker compose config | grep DATABASE_URL` rend une ligne **contenant `@mariadb:3306`**.
    Sinon **arrêt, pas de `up`** : c'est le compose Synology, qui viserait la base de son `.env`.
 5. **`.env` de la copie depuis `.env.example`, jamais depuis celui de la production.** `curl -fsSL
    "$DEPOT/.env.example" -o .env`, puis : `MARIADB_ROOT_PASSWORD` et `MARIADB_PASSWORD` **neufs** (`openssl rand -hex
    32`) — **F L9 : pour la copie c'est la bonne valeur** ; `KESH_JWT_SECRET` neuf ; `KESH_COOKIE_SECURE=false` (HTTP
    local) ; **aucun** `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD` (l'écran `/setup` crée l'administrateur provisoire, la
    marche de `Restaurer sur une nouvelle machine`) ; **aucune** ligne `KESH_*_HOST_DIR`, **aucune** variable `KESH_SMTP_*`
    (la copie n'envoie aucun courriel — le dire, c'est la garantie que des rappels débiteurs importés ne partent pas).
 6. **Aucun montage hors de la copie (R2).** `env | grep '^KESH_'` ne rend rien ; `grep -E 'HOST_DIR' .env` ne rend rien ;
    **et** `docker compose config | grep -E '^\s+source:'` ne montre que des chemins **sous le dossier de la copie**
    (et le volume nommé `kesh-mariadb-data`). Tout autre chemin : arrêt.
 7. **Image épinglée et port local.** `docker-compose.override.yml` : `image: gcorbaz/kesh:<version relevée>` et
    `ports: !override ["127.0.0.1:<port libre>:80"]` — la copie n'écoute que sur la boucle locale (C-31-4-6).
    `docker compose config` est relu **après** l'override (contrôles 4 et 6 refaits sur la sortie finale).
 Le manuel écrit ces sept contrôles comme une **liste numérotée** dont le dernier pas est « seulement alors :
 `docker compose up -d` ».

**AC4 — la restauration, par renvoi et non par recopie.** Après le `up`, le manuel **renvoie** à `Restaurer sur une
nouvelle machine` (§ `sec:restauration-machine-neuve`) pour `/setup`, le lien « Restaurer une sauvegarde » et l'import,
et ne redit que ce qui est **propre à la copie** : l'adresse (`http://127.0.0.1:<port>`), le fichier à importer (le
`.keshbackup` pris sur la production par **Administration → Sauvegarde complète**, copié sur la station par un moyen que
le lecteur choisit — et traité comme un secret), la lecture du bilan (« pièces restaurées / manquantes »), la
reconnexion avec **les identifiants de la production**. **Critère de réussite** nommé : le bilan annonce *N* pièces
restaurées et **0 manquante** pour une sauvegarde de format 2 ; la facture la plus récente de la production est dans la
liste ; son PDF se télécharge ; un document fournisseur s'affiche.

**AC5 — la répétition des gestes de la clôture, avec pour chacun ce qu'on doit voir.** Quatre étapes, chacune renvoyant
au manuel utilisateur (`\ref`) pour le geste et écrivant **le constat attendu** :
 1. **Préparer la clôture** (29-1a) de l'exercice à clore : les sept contrôles s'affichent ; ce qui est « à signaler »
    est **noté** (c'est précisément ce que la répétition sert à découvrir) ; chaque lien mène à son écran.
 2. **Contre-passer les transitoires** (29-1b) : pour une écriture de régularisation (compte 1300 ou 2300), le bouton
    « Contre-passer au début de l'exercice suivant » ; la confirmation nomme l'écriture, la **date du premier jour de
    l'exercice suivant** et l'exercice ; l'écriture inverse est liée à l'origine ; le bilan de N porte la transitoire, le
    bilan de N+1 au premier jour la montre **soldée**. Si l'exercice suivant n'existe pas, le bouton est absent **et la
    fiche dit pourquoi** : le créer (c'est un point que la préparation signale).
 3. **Clôturer** : le dialogue affiche « Aucun point à vérifier » (ou la liste restante, notée) ; l'exercice est clos ;
    une saisie dans cet exercice est refusée.
 4. **Rouvrir** (administrateur, motif obligatoire) : l'exercice redevient ouvert, la saisie passe de nouveau ; on
    **referme** si l'on veut conserver l'état « clos » pour la suite de la répétition.
 Le manuel précise que ces gestes agissent **sur la copie seulement** : « rouvrir » ou « clôturer » dans la copie ne
 change rien à la production, et c'est le but.

**AC6 — ce que la répétition apprend, à noter.** Une courte liste **à remplir par le lecteur** (durée de la restauration,
points signalés par la préparation, refus rencontrés, pièces manquantes) à consigner dans le procès-verbal du test de
restauration annuel (OLICo art. 10 al. 1, `Test de restauration périodique`) : la répétition **est** ce test, avec un
objet de plus. Aucune valeur chiffrée de l'installation de Guy n'entre dans le manuel.

**AC7 — la copie ne laisse rien derrière elle.** (a) Le fichier `.keshbackup` importé, la sauvegarde de sécurité
`backup/kesh-pre-import-*` (secrets) et le `.env` de la copie font partie de ce qui est détruit. (b) La destruction est
`docker compose down -v` (avec le `COMPOSE_PROJECT_NAME` du contrôle 2 : volume `kesh-repetition_kesh-mariadb-data`), puis la
suppression du **dossier de la copie** — dossiers `log/`, `inbox/`, `documents/`, `backup/` **créés par Docker, en
root** : deux chemins écrits et éprouvés, **`sudo rm -rf <dossier>`** et, **sans `sudo`**, `docker run --rm -v
"<dossier>":/copie --entrypoint sh <image mariadb déjà tirée> -c 'rm -rf /copie/log /copie/inbox /copie/documents
/copie/backup'` puis `rm -rf <dossier>`. (c) **Contrôle final** écrit : `docker ps -a`, `docker volume ls`, `docker
network ls` ne montrent plus rien de `kesh-repetition`, et le dossier n'existe plus.
Le manuel écrit `rm -rf` sur une variable **uniquement** avec la garde « le chemin est non vide et se termine par
le nom du dossier de la copie » (consigne : un `rm -rf "$VAR"` non gardé est la faute dangereuse type) ; le dev
l'éprouve avec une variable vide et lit que la garde arrête.

**AC8 — la fixture synthétique : `scripts/repetition-fixture.sh`.** Un script `bash` (+ `curl`, `jq`, déjà présents sur
la station) qui, **contre une instance source en `KESH_TEST_MODE`** (base `kesh_314`, port attribué, loopback), fabrique
une comptabilité **synthétique** et en tire un `.keshbackup` de format 2 : `POST /api/v1/_test/seed {"preset":
"with-company-no-fy"}` ; exercices **2025 et 2026** (années fixes : rien ne dépend du jour d'exécution, comme le
parcours E2E) ; comptes `1300 Actifs transitoires`, `1020 Banque` ; compte bancaire lié (IBAN **de test** de
`fiscal-year-closing-journey.spec.ts`, `CH4431999123000889012`) et relevé CAMT `camt053_v04_year_end.xml` (fixture
existante, **non rapprochée** : la préparation doit la signaler) ; la **régularisation** de 250.00 au 31.12.2025 (débit
1300) ; **une facture client validée** (son PDF figé est une pièce du format 2) ; **un document fournisseur** déposé
dans le dossier d'import et importé (autre pièce du format 2) ; puis `GET /api/v1/admin/full-export` vers le fichier
demandé. Écrit à la lettre des appels de `fiscal-year-closing-journey.spec.ts` et de
`backup-restore-new-install.spec.ts` (mêmes routes, mêmes corps) ; **aucune donnée réelle**, aucun relevé du dossier
`~/kesh-test/` (jamais ouvert). Le script **refuse de tourner** si l'instance visée n'est pas en `KESH_TEST_MODE`
(l'endpoint `_test/seed` est alors absent : il s'arrête sur le 404) et n'écrit que dans le dossier de sortie qu'on lui
donne. Un test de script (`scripts/tests/repetition-fixture.test.sh`, sur le modèle de
`scripts/tests/prepare-release.test.sh`) exerce ses refus **sans serveur** (argument manquant, dossier de sortie non
vide, instance injoignable) ; le chemin nominal est exercé **au développement**, sur l'instance source réelle.

**AC9 — le journal d'exécution, hors dépôt.** `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-<AAAAMMJJ>.log` :
chaque commande du manuel **et sa sortie**, dans l'ordre, horodatée, avec le résultat des contrôles de l'AC3 (y compris
**un essai volontaire du refus** : le compose Synology, `docker-compose.prod.yml` du tag, passé aux contrôles 4 et 6, doit
**arrêter** la procédure — sans jamais lancer son `up`). Le journal **ne se verse pas au dépôt** (il peut contenir des
chemins et des empreintes). Les **captures** (thème clair seulement) de l'écran de restauration réussie, du panneau
*Préparer la clôture* et de la confirmation de contre-passation sont prises à l'exécution ; elles vont dans
`docs/manual/fr/screenshots/` **si le manuel les montre** (macro `\keshscreenshot`), sinon restent hors dépôt avec le
journal. Les données des captures sont celles de la fixture synthétique.

**AC10 — ce qui ne s'éprouve pas sur la station, écrit comme tel.** Le manuel contient un court encadré
(`keshnote`) *Ce que cette procédure n'a pas éprouvé* : **le Synology** (DSM, Container Manager, les ACL des dossiers
partagés, le compose `prod` à réseau externe, la copie du `.keshbackup` depuis le NAS vers la station — gestes décrits
par `Restaurer depuis elle`, dont la procédure est celle de la 31-1b), la **taille réelle** de l'installation (les temps
et l'espace disque de la fixture ne valent pas ceux de l'installation de Guy ; le manuel renvoie aux ordres de grandeur
déjà écrits : « jusqu'à environ trois fois la taille du fichier »), et **l'image publiée** si la procédure a été jouée
sur une image construite depuis `main` (C-31-4-4). **Comment la procédure les couvre** : (i) elle ne s'exécute **jamais
sur le NAS** ; le seul geste Synology qu'elle contient est la récupération du fichier, déjà documentée ; (ii) les
contrôles de l'AC3 sont **indépendants de l'hôte** (ils lisent la sortie de `docker compose config`) ; (iii) le **rejeu
sur l'image publiée** est demandé à la release (voir *Actions demandées*, n° 3).

**AC11 — la garde G22 : le manuel ne perd pas ses contrôles.** `crates/kesh-api/tests/textes_coherents.rs` gagne un test
**G22** (ligne du tableau d'en-tête, issue : aucune → « Story 31-4 ») qui lit `admin-manual.tex`, isole la
sous-section `sec:repetition-cloture`, la normalise (`manuel::normaliser`, qui lève `\keshcommand{}` et consorts) et exige :
(a) les trois jetons `@mariadb:3306`, `docker compose config` et `COMPOSE_PROJECT_NAME` ; (b) **l'ordre** : la
première occurrence de `docker compose config` précède la première de `up -d` ; (c) l'absence de tout `KESH_LOG_HOST_DIR=`,
`KESH_INBOX_HOST_DIR=`, `KESH_DOCUMENTS_HOST_DIR=` **assigné** dans un `lstlisting` (nommer la variable en prose, pour
dire de ne pas la poser, reste permis) ; (d) **positive adossée** : la sous-section existe et fait plus de N lignes
(la garde ne passe pas à vide sur une sous-section introuvable). Elle ne prétend pas prouver que la procédure est
*correcte* — seulement que ses contrôles ne s'effacent pas en silence ; la preuve de correction est le journal (AC9).
G19 (URL au tag) couvre déjà l'URL de la sous-section, sans test de plus.

**AC12 — CHANGELOG.** Entrée `Ajouté` dans la section `[0.15.0] — Non publié` : la répétition de clôture sur une copie,
au manuel d'administration, jouée sur une sauvegarde complète. Aucune autre entrée.

## La marche prescrite (squelette ; le texte exact est écrit en exécutant)

Le manuel écrit, dans cet ordre, et **ne change rien à cet ordre sans l'avoir rejoué** :

0. Relever sur la production : la version (`GET /health`, champ `version`) ; produire le `.keshbackup`
   (*Administration → Sauvegarde complète*) ; le copier sur la machine de la copie.
1. Contrôle d'hôte (AC3-1) → dossier neuf (AC3-2).
2. Compose et `.env.example` au tag (AC3-3, AC3-5) → **contrôles 4 et 6**, puis override (AC3-7) → contrôles refaits.
3. `docker compose up -d` ; attendre `GET /health` ; ouvrir `http://127.0.0.1:<port>/setup` ; **renvoi** à
   *Restaurer sur une nouvelle machine* pour l'assistant et l'import (AC4).
4. Les quatre gestes (AC5) ; le constat de chacun.
5. Le procès-verbal (AC6).
6. Destruction (AC7) et contrôle final.

## Mutations à prévoir

Sous verrou, mutation → test rouge → restauration (`git checkout -- <fichier>` **puis `touch`**, § 7 de
`docs/agents-orchestration.md`) :

| # | Mutation | Attendu |
|---|---|---|
| M1 | retirer du `.tex` le jeton `@mariadb:3306` de la sous-section | G22 (a) rouge |
| M2 | déplacer le `docker compose config` après le `up -d` | G22 (b) rouge |
| M3 | ajouter `KESH_DOCUMENTS_HOST_DIR=/volume1/kesh/documents` dans un `lstlisting` de la sous-section | G22 (c) rouge |
| M4 | renommer le label `sec:repetition-cloture` | G22 (d) rouge (sous-section introuvable, pas « vert à vide ») |
| M5 | écrire dans la sous-section une URL `raw.githubusercontent.com/…/main/…` | G19 rouge |
| M6 | retirer du script de fixture le refus « instance hors `KESH_TEST_MODE` » | `repetition-fixture.test.sh` rouge |
| M7 | retirer du script le refus « dossier de sortie non vide » | idem |

**Non mutable, dit ici** : l'exactitude de la procédure elle-même. Elle n'est portée que par le **journal d'exécution**
(AC9) ; aucun test ne remplace d'avoir joué les commandes. C'est la raison d'être de la story.

## Ce qui se vérifie à la station et ce qui ne se vérifie nulle part ici

| Vérifié sur la station | Non vérifiable ici |
|---|---|
| Toute la mécanique Docker/Compose de la copie ; les sept contrôles et leurs refus (dont le compose Synology refusé) ; l'import d'une sauvegarde de format 2 ; les quatre gestes de la clôture ; la destruction avec et sans `sudo` | Synology (DSM, Container Manager, ACL, `frontend` externe, Hyper Backup) ; la taille et la durée sur l'installation réelle ; l'image **publiée** 0.15.0 tant qu'elle n'existe pas |

## Issues à demander (à l'orchestrateur)

1. **`docker-compose.yml` ne permet pas une seconde pile** : `container_name` (`kesh-api`, `kesh-mariadb`) et
   publication `"80:80"` en dur — une copie ne peut coexister avec la production sur un même hôte, et le port 80 se
   surcharge seulement par un fichier `override` (`!override`, Compose ≥ 2.24). La procédure le **contourne** (hôte
   distinct, override) sans le corriger. Priorité proposée : **P3** (dette de configuration), hors décompte des bugs.
   Candidate à une story ultérieure ; pas à celle-ci (modifier les compose touche l'installation de Guy).

## Actions demandées à l'orchestrateur

1. **Aucun `closes`** nulle part : aucune issue ne porte la story ; la PR dit `refs` seulement si une issue est ouverte
   pour le n° 1 ci-dessus.
2. **`sprint-status.yaml`** : la clé `31-4-repetition-a-blanc` passe `ready-for-dev` (fait sur cette branche) ; la clé
   `29-7-repetition-a-blanc` reste `superseded` (déjà).
3. **À la release v0.15.0, après la publication de l'image** : rejouer la procédure du manuel **avec l'image publiée**
   (pas l'image de `main`) et verser le journal à côté du premier (`314-repetition-<date>.log`). C'est ce rejeu qui lève la
   réserve de l'AC10 (« image publiée »). Le script de fixture le rend peu coûteux (quelques minutes, hors verrou de gate
   hormis le `docker pull`). **Cette action n'est pas un critère du développeur** : elle appartient à la release.

## Tâches

- [ ] **T0 — état des lieux rejoué** : `git log -1 origin/main` ; `docker ps -a` (aucune pile `kesh-api` /
      `kesh-mariadb`) ; port libre ; `sudo -n true` (confirmer l'absence de `sudo`) ; base `kesh_314` créée et migrée
      depuis les migrations **du worktree**, port d'instance source attribué par l'orchestrateur.
- [ ] **T1 — la fixture** (AC8) : écrire `scripts/repetition-fixture.sh` **et son test sans serveur** (rouge d'abord,
      M6/M7) ; la jouer contre l'instance source (`cargo run`, `KESH_TEST_MODE=true`, recette de `docs/testing.md`) ;
      vérifier que le `.keshbackup` produit est de **format 2** et porte **deux pièces** (manifeste lu, sans l'ouvrir
      ailleurs que dans le dossier temporaire de la story).
- [ ] **T2 — l'image** : construire l'image depuis `main` du worktree (`docker build -t gcorbaz/kesh:314 .`, sous verrou
      et `mem-guard.sh` ; elle n'écrase pas `latest`). Noter l'empreinte au journal.
- [ ] **T3 — jouer la procédure** (AC3 à AC7), **avant** d'écrire le manuel, dans un dossier neuf hors dépôt ; journal
      AC9 ; les captures utiles ; **l'essai volontaire du refus** (compose Synology) ; la destruction avec `sudo` si
      disponible, sans `sudo` (conteneur jetable) sinon, **et** l'essai de la garde `rm -rf` à variable vide.
- [ ] **T4 — écrire la sous-section** à partir du journal, pas de la mémoire ; renvois par `\ref` ; encadré AC10 ;
      `make fr` sous verrou ; contrôler le PDF **aplati** (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) : la sous-section,
      les sept contrôles, l'URL au tag, la phrase « ne se joue jamais sur le NAS ».
- [ ] **T5 — G22** (AC11), tests rouges d'abord (M1 à M5), puis verts ; G19 relancée.
- [ ] **T6 — CHANGELOG** (AC12) ; `sprint-status.yaml` ; registre.
- [ ] **T7 — gate** : `scripts/test-fast.sh` complet sous verrou au dernier commit de code, base remise à zéro d'abord ;
      `cargo fmt`, `clippy` ; `scripts/tests/repetition-fixture.test.sh` ; **E2E complet non requis** (aucun écran, aucun
      code produit ne change) — **à dire dans le Dev Agent Record, sans l'afficher comme exécuté**.
- [ ] **T8 — rejouer** la procédure **à partir du manuel PDF** (et non du journal) : un second passage, en suivant le
      texte tel que livré, pour attraper l'écart entre ce qu'on a fait et ce qu'on a écrit ; corriger le manuel de ce
      que ce passage révèle.

## Dev Notes

- **La procédure contient du `rm -rf` et du `docker compose down -v`.** Ce sont les deux gestes qui détruisent ; tous
  deux sont gardés (nom de projet propre, chemin non vide). Au développement, **chacun s'exécute d'abord à blanc**
  (`echo`, `docker compose config`) et sur le seul dossier de la copie ; aucun `down` ni `rm` sur un dossier que la
  story n'a pas créé. `kesh-mariadb-dev` n'est **jamais** touché (interdit, `docs/agents-orchestration.md` § 2).
- **Pas de lecteur de la base de dev partagée** : la copie a sa propre MariaDB dans son volume ; seule l'instance
  **source** de la fixture emploie `kesh_314` du conteneur de dev.
- **Aucune donnée réelle** : la fixture est synthétique de bout en bout ; `~/kesh-test/` n'est pas ouvert ; le journal
  de gate-logs ne se verse pas ; les captures ne montrent que la fixture.
- **Le texte d'un manuel se vérifie contre lui-même** : T8 est le geste qui manque aux fiches qui « documentent » sans
  rejouer (les six tours de l'Epic 24, § *Le prompt d'une passe doit NOMMER le manuel*).
- **Conventions du manuel** : `\keshcommand{}`, `\keshpath{}`, `keshwarning`/`keshnote`/`keshtip`, `lstlisting[language=bash]`.
  Éviter l'emoji ; les exemples de commandes sont ceux du journal, avec les chemins de la copie en `<…>`.
- **Version dans le manuel** : tant que `\keshVersion` vaut `0.14.0`, l'URL est au tag `v0.14.0` (G19). La phrase « format
  2, depuis la 0.15.0 » est déjà celle du manuel (31-1a/31-1b) ; au bump de release, G19 force la mise à jour de l'URL.

## Hors périmètre

La sauvegarde automatique (31-1c) ; la correction des noms et du port en dur du compose (issue n° 1) ; une répétition
**sur le NAS** ; un E2E de la procédure (elle est une procédure d'exploitation sur Docker, hors du périmètre de Playwright)
; la traduction (le manuel d'administration n'existe qu'en français) ; le manuel utilisateur ; #404 et #235 (P1,
nommés à la rétrospective de l'Epic 31) ; #671.

## Change Log

- 2026-10-24 — fiche créée (agent de fiche, autonomie). Prémisses du cahier des charges de la 29-7 établies au code et
  à la station (R1, R2, F5, F L9, G19) ; trois ajouts (collision de noms fixes, épinglage de la version de la copie,
  sauvegarde de sécurité de la copie) ; une absence de `sudo` constatée sur la station. Choix C-31-4-1 à 9. Validation
  P1 à lancer (Opus, lentilles R et F ; `314-validate-prompt-p1.md`).

## Dev Agent Record

*(à remplir au développement : gates réellement exécutés avec leurs chiffres, journal de répétition, captures,
mutations tuées, écarts à la fiche.)*
