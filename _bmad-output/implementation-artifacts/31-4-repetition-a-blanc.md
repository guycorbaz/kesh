# Story 31.4 : La répétition à blanc d'une clôture (ex-29-7, documentation exécutée)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie ; remédiée le même jour après la validation P1 (Opus, lentilles R et F).
Registre : `epic-31-choix-autonomes-314.md`. Aucune issue GitHub ne porte cette story (la 29-7 est née en validation P2
de la 29-1 ; son sort a été tranché par Guy à la rétrospective de l'Epic 29, Q1 : reportée **après #503**, avant la
clôture de l'exercice 2026). Cahier des charges d'origine : commentaire de la clé `29-7-repetition-a-blanc` de
`sprint-status.yaml`, que cette fiche reprend **sans l'affaiblir** et complète de ce que la 31-1 a changé.*

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
| Compose autonome | `docker-compose.yml` : services `mariadb` (`mariadb:10.11`) **et** `kesh-api`. `DATABASE_URL` y est **construit** : `mysql://${MARIADB_USER:-kesh}:${MARIADB_PASSWORD:?…}@mariadb:3306/${MARIADB_DATABASE:-kesh}` (l. 63) — le `DATABASE_URL` du `.env` **n'y est pas lu**. Les mots de passe sont exigés par `:?` (l. 12, 15, 63) : sans `.env` posé, `docker compose config` échoue. `build: context: .` (l. 47-48) : une image introuvable fait tenter une construction dans le dossier courant |
| Compose Synology | `docker-compose.prod.yml` : **un seul service** (`kesh-api`), `DATABASE_URL: ${DATABASE_URL}` **lu dans le `.env`**, réseau externe `frontend`, montages relatifs `./log`, `./inbox`, `./documents`, `./backup`. Le manuel Synology l'enregistre sous le nom **`docker-compose.yml`** (`admin-manual.tex` l. 573 : `curl … "$DEPOT/docker-compose.prod.yml" -o docker-compose.yml`) : **le nom du fichier ne dit pas lequel c'est** — R1. Et le manuel prescrit pour lui une `DATABASE_URL` en `@mariadb:3306` (`admin-manual.tex` l. 269) : **ce jeton ne distingue pas les deux compose ; seul `config --services` le fait** (validation P1, R3) |
| Montages absolus (R2) | `docker-compose.yml` l. 140 et 152-153 : `${KESH_LOG_HOST_DIR:-./log}`, `${KESH_INBOX_HOST_DIR:-./inbox}`, `${KESH_DOCUMENTS_HOST_DIR:-./documents}`, `./backup`. Posées (dans le `.env` **ou dans l'environnement du shell**, qui l'emporte sur le `.env`), ces variables rendent les montages absolus : un *Importer* dans la copie viderait/réécrirait le dossier **de la production**. `.env.example` les porte **commentées** (l. 232, 233, 262) |
| Noms fixes | `container_name: kesh-mariadb` et `kesh-api` en dur dans `docker-compose.yml` (l. 5, 50) ; réseau `kesh-network`, volume `kesh-mariadb-data` (préfixés par le **nom du projet** Compose). Sur un hôte qui porte déjà la production, une seconde pile **entre en collision** (refus de Compose, dans le meilleur cas) |
| Nom de projet et environnement Compose (validation P1) | `.env.example` **ne porte pas** `COMPOSE_PROJECT_NAME` (`grep -nF` : rc=1) : le poser dans le `.env` puis télécharger `.env.example` par-dessus le perdait (R1). `scripts/synology/recette.sh` l. 33 **exporte** `COMPOSE_PROJECT_NAME` dans le shell. `kesh-mariadb-dev` est le service `mariadb` du compose de dev (`docker-compose.dev.yml` l. 55-57), dont le projet porte le nom du dossier du clone (`kesh`). **Non exécutés, tirés de la documentation de Compose v2** (constats à faire en T3) : préséance de `-p` sur `COMPOSE_PROJECT_NAME`, et de celle-ci sur le `.env` ; appariement des conteneurs par étiquettes *projet* + *service* (une copie dont le projet s'appellerait `kesh` viserait `kesh-mariadb-dev`) ; recherche d'un compose dans les dossiers parents quand aucun `-f` n'est donné ; `docker-compose.override.yml` **non chargé** dès qu'un `-f` est donné |
| Port | `docker-compose.yml` publie `"80:80"` (toutes interfaces) ; `KESH_PORT` change le port **dans** le conteneur, pas la publication. Station au 2026-10-24 : port 80 libre (`ss -ltn`), 3306 pris par `kesh-mariadb-dev` (127.0.0.1) |
| Compose au tag | `curl -fsSL https://raw.githubusercontent.com/guycorbaz/kesh/v0.14.0/docker-compose.yml` → 200. **Écart avec `main`** (`git diff --stat v0.14.0 HEAD -- docker-compose*.yml .env.example Dockerfile`) : dans chaque compose, une ligne de commentaire remplacée et deux ajoutées ; dans `.env.example`, des commentaires seulement (+11/−7, dont le défaut commenté `#KESH_ADMIN_IMPORT_MAX_MB` passé de 512 à 2048) ; `Dockerfile` : `rust:1.96` → `rust:1.97`. Aucune ligne exécutable des compose n'a changé *(décompte corrigé en validation P1, F L1 : la fiche disait « quatre lignes de commentaires »)* |
| Essai à blanc de `docker compose config` | sur le compose et le `.env.example` du tag `v0.14.0`, avec `MARIADB_ROOT_PASSWORD`, `MARIADB_PASSWORD`, `KESH_JWT_SECRET` posés : `DATABASE_URL: mysql://kesh:…@mariadb:3306/kesh` ✔ ; deux services `mariadb`, `kesh-api` ✔ ; montages `source:` résolus **sous le dossier courant** ✔ ; un `docker-compose.override.yml` portant `ports: !override [ "127.0.0.1:18314:80" ]` et `image:` est pris en compte (`host_ip: 127.0.0.1`, `published: "18314"`) ✔ (Compose 2.40.3). *Cet essai a réussi **parce que** les mots de passe étaient posés dans le shell* (validation P1, R2) |
| `.env.example` | lignes **vides de valeur utile** : `DATABASE_URL=<EDIT: …>` (l. 52 ; ignorée par `docker-compose.yml`, exigée par le Synology), `KESH_JWT_SECRET=<GENERATE_ME: …>` (l. 126 ; **refusée au démarrage** depuis la 0.13.0 tant qu'elle n'est pas remplacée, `is_template_placeholder`, `config.rs` l. 1542), `MARIADB_ROOT_PASSWORD` / `MARIADB_PASSWORD` **commentées** (l. 374, 377 ; Compose refuse de démarrer sans) ; `KESH_COOKIE_SECURE=true` (l. 155 ; cookie `Secure` refusé sur HTTP : la copie, en `http://127.0.0.1`, pose `false`) |
| Manuel : existant | `admin-manual.tex` § *Sauvegarde et restauration* : `Test de restauration périodique` (l. 1598, **sans `\label`** ; OLICo art. 10 al. 1 ; son étape 3 exige aussi un export de souveraineté dont le SHA-256 égale celui de la production) ; `Export/import d'installation via l'interface Kesh` (l. 1851) ; `Restaurer sur une nouvelle machine` (l. 2013, `sec:restauration-machine-neuve`, livrée par la 31-1b). **Son étape 1 recrée le `.env` depuis l'ancien et reprend `KESH_SMTP_*` et les dossiers** (l. 2018-2022) : l'inverse de la copie — **le renvoi ne vaut que pour ses étapes 2 à 5** (validation P1, R4/F6) |
| Manuel : gestes de clôture | `user-manual.tex` : `Préparer la clôture` (`sec:preparer-cloture`, l. 792), `Les transitoires de fin d'exercice` (`sec:transitoires`, l. 685 ; bouton « Contre-passer au début de l'exercice suivant », l. 699), `Clôture d'un exercice` (l. **823**, sans label), `Réouverture d'un exercice clôturé` (`sec:reouverture-exercice`, l. 843). **Deux documents compilés séparément, aucun paquet `xr`** : un `\ref` du manuel d'administration vers ces labels rendrait « ?? » ; la convention est le renvoi **en prose** (« manuel utilisateur, \emph{…} », `admin-manual.tex` l. 886, 2149, 2213). Défaut préexistant : le chemin du menu s'écrit « Paramètres → Exercices comptables » (l. 794, 845) et « Administration → Exercices comptables » (l. 827) ; le menu le range sous *Administration* (`routes/(app)/+layout.svelte` l. 118-124) — signalé, non traité ici |
| Macros du manuel | `\keshscreenshot` n'est définie que dans le préambule du manuel utilisateur (`user-manual.tex` l. 15) : ni `kesh-style.sty`, ni `admin-manual.tex` ne l'ont |
| Gardes qui lisent déjà le manuel d'administration | `crates/kesh-api/tests/configuration_transmise.rs` : **G14** (services nommés par `docker compose <verbe> <service>` ; la forme `docker compose -p <projet> …` **n'est pas lue**, angle mort écrit l. 2660-2662), **G15** (volumes cités ; le préfixe `kesh-repetition_` est accepté, l. 2757-2759), **G16 (e)** (liste **fermée** des « Hyper Backup » hors des sections Synology, l. 2779-2793 : nommer Hyper Backup dans la sous-section la ferait rougir) |
| Parcours déjà automatisé | `frontend/tests/e2e/fiscal-year-closing-journey.spec.ts` (29-1b) joue : exercices 2025 et 2026, comptes `1300`/`1020`, compte bancaire lié, relevé CAMT `camt053_v04_year_end.xml`, préparation (K2 et K3 en attention), rapprochement manuel, régularisation de 250.00 au 31.12.2025, contre-passation, clôture, réouverture. **La répétition en est la version « copie du NAS »**, pas un nouveau test fonctionnel. `backup-restore-new-install.spec.ts` exporte **par l'écran** (l. 103-105, `exportByScreen`), non par l'API ; la facture du helper `createAndValidateInvoiceViaApi` est **datée du jour** (`helpers/api-fixtures.ts` l. 105-106) ; le dépôt dans la boîte d'import écrit dans `KESH_INBOX_DIR`, par défaut `/tmp/kesh-e2e/inbox` (`helpers/inbox.ts` l. 27, 100), avec une poignée de main (l. 153-160) |
| Hors mode test | les routes `/api/v1/_test` ne sont montées qu'en `test_mode` (`lib.rs` l. 1153-1157) ; hors de ce mode, la requête tombe sur le repli `ServeDir` (l. 175-176, 1167), et `test_endpoints_e2e.rs` l. 97-104 admet **404 ou 405** : le refus de la fixture ne peut pas reposer sur un « 404 » |
| Garde G19 | `crates/kesh-api/tests/textes_coherents.rs` l. 1188 : toute URL `raw.githubusercontent.com/guycorbaz/kesh/<réf>/…` des `.tex`, du README et de `website/` doit porter `v<\keshVersion>` (au 2026-10-24 : **`v0.14.0`**, `kesh-style.sty` l. 64). Au bump de release (0.15.0), elle rougit tant que les URL n'ont pas suivi — **ce qui est voulu** et suffit à tenir l'URL de la répétition |
| Droits de la station | `sudo -n true` → *interactive authentication is required* : **un agent n'a pas `sudo`**. Les dossiers `log/`, `inbox/`, `documents/`, `backup/` d'une copie sont créés **par Docker, en root** (le conteneur tourne en root) : leur suppression est le point où la procédure doit offrir un chemin **sans `sudo`** (C-31-4-5) |
| Station, images | `gcorbaz/kesh:0.13.2`, `0.13.3` et d'autres anciennes présentes ; **aucune image portant le format 2** (la 0.15.0 n'est pas publiée). `docker build` possible depuis `main` (`Dockerfile`, Rust 1.97). Les crates sont en `0.14.0` : l'image construite depuis `main` affiche **`0.14.0` à `/health`** tout en portant le format 2, que la `gcorbaz/kesh:0.14.0` publiée **refuse** (`admin_backup/manifest.rs` l. 25) |
| Santé de la version | `GET /health` → `{"status":"ok","db":true,"version":"…"}` (`crates/kesh-api/src/routes/health.rs` l. 38-41) : c'est **là** que « la version relevée » se lit |
| Précédent de destruction gardée | `scripts/synology/recette.sh` : conteneurs et réseau de test à nom propre, `trap 'rm -rf "$W"' EXIT` (l. 43) — modèle pour la fixture et la garde du `rm -rf` |

## Prémisses de l'issue / du cahier des charges : établies, corrigées, ajoutées

1. **R1 (HIGH) — établie.** Le danger est réel **pour le compose Synology** : sa `DATABASE_URL` vient du `.env`, et un
   `.env` recopié de la production fait viser à la « copie » la base de la production. Le compose *autonome* est, lui,
   **sans ce risque** (il construit sa `DATABASE_URL` vers `@mariadb:3306`). **Précision** : la procédure ne peut donc pas
   se contenter de « prendre le compose autonome » — elle doit **contrôler** ce qu'elle a obtenu, parce que le fichier
   porte le même nom dans les deux installations. Le contrôle est `docker compose config --services` (exactement
   `mariadb` et `kesh-api`), **avant tout `up`** (AC3, contrôle 7) ; le `grep` de `@mariadb:3306` n'est qu'un second
   filet (*corrigé en validation P1, R3*).
2. **R2 — établie, avec deux cas que le cahier des charges ne nomme pas** : les variables `KESH_*_HOST_DIR` peuvent venir
   **de l'environnement du shell** et non du `.env` ; et les variables **de Compose et de Docker** du shell
   (`COMPOSE_PROJECT_NAME`, `COMPOSE_FILE`, `DOCKER_HOST`, `DOCKER_CONTEXT`) l'emportent elles aussi (*ajouté en
   validation P1, R1/F3*). La procédure commence donc par **neutraliser** l'environnement (AC3, contrôle 1), et la sortie
   de `docker compose config` ne doit montrer **aucun** `source:` hors du dossier de la copie (AC3, contrôles 7 et 8).
3. **F5 — établie.** Les dossiers créés par Docker appartiennent à root. Le cahier des charges ne prévoit que `sudo` ;
   **la station d'un agent n'en a pas** (relevé ci-dessus), et le manuel s'adresse aussi à des lecteurs sans `sudo` : la
   procédure donne les **deux** chemins, dont l'un par un conteneur jetable (C-31-4-5), **vérifié** au développement.
4. **F L9 — établie.** Le message du compose « reprendre la valeur avec laquelle la base a été créée, jamais une neuve »
   vaut pour une mise à jour ; **dans la copie, volume neuf, une valeur neuve de `MARIADB_PASSWORD` est la bonne**. Le
   manuel le dit, avec la raison, pour qu'on ne recopie pas le mot de passe de la production par scrupule. Le volume
   neuf se **contrôle** (AC3, contrôle 2 : aucun volume du projet `kesh-repetition` ne subsiste).
5. **G19 — établie** (ci-dessus). L'URL est **écrite en dur** dans un `lstlisting` (la macro ne s'y étend pas) ; la garde la
   tient égale à la macro.
6. **Ajoutée — la collision de noms (`container_name` en dur).** Absente du cahier des charges. Conséquence : **la
   répétition ne se joue pas sur le NAS ni sur aucun hôte qui porte la production** ; elle se joue sur **une autre
   machine** (la station de Guy). Le manuel pose le contrôle d'hôte avant de commencer, et la raison (C-31-4-3). Le défaut
   lui-même (noms et port en dur, aucune pile jumelle possible) est **demandé en issue** (voir *Issues à demander*, n° 1).
7. **Ajoutée — la version de la copie.** « Compose de la copie téléchargé **au tag de la version relevée** » : ce compose
   référence `image: gcorbaz/kesh:latest`, qui n'est pas « la version relevée » dès qu'une version plus récente existe.
   La procédure **épingle** l'image par un `docker-compose.override.yml` (`image: gcorbaz/kesh:<version relevée>`) — c'est
   aussi ce qui permet, **au développement**, de substituer l'image construite depuis `main` (la 0.15.0 n'est pas
   publiée, C-31-4-4). **Corrigée en validation P1 (R9/F1)** : la fiche affirmait qu'une copie plus ancienne que la
   sauvegarde serait refusée par la protection anti-downgrade. C'est faux : l'import ne refuse que si
   `kesh_version_min_required` de la source dépasse le binaire (`kesh-db/src/version.rs` l. 270-275 ; `admin.rs` l. 203),
   et ce champ ne bouge qu'à une migration *breaking* — une sauvegarde de 0.15.1 s'importe dans une 0.15.0. L'argument
   est retiré ; à sa place, un contrôle : après le `up`, `GET /health` de la copie rend **exactement** la version relevée
   sur la production (AC3, contrôle 9), sinon destruction et arrêt.
8. **Ajoutée — la sauvegarde de sécurité de la copie.** L'import crée `backup/kesh-pre-import-*.keshbackup` (secret, 0600,
   root). Au **premier** import, elle ne porte qu'une installation presque vide (celle de `/setup`) ; au **second**
   (rejeu), elle porte **les données importées** — c'est le sens du « F5 » du cahier des charges (« la sauvegarde
   pré-import ne porte les secrets de la production qu'à partir d'un second import »). Elle fait partie de **ce que la
   destruction finale détruit** (AC7), avec toute autre sauvegarde produite par la copie.
9. **Corrigée — « ex-29-7 : après la 29-1b ».** Satisfaite (29-1a, 29-1b `done`) ; la répétition **rejoue** donc les
   gestes de la 29-1a (préparation) et de la 29-1b (contre-passation datée) tels qu'ils sont à `main`.
10. **Ajoutée en validation P1 — le nom de projet ne se confie pas au `.env`.** La fiche le posait dans le `.env` de la
    copie, que le téléchargement de `.env.example` écrasait aussitôt, et qu'un `COMPOSE_PROJECT_NAME` du shell aurait de
    toute façon dominé (R1 HIGH). D'où le *principe directeur* ci-dessous.

## Principe directeur des commandes Compose (validation P1)

**Chaque commande Compose de la procédure porte `-p kesh-repetition`, `--project-directory "$COPIE"` et des `-f`
explicites** ; aucune ne dépend du dossier courant, d'une variable du shell ni d'un fichier découvert implicitement :

```sh
# avant l'override (contrôle 7)
docker compose -p kesh-repetition --project-directory "$COPIE" -f "$COPIE/docker-compose.yml" config --services
# après l'override (contrôles 8 et 9, gestes, destruction)
docker compose -p kesh-repetition --project-directory "$COPIE" \
  -f "$COPIE/docker-compose.yml" -f "$COPIE/docker-compose.override.yml" <config|pull|up -d|ps|logs|down -v>
```

- `-p` fixe le nom de projet, donc les étiquettes par lesquelles Compose retrouve **ses** conteneurs, volumes et réseau :
  `down -v` ne peut viser que `kesh-repetition`.
- `--project-directory` fixe le dossier où le `.env` est lu et où les chemins relatifs (`./log`…) se résolvent.
- Les `-f` explicites empêchent `COMPOSE_FILE` et la recherche d'un compose dans les dossiers parents ; avec un `-f`,
  `docker-compose.override.yml` n'est plus chargé d'office : il est **nommé**, donc vu.
- `$COPIE` est un chemin **absolu**, posé une fois, dont le dernier élément commence par `kesh-repetition-`.
- **Avant tout**, la procédure neutralise les variables du shell qui l'emportent (`COMPOSE_*`, `DOCKER_HOST`,
  `DOCKER_CONTEXT`, `KESH_*`, `MARIADB_*`) et vérifie le contexte Docker (contrôle 1).

Le manuel peut écrire les commandes sur deux lignes (`\`), jamais les raccourcir par une fonction ou un alias : la garde
G22 (e) lit chaque commande. Ces comportements de Compose viennent de sa documentation : **ils sont des constats à faire
en T3, avant d'écrire le manuel** (voir T3).

## Périmètre

**Dedans** : une sous-section du manuel d'administration (`.tex` **et** PDF), et un `\label` posé sur la sous-section
voisine `Test de restauration périodique` (mécanique, pour le renvoi) ; un script de **fixture synthétique** versionné
(`scripts/repetition-fixture.sh`) qui fabrique la sauvegarde servant à écrire et à rejouer la procédure, avec son test
(`scripts/tests/repetition-fixture.test.sh`) rattaché au gate par une enveloppe Rust ; une **garde** de texte (G22) qui
épingle les contrôles de la copie ; l'entrée `CHANGELOG`. **La procédure est jouée réellement au développement**, journal
à l'appui.

**Dehors** (voir *Hors périmètre*) : toute modification du code du produit, des compose ou de `.env.example` (les noms
fixes sont **signalés**, non corrigés, n° 1) ; la répétition **sur le NAS** (impossible, ci-dessous) ; le manuel
utilisateur (le défaut de chemin de menu est signalé, n° 2), les trois autres langues (le manuel d'administration n'existe
qu'en français, `docs/manual/{de,en,it}/` ne porte pas de traduction de ce manuel) ; la sauvegarde automatique (31-1c).

## Critères d'acceptation

**AC1 — une sous-section au manuel d'administration, jouée avant d'être écrite.** `docs/manual/fr/admin-manual.tex`
gagne `\subsection{Répéter une clôture sur une copie}\label{sec:repetition-cloture}`, placée dans
*Sauvegarde et restauration* **juste après** `Test de restauration périodique` (l. 1598), qui y renvoie (la répétition
peut servir de test de restauration au sens de l'OLICo art. 10 al. 1, sous la condition de l'AC6) ; cette sous-section
voisine reçoit un `\label{sec:test-restauration}` pour le renvoi retour ; `Quelle méthode de sauvegarde choisir ?` n'est
pas touché. Le PDF est **régénéré** (`make fr`, sous verrou, § 5 de `docs/agents-orchestration.md`) et commité. **Chaque
commande du manuel a été exécutée** sur la station, dans l'ordre où le manuel l'écrit, et le journal de cette exécution
est versé hors dépôt (AC9). Un pas qui n'a pas pu être exécuté **ne figure pas au manuel comme acquis** : il porte la
mention « non éprouvé sur la station » et son motif (AC10).

**AC2 — le but et la portée, dits avant la première commande.** La sous-section ouvre sur : *ce qu'elle fait* (jouer sur
une copie, restaurée depuis une sauvegarde complète, les gestes de la clôture) ; *ce qu'elle ne fait pas* (elle ne
touche jamais l'installation de production ; elle ne la **remplace** pas par la copie) ; *où elle se joue* (une machine
qui ne porte pas la production, avec Docker et Compose v2 ≥ 2.24 ; **jamais le NAS** : noms de conteneurs fixes, § État
des lieux ; un dossier **local**, jamais synchronisé ni monté depuis le NAS) ; *ce qu'elle suppose* (une installation en
**0.15.0 ou plus** : le fichier `.keshbackup` est de format 2 et emporte les pièces ; une sauvegarde de format 1 se
restaure aussi, ses pièces manquent, ce que la copie montrera) ; *que la copie contient les données réelles et se détruit
à la fin* (AC7) ; et que **le fichier `.keshbackup` est un secret** (mots de passe hachés, jetons de session — rappel de
`Exporter`, sans le recopier).

**AC3 — les contrôles de la copie : les huit premiers AVANT le premier `up` (R1, R2, F5).** Toutes les commandes Compose
suivent le *principe directeur*. Dans l'ordre, chacun avec sa **sortie attendue** écrite, et **arrêt** sinon :
 1. **Environnement neutre, démon local.** `env | grep -E '^(COMPOSE_|KESH_|MARIADB_|DOCKER_HOST=|DOCKER_CONTEXT=)'` ne
    rend rien ; sinon `unset` de chaque variable listée, et contrôle refait. `docker context show` est lu et
    `docker context inspect --format '{{.Endpoints.docker.Host}}'` rend une adresse `unix://` (démon de la machine),
    jamais `tcp://` ni `ssh://` (un contexte distant ferait jouer la copie sur un autre hôte, le NAS compris).
 2. **Hôte sans production ni reste d'une répétition.** `docker ps -a --format '{{.Names}}'` **réussit** (code 0 : un
    démon injoignable « ne rend rien » aussi) et ne contient ni `kesh-api` ni `kesh-mariadb` (noms exacts ; la pile
    jumelle entrerait en collision, et le compose Synology partage le nom `kesh-api`) ; `docker ps -a -q`,
    `docker volume ls -q` et `docker network ls -q`, chacun avec `--filter label=com.docker.compose.project=kesh-repetition`,
    ne rendent rien (un volume resté d'une répétition non détruite garderait l'ancien mot de passe — F L9) ;
    `docker compose ls -a` ne liste pas `kesh-repetition`.
 3. **Dossier de la copie : neuf, local, absolu.** `COPIE=<chemin absolu>/kesh-repetition-<AAAAMMJJ>` (jamais un dossier
    nommé `kesh` : c'est le nom de projet du clone de dev et, par défaut, de bien des installations) ; créé **vide** pour
    l'occasion ; **jamais** sous un dossier synchronisé (Synology Drive Client, par défaut `~/SynologyDrive`) ni sur un
    partage monté : `findmnt -T "$COPIE"` rend un système de fichiers local (ni `cifs`, ni `nfs*`, ni `fuse.*`). Les
    pièces réelles restaurées et la sauvegarde de sécurité y vivront : un dossier synchronisé les ferait remonter vers
    le NAS et ses sauvegardes.
 4. **Compose et `.env.example` téléchargés au tag, dans la copie.** `DEPOT=https://raw.githubusercontent.com/guycorbaz/kesh/v0.14.0`
    (littéral, **tag = `v<\keshVersion>`**, G19 ; le manuel dit de le remplacer par le tag de la version **relevée** sur
    la production par `GET /health`) ; `curl -fsSL "$DEPOT/docker-compose.yml" -o "$COPIE/docker-compose.yml"` et
    `curl -fsSL "$DEPOT/.env.example" -o "$COPIE/.env"` — **jamais** le fichier du dossier de la production, ni le
    `docker-compose.prod.yml`. Le nom `docker-compose.yml` ne prouvant rien, vient le contrôle 7.
 5. **`.env` de la copie, rempli, jamais repris de la production.** Dans `$COPIE/.env` : `MARIADB_ROOT_PASSWORD` et
    `MARIADB_PASSWORD` **neufs** (`openssl rand -hex 32`) — **F L9 : pour la copie c'est la bonne valeur** ;
    `KESH_JWT_SECRET` neuf ; `KESH_COOKIE_SECURE=false` (HTTP local) ; **aucun** `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD`
    (l'écran `/setup` crée l'administrateur provisoire) ; **aucune** variable `KESH_SMTP_*` (la copie n'envoie aucun
    courriel — le dire, c'est la garantie que des rappels débiteurs importés ne partent pas) ; **pas** de
    `COMPOSE_PROJECT_NAME` (le `-p` le porte).
 6. **Aucune ligne `HOST_DIR` active.** `grep -nE '^[[:space:]]*KESH_[A-Z_]*HOST_DIR[[:space:]]*=' "$COPIE/.env"` ne rend
    rien (les trois lignes **commentées** de `.env.example`, l. 232, 233, 262, ne comptent pas ; un `grep 'HOST_DIR'` nu
    les rendrait et apprendrait au lecteur à passer outre un contrôle — validation P1, F2).
 7. **Première lecture de la configuration** (le `.env` est posé : les `:?` sont satisfaits). Avec le seul premier `-f` :
    `config --services` rend **exactement** `mariadb` et `kesh-api` — **c'est le contrôle décisif** : le compose Synology
    n'a qu'un service ; `config` rend en première ligne `name: kesh-repetition` ; sa `DATABASE_URL` contient
    `@mariadb:3306` (**second filet seulement** : le manuel prescrit cette forme au Synology aussi, l. 269) ; tous ses
    `source:` sont sous `$COPIE`, hormis le volume nommé `kesh-mariadb-data`.
 8. **Override, puis configuration relue.** `$COPIE/docker-compose.override.yml` : `image: gcorbaz/kesh:<version relevée>`
    et `ports: !override ["127.0.0.1:<port libre>:80"]` — la copie n'écoute que sur la boucle locale (C-31-4-6). Les
    commandes portent désormais les **deux** `-f`. Le contrôle 7 est **refait sur la sortie finale**, et en plus : l'`image:`
    est celle épinglée, la publication porte `host_ip: 127.0.0.1` et le port choisi.
 9. **Seulement alors : l'image, le démarrage, la version.** `pull kesh-api` (une version mal écrite échoue ici, au lieu
    d'une construction tentée dans `$COPIE`, `build: context: .`) ; `up -d` ; `curl -fsS http://127.0.0.1:<port>/health` :
    `version` **égale exactement** la version relevée à l'étape 0 — l'import n'y oblige pas (prémisse 7) ; sinon `down -v`
    et arrêt.
 Le manuel écrit ces neuf contrôles comme une **liste numérotée**, dans cet ordre.

**AC4 — la restauration, par renvoi et non par recopie.** Après le contrôle 9, le manuel **renvoie aux étapes 2 à 5** de
`Restaurer sur une nouvelle machine` (§ `sec:restauration-machine-neuve`) pour `/setup`, le lien « Restaurer une
sauvegarde », l'import et la reconnexion, et écrit : « **l'étape 1 (le `.env`) ne vaut pas ici** : le `.env` de la copie
est celui des contrôles 4 à 6, rien n'est repris de l'ancien ». Il ne redit que ce qui est **propre à la copie** :
l'adresse (`http://127.0.0.1:<port>`) ; le fichier à importer — le `.keshbackup` pris sur la production par
**Administration → Sauvegarde complète** —, dont la répétition travaille sur **un exemplaire placé dans
`$COPIE/source.keshbackup`** (mode 0600) : l'original, où qu'il soit conservé, n'est ni déplacé ni supprimé par la
procédure, et un exemplaire **créé pour l'occasion** hors de `$COPIE` (un téléchargement du navigateur) y est
**déplacé** ; tout fichier téléchargé depuis la copie (un PDF de facture) va dans `$COPIE/telechargements/` ; la lecture
du bilan (« pièces restaurées / manquantes ») ; la reconnexion avec **les identifiants de la production**. **Critère de
réussite** nommé : le bilan annonce *N* pièces restaurées et **0 manquante** pour une sauvegarde de format 2 ; la facture
la plus récente de la production est dans la liste ; son PDF se télécharge ; un document fournisseur s'affiche.

**AC5 — la répétition des gestes de la clôture, avec pour chacun ce qu'on doit voir.** Quatre étapes, chacune renvoyant
au manuel utilisateur **en prose, par le titre de sa section** (« manuel utilisateur, \emph{Préparer la clôture} », comme
`admin-manual.tex` l. 886 — aucun `\ref` entre les deux manuels, qui rendrait « ?? ») pour le geste, nommant le menu tel
que le code le range (**Administration → Exercices comptables**), et écrivant **le constat attendu** :
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
points signalés par la préparation, refus rencontrés, pièces manquantes) à consigner dans un procès-verbal. La répétition
**vaut test de restauration annuel** (OLICo art. 10 al. 1, `Test de restauration périodique`, `\S\ref{sec:test-restauration}`)
**seulement si** le lecteur fait aussi, **avant les gestes de la clôture** (qui modifient la copie), l'étape 3 de ce test :
l'export de souveraineté et la comparaison de son SHA-256 avec celui de la production. Si le constat de T3 montre que les
deux empreintes ne peuvent pas être égales par construction, le manuel le dit et ne prétend pas que la répétition vaut ce
test. Aucune valeur chiffrée de l'installation de Guy n'entre dans le manuel.

**AC7 — la copie ne laisse rien derrière elle.** (a) **Ce qui est détruit** : les conteneurs, le volume
`kesh-repetition_kesh-mariadb-data`, le réseau, et **le dossier `$COPIE` entier** — qui contient l'exemplaire
`source.keshbackup`, le `.env`, **toute sauvegarde produite par la copie** (`backup/kesh-pre-import-*` et toute autre) et
les téléchargements de `$COPIE/telechargements/`. L'original de la sauvegarde n'est jamais touché. (b) **La
destruction** : `down -v` selon le *principe directeur* (`-p kesh-repetition`, `--project-directory`, les deux `-f`), puis
la suppression de `$COPIE` — dossiers `log/`, `inbox/`, `documents/`, `backup/` **créés par Docker, en root** : deux
chemins écrits et éprouvés, **`sudo rm -rf "$COPIE"`** et, **sans `sudo`**, `docker run --rm -v "$COPIE":/copie
--entrypoint sh <image mariadb déjà tirée> -c 'rm -rf /copie/log /copie/inbox /copie/documents /copie/backup'` puis
`rm -rf "$COPIE"`. Chaque `rm -rf` et le `docker run -v` sont précédés de la **garde** : `$COPIE` est non vide, **absolu**
(un chemin relatif passé à `-v` crée un volume nommé, et le `rm` tourne à vide — validation P1, R L2), son dernier élément
commence par `kesh-repetition-`, et c'est un dossier existant ; la garde est écrite sans `exit` (qui fermerait le terminal
du lecteur). Consigne : un `rm -rf "$VAR"` non gardé est la faute dangereuse type ; le dev l'éprouve avec une variable vide
**et** un chemin relatif, et lit que la garde arrête. (c) **Contrôle final** écrit : le contrôle 2 refait (ni `kesh-api` ni
`kesh-mariadb` parmi les noms ; conteneurs, volumes et réseaux filtrés par l'étiquette `kesh-repetition` : rien) ;
`test ! -e "$COPIE"` ; et aucun exemplaire de la répétition (`.keshbackup`, PDF) ne subsiste dans le dossier de
téléchargements du navigateur.

**AC8 — la fixture synthétique : `scripts/repetition-fixture.sh`.** Un script `bash` (+ `curl`, `jq`, déjà présents sur
la station) qui, **contre une instance source en `KESH_TEST_MODE`** (base `kesh_314`, port attribué, loopback), fabrique
une comptabilité **synthétique** et en tire un `.keshbackup` de format 2.
- **Arguments, trois, sans défaut ni héritage de l'environnement** : `repetition-fixture.sh <url> <dossier de sortie>
  <boîte de dépôt>`. Le script ne lit pas `KESH_INBOX_DIR`. La boîte est **propre à l'instance source** : le dev lance
  `cargo run` avec `KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR` et `KESH_ADMIN_BACKUP_DIR` sous
  `/home/gcorbaz/devel/kesh-gate-logs/314-source/` (jamais `/tmp/kesh-e2e/`, partagé entre worktrees). Le script
  n'écrit **que** dans le dossier de sortie et dans la boîte.
- **Refus, tous avant le premier appel qui écrit** : argument manquant ; URL hors boucle locale (`http://127.0.0.1:` ou
  `http://localhost:` seulement) ; dossier de sortie non vide ; boîte non vide ; instance injoignable ; et
  `POST /api/v1/_test/seed` qui ne rend pas un **2xx** — **tout autre code refuse**, 404 et 405 compris (hors mode test,
  le repli `ServeDir` peut rendre l'un ou l'autre, § État des lieux). Ce dernier refus est le seul qui empêche le
  `TRUNCATE` de `_test/seed` sur une instance qui n'est pas en mode test.
- **Contenu** : `POST /api/v1/_test/seed {"preset": "with-company-no-fy"}` ; exercices **2025 et 2026** ; comptes
  `1300 Actifs transitoires`, `1020 Banque` ; compte bancaire lié (IBAN **de test** de `fiscal-year-closing-journey.spec.ts`,
  `CH4431999123000889012`) et relevé CAMT `camt053_v04_year_end.xml` (fixture existante, **non rapprochée** : la
  préparation doit la signaler) ; la **régularisation** de 250.00 au 31.12.2025 (débit 1300) ; **une facture client
  validée**, datée d'une **date fixe de 2026** (`2026-06-15`), non du jour comme le helper `createAndValidateInvoiceViaApi`
  (son PDF figé est une pièce du format 2) ; **un document fournisseur** déposé dans la boîte et importé
  (`POST /api/v1/inbox-import`), qui doit revenir dans `accepted[]` — sinon refus : c'est la poignée de main de
  `helpers/inbox.ts` l. 153-160, sans sonde distincte ; puis `GET /api/v1/admin/full-export` (`lib.rs` l. 251) vers
  `<dossier de sortie>/fixture.keshbackup`. Les routes et les corps sont **repris** de `fiscal-year-closing-journey.spec.ts`
  et de `backup-restore-new-install.spec.ts` ; l'appel d'export est **écrit sur la route** (les deux spécifications
  exportent par l'écran). Rien ne dépend du jour d'exécution. **Aucune donnée réelle**, aucun relevé du dossier
  `~/kesh-test/` (jamais ouvert).
- **Test** : `scripts/tests/repetition-fixture.test.sh`, sur le modèle de `scripts/tests/prepare-release.test.sh` (un
  `curl` **factice** en tête du `PATH`, qui trace ses appels, l. 6-13 du modèle) : argument manquant, URL non locale,
  sortie non vide, boîte non vide, instance injoignable (code de `curl` 7) ; `_test/seed` en **404** puis en **405** →
  refus **et aucun appel suivant** dans la trace ; en **200** → le script passe à l'appel suivant (la trace en porte un
  second). Le test est lancé au gate par une enveloppe `crates/kesh-api/tests/repetition_fixture_script.rs`, calquée sur
  `crates/kesh-db/tests/prepare_release_script.rs` (un test shell que rien ne lance serait un test muet). Le chemin
  nominal est exercé **au développement**, sur l'instance source réelle.

**AC9 — le journal d'exécution, hors dépôt.** `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-<AAAAMMJJ>.log` :
chaque commande du manuel **et sa sortie**, dans l'ordre, horodatée, avec le résultat des contrôles de l'AC3, les
constats de T3, et **trois essais volontaires du refus**, aucun ne lançant de `up` :
 (i) **le compose Synology** : `docker-compose.prod.yml` du tag, enregistré sous le nom `docker-compose.yml` dans un
     second dossier `kesh-repetition-essai-<AAAAMMJJ>`, avec un `.env` **synthétique** dont la `DATABASE_URL` vaut
     `mysql://kesh:essai@mariadb:3306/kesh` — attendu : contrôle 7, `--services` rend `kesh-api` seul → **arrêt** ; le
     `grep` de `@mariadb:3306` **passe** (preuve qu'il ne suffit pas) ; les `source:` relatifs **passent** ;
 (ii) **un nom de projet hérité** : `export COMPOSE_PROJECT_NAME=kesh-essai-314` (jamais `kesh`, ni le nom d'une pile
      existante) — attendu : contrôle 1 le liste → arrêt ; puis, la variable posée, `config` avec `-p kesh-repetition`
      rend `name: kesh-repetition` (constat de préséance) ; puis `unset` ;
 (iii) **la garde du `rm -rf`** avec `COPIE` vide puis relatif — attendu : la garde arrête, rien n'est supprimé.
Le journal **ne se verse pas au dépôt** (il peut contenir des chemins et des empreintes). Les **captures** (thème clair
seulement) de l'écran de restauration réussie, du panneau *Préparer la clôture* et de la confirmation de
contre-passation sont prises à l'exécution et **restent hors dépôt**, avec le journal : le manuel d'administration n'a pas
de macro de capture (`\keshscreenshot` est propre au manuel utilisateur) ; s'il en montre une, c'est par
`\includegraphics` dans une `figure`, la capture allant dans `docs/manual/fr/screenshots/`. Les données des captures sont
celles de la fixture synthétique.

**AC10 — ce qui ne s'éprouve pas sur la station, écrit comme tel.** Le manuel contient un court encadré
(`keshnote`) *Ce que cette procédure n'a pas éprouvé* : **le Synology** (DSM, Container Manager, les ACL des dossiers
partagés, le compose `prod` à réseau externe, la copie du `.keshbackup` depuis le NAS vers la station — gestes décrits
par `Restaurer depuis elle`, dont la procédure est celle de la 31-1b), la **taille réelle** de l'installation (les temps
et l'espace disque de la fixture ne valent pas ceux de l'installation de Guy ; le manuel renvoie aux ordres de grandeur
déjà écrits : « jusqu'à environ trois fois la taille du fichier »), et **l'image publiée** si la procédure a été jouée
sur une image construite depuis `main` (C-31-4-4 : au développement, `gcorbaz/kesh:314` affiche `0.14.0` à `/health`
tant que les crates ne sont pas bumpées, et le `pull` est remplacé par `docker image inspect`). **Comment la procédure
les couvre** : (i) elle ne s'exécute **jamais sur le NAS** ; le seul geste Synology qu'elle contient est la récupération
du fichier, déjà documentée ; (ii) les contrôles de l'AC3 sont **indépendants de l'hôte** (ils lisent l'environnement,
le démon et la sortie de `docker compose config`) ; (iii) le **rejeu sur l'image publiée** est demandé à la release (voir
*Actions demandées*, n° 3). L'encadré **ne nomme pas Hyper Backup** (G16 (e), liste fermée).

**AC11 — la garde G22 : le manuel ne perd pas ses contrôles.** `crates/kesh-api/tests/textes_coherents.rs` gagne un test
**G22** (ligne du tableau d'en-tête, issue : aucune → « Story 31-4 ») qui lit `admin-manual.tex`, isole la
sous-section `sec:repetition-cloture`, la normalise (`manuel::normaliser`) et lit ses listings par `manuel::listings`
(`tests/common/manuel.rs` l. 125). Elle exige :
(a) les jetons `config --services`, `name: kesh-repetition`, `@mariadb:3306`, `docker context`, `/health` et
    `étapes 2 à 5` ;
(b) **l'ordre, par position dans le texte concaténé des listings** (non par première occurrence) : toute occurrence de
    `config` d'une commande `docker compose` précède la première de `up -d` ; **au moins une** de ces occurrences se situe
    **après** la première mention de `docker-compose.override.yml` et **avant** `up -d` (la relecture après override) ;
    `/health` suit `up -d` ;
(c) l'absence de toute affectation active `KESH_*HOST_DIR=` dans un listing (motif `^\s*KESH_[A-Z_]*HOST_DIR\s*=` ;
    nommer la variable en prose, pour dire de ne pas la poser, reste permis) ;
(d) **positive adossée** : la sous-section existe, compte **au moins 60 lignes** normalisées, et ses listings portent
    **au moins 8** commandes `docker compose` (la garde ne passe pas à vide sur une sous-section introuvable ou vidée) ;
(e) **chaque** commande `docker compose` de ses listings porte `-p kesh-repetition` **et** `--project-directory`
    (une commande continuée par `\` se lit jusqu'à sa fin).
Elle ne prétend pas prouver que la procédure est *correcte* — seulement que ses contrôles ne s'effacent pas en silence ;
la preuve de correction est le journal (AC9). G19 (URL au tag) couvre déjà l'URL de la sous-section, sans test de plus.
**G14** ne lit pas les commandes de la sous-section (forme `-p`, angle mort écrit de G14) : c'est G22 (e) qui les tient.
**G15** et **G16** relisent la sous-section : lancer le binaire `configuration_transmise` au gate ciblé (T5).

**AC12 — CHANGELOG.** Entrée `Ajouté` dans la section `[0.15.0] — Non publié` : la répétition de clôture sur une copie,
au manuel d'administration, jouée sur une sauvegarde complète. Aucune autre entrée.

## La marche prescrite (squelette ; le texte exact est écrit en exécutant)

Le manuel écrit, dans cet ordre, et **ne change rien à cet ordre sans l'avoir rejoué** :

0. Relever sur la production : la version (`GET /health`, champ `version`) ; produire le `.keshbackup`
   (*Administration → Sauvegarde complète*).
1. Contrôles 1 à 3 (environnement, hôte, dossier) ; placer l'exemplaire `$COPIE/source.keshbackup` (0600) (AC4).
2. Contrôles 4 à 8 : téléchargements au tag → `.env` rempli → lignes `HOST_DIR` → première lecture de `config` →
   override et relecture.
3. Contrôle 9 : `pull`, `up -d`, version à `/health` ; ouvrir `http://127.0.0.1:<port>/setup` ; **renvoi aux étapes 2 à 5**
   de *Restaurer sur une nouvelle machine* (AC4).
4. Le cas échéant, l'export de souveraineté et son SHA-256 (AC6) ; puis les quatre gestes (AC5) et le constat de chacun.
5. Le procès-verbal (AC6).
6. Destruction (AC7) et contrôle final.

## Mutations à prévoir

Sous verrou, mutation → test rouge → restauration (`git checkout -- <fichier>` **puis `touch`**, § 7 de
`docs/agents-orchestration.md`) :

| # | Mutation | Attendu |
|---|---|---|
| M1 | retirer du `.tex` le jeton `config --services` de la sous-section | G22 (a) rouge |
| M2 | déplacer **la relecture `config` qui suit l'override** après le `up -d` | G22 (b) rouge |
| M3 | ajouter `KESH_DOCUMENTS_HOST_DIR=/volume1/kesh/documents` dans un `lstlisting` de la sous-section | G22 (c) rouge |
| M4 | renommer le label `sec:repetition-cloture` | G22 (d) rouge (sous-section introuvable, pas « vert à vide ») |
| M5 | écrire dans la sous-section une URL `raw.githubusercontent.com/…/main/…` | G19 rouge |
| M6 | réduire le refus de `_test/seed` à « 404 seulement » | `repetition-fixture.test.sh` rouge (cas 405), donc l'enveloppe Rust rouge |
| M7 | retirer du script le refus « dossier de sortie non vide » | idem |
| M8 | retirer `-p kesh-repetition` de la commande `down -v` | G22 (e) rouge |
| M9 | retirer la phrase « étapes 2 à 5 » du renvoi | G22 (a) rouge |
| M10 | retirer du script le refus « boîte de dépôt non vide » | `repetition-fixture.test.sh` rouge |
| M11 | retirer du script le refus « URL hors boucle locale » | idem |

**Non mutable, dit ici** : l'exactitude de la procédure elle-même. Elle n'est portée que par le **journal d'exécution**
(AC9) ; aucun test ne remplace d'avoir joué les commandes. C'est la raison d'être de la story.

## Ce qui se vérifie à la station et ce qui ne se vérifie nulle part ici

| Vérifié sur la station | Non vérifiable ici |
|---|---|
| Toute la mécanique Docker/Compose de la copie ; les neuf contrôles et leurs refus (compose Synology, nom de projet hérité, garde du `rm -rf`) ; l'import d'une sauvegarde de format 2 ; les quatre gestes de la clôture ; la destruction avec et sans `sudo` | Synology (DSM, Container Manager, ACL, `frontend` externe, sauvegardes du NAS) ; la taille et la durée sur l'installation réelle ; l'image **publiée** 0.15.0 tant qu'elle n'existe pas |

## Issues à demander (à l'orchestrateur)

1. **`docker-compose.yml` ne permet pas une seconde pile** : `container_name` (`kesh-api`, `kesh-mariadb`) et
   publication `"80:80"` en dur — une copie ne peut coexister avec la production sur un même hôte, et le port 80 se
   surcharge seulement par un fichier `override` (`!override`, Compose ≥ 2.24). La procédure le **contourne** (hôte
   distinct, override) sans le corriger. Priorité proposée : **P3** (dette de configuration), hors décompte des bugs.
   Candidate à une story ultérieure ; pas à celle-ci (modifier les compose touche l'installation de Guy).
2. **Manuel utilisateur : le chemin des exercices s'écrit de deux façons** (validation P1, F L8) — « Paramètres →
   Exercices comptables » (`user-manual.tex` l. 794, 845) et « Administration → Exercices comptables » (l. 827) ; le menu
   le range sous *Administration* (`routes/(app)/+layout.svelte` l. 118-124). Priorité proposée : **P3** (manuel qui décrit
   autre chose que le code), label `bug`. La 31-4 écrit le chemin juste dans sa propre sous-section et ne touche pas le
   manuel utilisateur.

## Actions demandées à l'orchestrateur

1. **Aucun `closes`** nulle part : aucune issue ne porte la story ; la PR dit `refs` seulement si une issue est ouverte
   pour le n° 1 ou le n° 2 ci-dessus.
2. **`sprint-status.yaml`** : la clé `31-4-repetition-a-blanc` reste `ready-for-dev` (fait sur cette branche) ; la clé
   `29-7-repetition-a-blanc` reste `superseded` (déjà).
3. **À la release v0.15.0, après la publication de l'image** : rejouer la procédure du manuel **avec l'image publiée**
   (pas l'image de `main`) et verser le journal à côté du premier (`314-repetition-<date>.log`). C'est ce rejeu qui lève la
   réserve de l'AC10 (« image publiée »). Le script de fixture le rend peu coûteux (quelques minutes, hors verrou de gate
   hormis le `docker pull`). **Cette action n'est pas un critère du développeur** : elle appartient à la release.

## Tâches

- [ ] **T0 — état des lieux rejoué** : `git log -1 origin/main` ; `docker ps -a` (aucune pile `kesh-api` /
      `kesh-mariadb`) ; port libre ; `sudo -n true` (confirmer l'absence de `sudo`) ; base `kesh_314` créée et migrée
      depuis les migrations **du worktree**, port d'instance source attribué par l'orchestrateur ; dossiers de l'instance
      source sous `/home/gcorbaz/devel/kesh-gate-logs/314-source/` (AC8).
- [ ] **T1 — la fixture** (AC8) : écrire `scripts/repetition-fixture.sh`, **son test au `curl` factice** et l'enveloppe
      `crates/kesh-api/tests/repetition_fixture_script.rs` (rouge d'abord, M6, M7, M10, M11) ; la jouer contre l'instance
      source (`cargo run`, `KESH_TEST_MODE=true`, recette de `docs/testing.md`, avec la boîte propre) ; vérifier que le
      `.keshbackup` produit est de **format 2** et porte **deux pièces** (manifeste lu, sans l'ouvrir ailleurs que dans le
      dossier temporaire de la story).
- [ ] **T2 — l'image** : construire l'image depuis `main` du worktree (`docker build -t gcorbaz/kesh:314 .`, sous verrou
      et `mem-guard.sh` ; elle n'écrase pas `latest`). Noter l'empreinte au journal.
- [ ] **T3 — jouer la procédure** (AC3 à AC7), **avant** d'écrire le manuel, dans un `$COPIE` neuf hors dépôt ; journal
      AC9 ; les captures utiles ; **les trois essais du refus** (AC9) ; la destruction avec `sudo` si disponible, sans
      `sudo` (conteneur jetable) sinon. **Constats à faire et à verser au journal avant d'écrire le manuel** (tirés de la
      documentation de Compose, non exécutés à la validation) :
      (a) `-p kesh-repetition` l'emporte sur un `COMPOSE_PROJECT_NAME` exporté (`config` → `name:`) ;
      (b) avec un `-f` explicite, `docker-compose.override.yml` **n'est pas** lu tant qu'il n'est pas nommé, et l'est
      quand il l'est ;
      (c) `--project-directory "$COPIE"` lit le `.env` de `$COPIE` quand la commande est lancée depuis un autre dossier
      (son dossier parent, jamais le dépôt) ;
      (d) sans `.env`, `config` échoue sur les `:?` (preuve que l'ordre des contrôles 5 et 7 importe) ;
      (e) le code que rend `POST /api/v1/_test/seed` sur une instance **sans** `KESH_TEST_MODE` (404 ou 405), pour le
      journal — le script refuse l'un et l'autre ;
      (f) la sortie de `findmnt -T "$COPIE"` et de `docker context inspect` sur la station ;
      (g) où le navigateur dépose un téléchargement, et le réglage qui le fait demander l'emplacement ;
      (h) les SHA-256 de l'export de souveraineté, source et copie : égaux ou non (AC6) ;
      (i) `docker compose ls -a` disponible dans la version installée.
      Un constat contraire à la fiche **change la procédure** avant d'écrire le manuel, et se dit au Dev Agent Record.
- [ ] **T4 — écrire la sous-section** à partir du journal, pas de la mémoire ; renvois **en prose** au manuel
      utilisateur, `\ref` seulement vers des labels du manuel d'administration (`sec:restauration-machine-neuve`,
      `sec:test-restauration`, posé ici) ; encadré AC10 ; `make fr` sous verrou ; contrôler le PDF **aplati**
      (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) : la sous-section, les neuf contrôles, l'URL au tag, la phrase « ne se
      joue jamais sur le NAS », « étapes 2 à 5 », et **aucun `??`** dans la sous-section.
- [ ] **T5 — G22** (AC11), tests rouges d'abord (M1 à M5, M8, M9), puis verts ; G19 relancée ; binaire
      `configuration_transmise` (G14, G15, G16) relancé.
- [ ] **T6 — CHANGELOG** (AC12) ; `sprint-status.yaml` ; registre.
- [ ] **T7 — gate** : `scripts/test-fast.sh` complet sous verrou au dernier commit de code, base remise à zéro d'abord ;
      `cargo fmt`, `clippy` ; l'enveloppe `repetition_fixture_script` y passe (le test shell exige `jq` : présent sur
      la station et sur `ubuntu-latest`) ; **E2E complet non requis** (aucun écran, aucun code produit ne change) — **à
      dire dans le Dev Agent Record, sans l'afficher comme exécuté**.
- [ ] **T8 — rejouer** la procédure **à partir du manuel PDF** (et non du journal) : un second passage, en suivant le
      texte tel que livré, pour attraper l'écart entre ce qu'on a fait et ce qu'on a écrit ; corriger le manuel de ce
      que ce passage révèle. **L'image** : `gcorbaz/kesh:314`, substituée à `gcorbaz/kesh:<version relevée>` ; elle
      affiche `0.14.0` à `/health` tant que les crates ne sont pas bumpées — c'est aussi la version de l'instance source
      (`cargo run` du worktree), si bien que le contrôle 9 s'éprouve tel quel ; le `pull` est remplacé par
      `docker image inspect gcorbaz/kesh:314` (image locale, absente du Hub). Les deux substitutions vont au journal.

## Dev Notes

- **La procédure contient du `rm -rf` et du `docker compose down -v`.** Ce sont les deux gestes qui détruisent ; tous
  deux sont gardés (nom de projet par `-p`, chemin absolu et nommé). Au développement, **chacun s'exécute d'abord à blanc**
  (`echo`, `docker compose config`) et sur le seul dossier de la copie ; aucun `down` ni `rm` sur un dossier que la
  story n'a pas créé. `kesh-mariadb-dev` n'est **jamais** touché (interdit, `docs/agents-orchestration.md` § 2) — c'est
  aussi pourquoi aucun essai ne pose un nom de projet `kesh`.
- **Pas de lecteur de la base de dev partagée** : la copie a sa propre MariaDB dans son volume ; seule l'instance
  **source** de la fixture emploie `kesh_314` du conteneur de dev.
- **Aucune donnée réelle** : la fixture est synthétique de bout en bout ; `~/kesh-test/` n'est pas ouvert ; le journal
  de gate-logs ne se verse pas ; les captures ne montrent que la fixture.
- **Le texte d'un manuel se vérifie contre lui-même** : T8 est le geste qui manque aux fiches qui « documentent » sans
  rejouer (les six tours de l'Epic 24, § *Le prompt d'une passe doit NOMMER le manuel*).
- **Conventions du manuel** : `\keshcommand{}`, `\keshpath{}`, `keshwarning`/`keshnote`/`keshtip`, `lstlisting[language=bash]`.
  Éviter l'emoji ; les exemples de commandes sont ceux du journal, avec les chemins de la copie en `<…>` ou `$COPIE`.
- **Version dans le manuel** : tant que `\keshVersion` vaut `0.14.0`, l'URL est au tag `v0.14.0` (G19). La phrase « format
  2, depuis la 0.15.0 » est déjà celle du manuel (31-1a/31-1b) ; au bump de release, G19 force la mise à jour de l'URL.
- **Précédent de destruction gardée** : `scripts/synology/recette.sh` (noms propres, `trap … EXIT`), à imiter pour la
  fixture (dossiers temporaires du test) et pour l'écriture de la garde.

## Hors périmètre

La sauvegarde automatique (31-1c) ; la correction des noms et du port en dur du compose (issue n° 1) ; le chemin de menu
du manuel utilisateur (issue n° 2) ; une répétition **sur le NAS** ; un E2E de la procédure (elle est une procédure
d'exploitation sur Docker, hors du périmètre de Playwright) ; la traduction (le manuel d'administration n'existe qu'en
français) ; le manuel utilisateur ; #404 et #235 (P1, nommés à la rétrospective de l'Epic 31) ; #671.

## Change Log

- 2026-10-24 — fiche créée (agent de fiche, autonomie). Prémisses du cahier des charges de la 29-7 établies au code et
  à la station (R1, R2, F5, F L9, G19) ; trois ajouts (collision de noms fixes, épinglage de la version de la copie,
  sauvegarde de sécurité de la copie) ; une absence de `sudo` constatée sur la station. Choix C-31-4-1 à 9. Validation
  P1 à lancer (Opus, lentilles R et F ; `314-validate-prompt-p1.md`).
- 2026-10-24 — **Validation P1** (Opus, deux lentilles, contexte frais, lecture seule). Bruts : **R 0/1/9/7**
  (CRITICAL/HIGH/MEDIUM/LOW), **F 0/0/10/9**. **Aucun finding réfuté** : chacun vérifié à la lecture (`grep -nF`,
  `sed -n`, `git diff --stat v0.14.0 HEAD`) — `.env.example` sans `COMPOSE_PROJECT_NAME` (rc=1) ; `recette.sh` l. 33
  l'exporte ; `docker-compose.yml` l. 12, 15, 63 (`:?`) ; `admin-manual.tex` l. 269 (`@mariadb:3306` au Synology) et
  l. 2018-2022 (étape 1 reprenant `KESH_SMTP_*`) ; `version.rs` l. 270-275 (refus seulement sur `min_required`) ; aucun
  paquet `xr` ; `Clôture d'un exercice` en l. 823 ; `.env.example` l. 232, 233, 262 (`HOST_DIR` commentées) ;
  `test_endpoints_e2e.rs` l. 97-104 (404 ou 405) ; `api-fixtures.ts` l. 105-106 (date du jour) ; `backup-restore-new-install.spec.ts`
  l. 103-105 (export par l'écran) ; `inbox.ts` l. 27, 100, 153-160 ; `user-manual.tex` l. 15 (`\keshscreenshot`) ;
  `configuration_transmise.rs` l. 2660-2662, 2757-2759, 2779-2793 ; `+layout.svelte` l. 118-124. **Renvoyés au constat
  du développement (T3), faute de pouvoir s'établir sans exécuter** : la préséance de `-p` et des `COMPOSE_*` du shell,
  l'appariement par étiquettes, la recherche dans les dossiers parents, le non-chargement de l'override sous `-f`, le
  code (404 ou 405) hors mode test, `findmnt`/`docker context` sur la station, l'emplacement des téléchargements,
  l'égalité des SHA-256 de souveraineté. **Corrections** : R1 (HIGH) et F3 → *principe directeur* (`-p kesh-repetition`,
  `--project-directory`, `-f` explicites sur chaque commande ; neutralisation de `COMPOSE_*`, `DOCKER_HOST`,
  `DOCKER_CONTEXT`, `KESH_*`, `MARIADB_*` ; `docker context`), prémisse 10, contrôles 1 et 2 élargis (code de retour,
  étiquettes, volumes) ; R2 → ordre de l'AC3 réparé (`.env` posé avant toute lecture de `config`), *La marche* alignée ;
  R3 → `config --services` décisif, `@mariadb:3306` second filet, `name: kesh-repetition` ; R4/F6 → renvoi aux seules
  étapes 2 à 5, l'étape 1 écartée en toutes lettres ; R5/F5 → exemplaire `$COPIE/source.keshbackup`, téléchargements dans
  `$COPIE/telechargements/`, original jamais touché ; R6 → dossier local non synchronisé, `findmnt` ; R7/F8 → fixture à
  trois arguments explicites, boîte propre et vide, poignée de main, date fixe, export écrit sur la route ; R8/F9 → refus
  sur tout non-2xx, test au `curl` factice (404, 405, 200) qui tue M6 ; R9/F1 → argument anti-downgrade retiré, version
  de `/health` égale exigée (contrôle 9), prémisse 7 corrigée ; R10/F7 → renvois en prose, aucun `??` au PDF ; F2 →
  `grep` des seules lignes `HOST_DIR` actives ; F4 → contrôle final refait par les noms fixes et les étiquettes ; F10 et
  R L3 → G22 (b) par position, sur les listings, relecture après override exigée ; G22 (d) chiffrée (60 lignes, 8
  commandes) ; G22 (e) nouvelle. **LOW traités** : R L1 (code de retour de `docker ps`), R L2 (chemin absolu à la garde),
  R L4 (enveloppe Rust), R L5 (condition OLICo, AC6), R L6 (« toute sauvegarde produite par la copie »), R L7 (`pull`
  avant `up`) ; F L1 (décompte de l'écart au tag), F L2 (l. 823), F L3 (captures hors dépôt ou `\includegraphics`), F L4
  (G14, G15, G16 nommées, G14 aveugle à `-p`, Hyper Backup tu), F L5 (`\label{sec:test-restauration}`), F L6 (attendus de
  l'essai Synology), F L7 (volume contrôlé), F L9 (`recette.sh` cité). **LOW consigné, non traité** : F L8 (chemin de
  menu du manuel utilisateur) → *Issues à demander* n° 2. Précisé à la T8 : l'image de `main` affiche `0.14.0` à
  `/health`. **Recomptes** (fiche après remédiation) : AC1 à AC12 (12) ; T0 à T8 (9) ; contrôles de l'AC3 : 9 (étaient
  7) ; mutations M1 à M11 (étaient 7) ; choix C-31-4-1 à 12 (étaient 9) ; issues à demander : 2 (était 1). Validation P2
  à lancer (Sonnet, R et F ; `314-validate-prompt-p2.md`).

## Dev Agent Record

*(à remplir au développement : gates réellement exécutés avec leurs chiffres, journal de répétition, constats de T3,
captures, mutations tuées, écarts à la fiche.)*
