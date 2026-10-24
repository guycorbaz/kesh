# Story 31.4 : La répétition à blanc d'une clôture (ex-29-7, documentation exécutée)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie ; remédiée le même jour après la validation P1 (Opus, lentilles R et F), après
la P2 (Sonnet ; choix C-31-4-13 et C-31-4-14), puis après la P3 (Opus ; **signal de recyclage déclaré, sortie par retrait**,
choix C-31-4-15 de l'orchestrateur : le script de fixture, son test shell, l'enveloppe Rust et la garde G22 sont retirés),
puis après la P4 (Sonnet ; choix C-31-4-17 : la source se prépare à l'écran, l'étape 0 suit le contrôle 3).
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
| Manuel : existant | `admin-manual.tex` § *Sauvegarde et restauration* : `Test de restauration périodique` (l. 1598, **sans `\label`** ; OLICo art. 10 al. 1 ; son étape 3 exige aussi un export de souveraineté dont le SHA-256 égale celui de la production) ; `Export/import d'installation via l'interface Kesh` (l. 1851) ; `Restaurer sur une nouvelle machine` (l. 2013, `sec:restauration-machine-neuve`, livrée par la 31-1b). **Son étape 1 recrée le `.env` depuis l'ancien et reprend `KESH_SMTP_*` et les dossiers** (l. 2018-2022) : l'inverse de la copie — **le renvoi ne vaut que pour ses étapes 2 à 5** (validation P1, R4/F6). **Renvoi relatif** (validation P2, F4) : l'encadré de fin de `Backup natif sur Synology DSM` (`sec:backup-dsm`, l. 1613) écrit « \textbf{Test de restauration annuel obligatoire} (cf. sous-section précédente OLICo Art. 10 al. 1) » (l. 1847, seule occurrence de « sous-section précédente/suivante ») : il désigne aujourd'hui `Test de restauration périodique` ; une sous-section insérée entre les deux le rendrait faux. Le « (cf. section précédente) » de l. 2085 vise la *section* qui précède *Mise à jour* : l'insertion d'une sous-section ne le déplace pas |
| Manuel : gestes de clôture | `user-manual.tex` : `Préparer la clôture` (`sec:preparer-cloture`, l. 792), `Les transitoires de fin d'exercice` (`sec:transitoires`, l. 685 ; bouton « Contre-passer au début de l'exercice suivant », l. 699), `Clôture d'un exercice` (l. **823**, sans label), `Réouverture d'un exercice clôturé` (`sec:reouverture-exercice`, l. 843). **Deux documents compilés séparément, aucun paquet `xr`** : un `\ref` du manuel d'administration vers ces labels rendrait « ?? » ; la convention est le renvoi **en prose** (« manuel utilisateur, \emph{…} », `admin-manual.tex` l. 886, 2149, 2213). Défaut préexistant : le chemin du menu s'écrit « Paramètres → Exercices comptables » (l. 794, 845) et « Administration → Exercices comptables » (l. 827) ; le menu le range sous *Administration* (`routes/(app)/+layout.svelte` l. 118-124) — ~~signalé, non traité ici~~ **corrigé par la story (C-31-4-13, AC13)**. *(Annoté en remédiation P2, 2026-10-24 — R6 : le même chemin s'écrit aussi dans le manuel d'administration, `admin-manual.tex` l. 1445 et 1451 ; `grep -nE 'Paramètres[^.]{0,60}Exercices' docs/manual/fr/*.tex` rend ces quatre lignes et elles seules. **Précision de fait** : l'ancien chemin n'est pas une impasse — la page *Paramètres* (`/settings`, elle-même rangée sous *Administration*, `+layout.svelte` l. 130) porte une carte *Exercices comptables* et son bouton *Gérer* (`routes/(app)/settings/+page.svelte` l. 582-590, Story 3.7) ; le défaut est que les manuels écrivent deux chemins pour un même écran, et que celui des l. 794, 845, 1445, 1451 n'est pas l'entrée du menu.)* |
| Macros du manuel | `\keshscreenshot` n'est définie que dans le préambule du manuel utilisateur (`user-manual.tex` l. 15) : ni `kesh-style.sty`, ni `admin-manual.tex` ne l'ont |
| Gardes qui lisent déjà le manuel d'administration | `crates/kesh-api/tests/configuration_transmise.rs` : **G14** (services nommés par `docker compose <verbe> <service>` ; la forme `docker compose -p <projet> …` **n'est pas lue**, angle mort écrit l. 2660-2662), **G15** (volumes cités ; le préfixe `kesh-repetition_` est accepté, l. 2762), **G16 (e)** (liste **fermée** des « Hyper Backup » hors des sections Synology, constante `HYPER_BACKUP_HORS_SYNOLOGY`, l. 2785 et suivantes : nommer Hyper Backup dans la sous-section la ferait rougir) |
| Parcours déjà automatisé | `frontend/tests/e2e/fiscal-year-closing-journey.spec.ts` (29-1b) joue : exercices 2025 et 2026, comptes `1300`/`1020`, compte bancaire lié, relevé CAMT `camt053_v04_year_end.xml`, préparation (K2 et K3 en attention), rapprochement manuel, régularisation de 250.00 au 31.12.2025, contre-passation, clôture, réouverture. **La répétition en est la version « copie du NAS »**, pas un nouveau test fonctionnel. `backup-restore-new-install.spec.ts` exporte **par l'écran** (l. 103-105, `exportByScreen`) |
| Instance de démonstration (P3, C-31-4-15 ; **établi au code en P4, C-31-4-17**) | l'assistant d'installation propose la **démonstration** (`POST /api/v1/onboarding/seed-demo`, `routes/onboarding.rs` l. 204-229, `kesh_seed::seed_demo` ; manuel d'administration l. 1409-1413) : c'est la source de la sauvegarde synthétique, exportée **par l'écran** (*Administration → Sauvegarde complète*) au format 2. **Ce qu'elle porte, lu dans `crates/kesh-seed/src/lib.rs` (l. 139-143 et 272-296)** : la société renommée, le plan comptable PME (qui contient `1300` et `2300`), **un seul exercice** (`Exercice <année courante>`), les réglages de facturation et les quatre taux de TVA, le journal d'audit. **Aucune écriture, aucun exercice suivant, aucun contact, aucune facture, aucun PDF figé, aucun document** (`grep -nE "journal_entries|contacts|invoices" crates/kesh-seed/src/lib.rs` ne rend rien). La source est donc **à compléter à l'écran** avant l'export, et c'est un geste prescrit, journalisé, de la T1 : exercice suivant, écriture de régularisation 1300/2300, au moins une pièce, et — si l'écran le permet — le PDF figé d'une facture |
| Garde G19 | `crates/kesh-api/tests/textes_coherents.rs` l. 1188 : toute URL `raw.githubusercontent.com/guycorbaz/kesh/<réf>/…` des `.tex`, du README et de `website/` doit porter `v<\keshVersion>` (au 2026-10-24 : **`v0.14.0`**, `kesh-style.sty` l. 64). Au bump de release (0.15.0), elle rougit tant que les URL n'ont pas suivi — **ce qui est voulu** et suffit à tenir l'URL de la répétition |
| Droits de la station | `sudo -n true` → *interactive authentication is required* : **un agent n'a pas `sudo`**. Les dossiers `log/`, `inbox/`, `documents/`, `backup/` d'une copie sont créés **par Docker, en root** (le conteneur tourne en root) : leur suppression est le point où la procédure doit offrir un chemin **sans `sudo`** (C-31-4-5) |
| Station, images | `gcorbaz/kesh:0.13.2`, `0.13.3` et d'autres anciennes présentes ; **aucune image portant le format 2** (la 0.15.0 n'est pas publiée). `docker build` possible depuis `main` (`Dockerfile`, Rust 1.97). Les crates sont en `0.14.0` : l'image construite depuis `main` affiche **`0.14.0` à `/health`** tout en portant le format 2, que la `gcorbaz/kesh:0.14.0` publiée **refuse** (`admin_backup/manifest.rs` l. 25) |
| Santé de la version | `GET /health` → `{"status":"ok","db":true,"version":"…"}` (`crates/kesh-api/src/routes/health.rs` l. 38-41) : c'est **là** que « la version relevée » se lit |
| Précédent de destruction gardée | `scripts/synology/recette.sh` : conteneurs et réseau de test à nom propre, `trap 'rm -rf "$W"' EXIT` (l. 43) — modèle pour l'écriture de la garde du `rm -rf` |

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
- `$COPIE` est un chemin **absolu**, posé une fois, dont le dernier élément commence par `kesh-repetition-` —
  *(précisé en remédiation P2, C-31-4-14)* et qui vit directement sous `/var/tmp` : `COPIE=/var/tmp/kesh-repetition-<AAAAMMJJ>`.
- **Avant tout**, la procédure neutralise les variables du shell qui l'emportent (`COMPOSE_*`, `DOCKER_HOST`,
  `DOCKER_CONTEXT`, `DOCKER_CONFIG`, `DOCKER_TLS_VERIFY`, `DOCKER_CERT_PATH`, `KESH_*`, `MARIADB_*`) et vérifie le contexte Docker (contrôle 1).

Le manuel peut écrire les commandes sur deux lignes (`\`), jamais les raccourcir par une fonction ou un alias : une
session où la fonction manque retomberait sur les défauts implicites (C-31-4-10 (c)). Ces comportements de Compose viennent
de sa documentation : **ils sont des constats à faire en T3, avant d'écrire le manuel** (voir T3).

*(Amendé en remédiation P2, puis P3.)* Le principe vaut pour les commandes **qui désignent un projet** : `config`, `pull`,
`up`, `ps`, `logs`, `exec`, `down`. `docker compose ls -a` (contrôle 2, qui doit voir **tous** les projets) et
`docker compose version` (contrôle 1) n'en désignent aucun et ne portent **ni** `-p` **ni** `--project-directory`. Aucune
garde automatique ne lit la forme des commandes (la garde G22 est retirée, C-31-4-15) : elle est tenue par le **journal
d'exécution** et le rejeu depuis le PDF (T8).

## Périmètre

**Dedans** : une sous-section du manuel d'administration (`.tex` **et** PDF), et un `\label` posé sur la sous-section
voisine `Test de restauration périodique` (mécanique, pour le renvoi) ; l'entrée `CHANGELOG`. **La procédure est jouée
réellement au développement** sur une sauvegarde synthétique exportée par l'écran depuis une instance de démonstration,
journal à l'appui. Aucun script, aucun test shell, aucune garde de texte nouvelle : la justesse de la procédure est portée
par son exécution journalisée et par la relecture du manuel (`.tex` et PDF) en revue (C-31-4-15).

**Dedans aussi** *(C-31-4-13)* : le chemin des exercices comptables écrit tel que le menu le range, dans les **deux**
manuels (`user-manual.tex` l. 794, 845 ; `admin-manual.tex` l. 1445, 1451 ; AC13), avec la régénération de
`user-manual.pdf` ; et le renvoi relatif « cf. sous-section précédente » de `admin-manual.tex` l. 1847, remplacé par
`\S\ref{sec:test-restauration}` (AC1). Corrections mécaniques : deux `.tex` et deux PDF, sous le seuil de la règle de
splitting.

**Dehors** (voir *Hors périmètre*) : toute modification du code du produit, des compose ou de `.env.example` (les noms
fixes sont **signalés**, non corrigés, n° 1) ; la répétition **sur le NAS** (impossible, ci-dessous) ; le manuel
utilisateur **hors du chemin des exercices** (AC13) ; **l'application** : les messages et replis qui écrivent « Paramètres
→ Exercices comptables » (AC13) ; une garde de hiérarchie des chemins de menu ; les trois autres langues (le manuel
d'administration n'existe qu'en français, `docs/manual/{de,en,it}/` ne porte que des `README.md`) ; la sauvegarde
automatique (31-1c) ; tout outillage de fixture (retiré en P3, une story ultérieure pourra le proposer).

## Critères d'acceptation

**AC1 — une sous-section au manuel d'administration, jouée avant d'être écrite.** `docs/manual/fr/admin-manual.tex`
gagne `\subsection{Répéter une clôture sur une copie}\label{sec:repetition-cloture}`, placée dans
*Sauvegarde et restauration* **juste après** `Test de restauration périodique` (l. 1598), qui y renvoie (la répétition
peut servir de test de restauration au sens de l'OLICo art. 10 al. 1, sous la condition de l'AC6) ; cette sous-section
voisine reçoit un `\label{sec:test-restauration}` pour le renvoi retour ; `Quelle méthode de sauvegarde choisir ?` n'est
pas touché. **Le renvoi relatif de l. 1847** (« cf. sous-section précédente OLICo Art. 10 al. 1 », encadré de
`sec:backup-dsm`), que l'insertion rendrait faux, devient « cf. \S\ref{sec:test-restauration}, OLICo Art. 10 al. 1 » ;
la sous-section reste à sa place, qu'elle complète *(tranché en remédiation P2, F4 : le placer ailleurs, après
`sec:backup-dsm`, l'éloignerait du test qu'elle peut remplacer ; la référence explicite ne dépend plus de l'ordre)*. Le
fragment est dans une section Synology : G16 relue au binaire `configuration_transmise` (T5). Le PDF est **régénéré** (`make fr`, sous verrou, § 5 de `docs/agents-orchestration.md`) et commité. **Chaque
commande du manuel a été exécutée** sur la station, dans l'ordre où le manuel l'écrit, et le journal de cette exécution
est versé hors dépôt (AC9). Un pas qui n'a pas pu être exécuté **ne figure pas au manuel comme acquis** : il porte la
mention « non éprouvé sur la station » et son motif (AC10).

**AC2 — le but et la portée, dits avant la première commande.** La sous-section ouvre sur : *ce qu'elle fait* (jouer sur
une copie, restaurée depuis une sauvegarde complète, les gestes de la clôture) ; *ce qu'elle ne fait pas* (elle ne
touche jamais l'installation de production ; elle ne la **remplace** pas par la copie) ; *où elle se joue* (une machine
qui ne porte pas la production, **machine Linux** *(P4, F5 : `realpath -e` et `findmnt` sont propres à Linux ; sous macOS,
`/var/tmp` est un lien et le contrôle 3 échouerait toujours — arrêt sûr, mais la procédure n'y passe jamais)*, avec Docker et Compose v2 ≥ 2.24 ; **jamais le NAS** : noms de conteneurs fixes, § État
des lieux ; un dossier **local**, jamais synchronisé ni monté depuis le NAS — `/var/tmp`, hors de `$HOME`, AC3
contrôle 3) ; *ce qu'elle suppose* (une installation en
une production qui **exporte en format 2** — Kesh 0.15.0 ou plus : le fichier `.keshbackup` emporte les pièces et
annonce leur nombre ; une production antérieure se **met d'abord à jour** (renvoi en prose à la section *Mise à jour*,
`sec:mise-a-jour`), et une sauvegarde de format 1 **ne vaut pas répétition** — ses pièces ne sont pas dans le fichier
et, dans la copie neuve, seraient toutes manquantes ; ce prérequis se vérifie **avant le contrôle 1**, avant de rien
créer : *Administration* → sauvegarde, la version affichée, ou un export d'essai dont l'en-tête `X-Kesh-Backup-Files` est
présent ; *P8, M1/M2*) ; *que la copie contient les données réelles et se détruit
à la fin* (AC7) ; et que **le fichier `.keshbackup` est un secret** (mots de passe hachés, jetons de session — rappel de
`Exporter`, sans le recopier).

**AC3 — les contrôles de la copie : les huit premiers AVANT le premier `up` (R1, R2, F5).** Toutes les commandes Compose
suivent le *principe directeur*. Dans l'ordre, chacun avec sa **sortie attendue** écrite, et **arrêt** sinon (« arrêt » est
défini une fois pour toutes à l'AC7 (d) : destruction, puis reprise au contrôle 1). **Seule exception, le contrôle 1 pour
les variables d'environnement** *(P4, R5)* : son issue est **corrective** (`unset`, contrôle refait) ; tout autre écart,
au contrôle 1 comme aux suivants, est un arrêt :
 1. **Environnement neutre, démon local.** `env | grep -E '^(COMPOSE_|KESH_|MARIADB_|DOCKER_HOST=|DOCKER_CONTEXT=|DOCKER_CONFIG=|DOCKER_TLS_VERIFY=|DOCKER_CERT_PATH=)'` ne
    rend rien *(P4, R8 : trois variables de Docker ajoutées — elles changent le contexte courant et le chiffrement du
    démon)* ; sinon `unset` de chaque variable listée, et contrôle refait (issue corrective, voir plus haut). `docker context show` est lu et
    `docker context inspect --format '{{.Endpoints.docker.Host}}'` rend une adresse `unix://` (démon de la machine),
    jamais `tcp://` ni `ssh://` (un contexte distant ferait jouer la copie sur un autre hôte, le NAS compris).
    *(Ajouté en remédiation P2, R1/F8.)* `docker compose version --short` rend **2.24.0 ou plus** : en deçà, `!override`
    (contrôle 8) n'est pas garanti, et une fusion des listes de ports garderait la publication `80:80` sur toutes les
    interfaces ; sinon arrêt. Commande sans `-p` ni `--project-directory` (elle ne désigne aucun projet, *principe
    directeur*).
 2. **Hôte sans production ni reste d'une répétition.** `docker ps -a --format '{{.Names}}'` **réussit** (code 0 : un
    démon injoignable « ne rend rien » aussi) et ne contient ni `kesh-api` ni `kesh-mariadb` (noms exacts ; la pile
    jumelle entrerait en collision, et le compose Synology partage le nom `kesh-api`) ; `docker ps -a -q`,
    `docker volume ls -q` et `docker network ls -q`, chacun avec `--filter label=com.docker.compose.project=kesh-repetition`,
    ne rendent rien (un volume resté d'une répétition non détruite garderait l'ancien mot de passe — F L9) ;
    `docker compose ls -a` ne liste pas `kesh-repetition` (sans `-p` : il doit voir tous les projets).
 3. **Dossier de la copie : neuf, sous `/var/tmp`, ni lien ni reste.** *(Réécrit en remédiation P2 — C-31-4-14, R5/F6,
    R8, F11.)* `COPIE=/var/tmp/kesh-repetition-<AAAAMMJJ>` — chemin **fixé** : sous `/var/tmp`, hors de `$HOME`, donc hors
    de tout dossier qu'un client de synchronisation surveille (Synology Drive Client, par défaut `~/SynologyDrive` : un
    tel dossier vit sur un système de fichiers local, que `findmnt` ne distingue pas — R5/F6) ; jamais un dossier nommé
    `kesh`. Puis, dans l'ordre, chaque commande avec sa sortie attendue :
    - `mkdir -m 0700 "$COPIE"` — **sans `-p`** : il échoue si le dossier existe (reste d'une répétition, ou dossier
      posé par un autre), et c'est le refus voulu ; `0700` parce que `/var/tmp` est partagé entre utilisateurs et que la
      copie contiendra les données réelles ;
    - `test ! -L "$COPIE"` réussit (le chemin n'est pas un lien symbolique : `docker run -v "$COPIE":…` le suivrait) ;
    - `[ "$(realpath -e -- "$COPIE")" = "$COPIE" ]` réussit (aucun lien ni `..` dans le chemin, `/var/tmp` compris :
      s'il est lui-même un lien sur la machine, le contrôle échoue, et c'est un arrêt sûr — constat T3) ;
    - `test -z "$(ls -A -- "$COPIE")"` réussit — dossier **vide**, vérifié **avant** le premier fichier qu'on y pose ;
    - `mkdir -m 0700 "$COPIE/telechargements"` *(P4, R4 ; sans `-p`, comme le précédent)* — **après** la vérification de
      vacuité, **avant** l'étape 0 : les exports de la production s'y enregistrent directement (R3), puis les
      téléchargements du contrôle 4 ; l'exemplaire `source.keshbackup` n'est posé qu'**après le contrôle 9**, AC4 ;
    - second filet seulement : `findmnt -T "$COPIE"` rend un système de fichiers local (ni `cifs`, ni `nfs*`, ni
      `fuse.*`) — il écarte un `/var/tmp` monté depuis le réseau, **pas** un dossier synchronisé, et le manuel le dit.
    Les pièces réelles restaurées et la sauvegarde de sécurité vivront dans ce dossier : synchronisé, il les ferait
    remonter vers le NAS et ses sauvegardes. Le fichier `.keshbackup` de la production s'y **enregistre directement**
    à l'étape 0 *(P5, T1 : repli par `mv`, AC4)*.
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
    `COMPOSE_PROJECT_NAME` (le `-p` le porte). *(Précisé en remédiation P2, R10 LOW.)* Les valeurs s'écrivent **dans le
    fichier**, en éditant la ligne existante de `.env.example` (`KESH_COOKIE_SECURE=true` l. 155 devient `false` ;
    *(P4, F4)* `MARIADB_ROOT_PASSWORD` et `MARIADB_PASSWORD` y sont **commentées** (`#MARIADB_ROOT_PASSWORD=` l. 374,
    `#MARIADB_PASSWORD=` l. 377) : « éditer la ligne » veut dire **ôter le `#` et poser la valeur**, non ajouter une
    ligne ; `MARIADB_DATABASE` et `MARIADB_USER`, déjà actives, ne bougent pas ; `DATABASE_URL=<EDIT: …>` (l. 52) reste
    telle quelle, sans effet — le compose autonome la construit lui-même), jamais
    en ajoutant une seconde ligne pour la même clé (laquelle l'emporte : constat T3 (k)) ni par un `export` dans le shell,
    que le contrôle 1 vient de neutraliser.
 6. **Aucune ligne `HOST_DIR` active.** `grep -nE '^[[:space:]]*KESH_[A-Z_]*HOST_DIR[[:space:]]*=' "$COPIE/.env"` ne rend
    rien (les trois lignes **commentées** de `.env.example`, l. 232, 233, 262, ne comptent pas ; un `grep 'HOST_DIR'` nu
    les rendrait et apprendrait au lecteur à passer outre un contrôle — validation P1, F2).
 7. **Première lecture de la configuration** (le `.env` est posé : les `:?` sont satisfaits). Avec le seul premier `-f` :
    `config --services` rend **exactement** `mariadb` et `kesh-api`, en **ensemble** (l'ordre de sortie n'est pas établi —
    validation P2, R10) — **c'est le contrôle décisif** : le compose Synology
    n'a qu'un service ; *(P3, R-L4)* la sortie porte `KESH_SMTP_HOST` **vide** (la copie n'envoie aucun courriel) ; `config` rend en première ligne `name: kesh-repetition` ; sa `DATABASE_URL` contient
    `@mariadb:3306` (**second filet seulement** : le manuel prescrit cette forme au Synology aussi, l. 269) ; tous ses
    `source:` sont sous `$COPIE`, hormis le volume nommé `kesh-mariadb-data`.
 8. **Override, puis configuration relue.** `$COPIE/docker-compose.override.yml` : `image: gcorbaz/kesh:<version relevée>`
    et `ports: !override ["127.0.0.1:<port libre>:80"]` — *(P4, F3)* le port est **libre, supérieur à 1023 et différent
    de 80** (un port 80 « libre » satisferait le premier critère et violerait le contrôle ci-dessous) — la copie n'écoute que sur la boucle locale (C-31-4-6) ; et
    `restart: "no"` sur les deux services *(ajouté en remédiation P2, R9 LOW : une copie oubliée ne repart pas avec les
    données réelles au prochain démarrage de Docker ; prise en compte à constater en T3)*. Les commandes portent
    désormais les **deux** `-f`. Le contrôle 7 est **refait sur la sortie finale**, et en plus : l'`image:` est celle
    épinglée ; *(précisé en remédiation P2, R1/F8)* le service `kesh-api` porte **une seule** publication de port, avec
    `host_ip: 127.0.0.1` et `published: "<port choisi>"` ; **dans le bloc `ports:` de `kesh-api`** *(P3, R1)*, aucun
    `published: "80"` et aucun `host_ip: 0.0.0.0` ; le service `mariadb` **aucune** publication. ⚠️ Le `0.0.0.0` de
    `KESH_HOST: 0.0.0.0` (défaut du compose, adresse d'écoute **dans** le conteneur) **n'est pas une publication** et se
    trouve dans toute sortie correcte : le contrôle ne cherche **pas** `0.0.0.0` dans la sortie entière, et le manuel dit
    de **ne pas** poser `KESH_HOST` au `.env` (constat T3 (p)). Deux publications, ou une sans `host_ip`, signifient que
    l'override n'a pas écrasé la liste : arrêt.
 9. **Seulement alors : l'image, le démarrage, la version.** `pull kesh-api` (une version mal écrite échoue ici, au lieu
    d'une construction tentée dans `$COPIE`, `build: context: .`) ; `up -d` ; `curl -fsS http://127.0.0.1:<port>/health` :
    `version` **égale exactement** la version relevée à l'étape 0 — l'import n'y oblige pas (prémisse 7) ; sinon `down -v`
    et arrêt. *(Ajouté en remédiation P2, R7 LOW.)* Les trois filtres du contrôle 2
    (`--filter label=com.docker.compose.project=kesh-repetition` sur `ps -a -q`, `volume ls -q`, `network ls -q`)
    rendent cette fois **chacun au moins une ligne** : c'est ce qui donne un sens au « rien » du contrôle final (AC7 (c)),
    qu'un filtre mal écrit rendrait aussi.
 Le manuel écrit ces neuf contrôles comme une **liste numérotée**, dans cet ordre.

**AC4 — la restauration, par renvoi et non par recopie.** Après le contrôle 9, le manuel **renvoie aux étapes 2 à 5** de
`Restaurer sur une nouvelle machine` (§ `sec:restauration-machine-neuve`) pour `/setup`, le lien « Restaurer une
sauvegarde », l'import et la reconnexion, et écrit : « **l'étape 1 (le `.env`) ne vaut pas ici** : le `.env` de la copie
est celui des contrôles 4 à 6, rien n'est repris de l'ancien ». Il ne redit que ce qui est **propre à la copie** :
l'adresse (`http://127.0.0.1:<port>`) ; le fichier à importer — le `.keshbackup` pris sur la production par
**Administration → Sauvegarde complète** —, dont la répétition travaille sur **un exemplaire** `$COPIE/source.keshbackup`
(mode 0600), **posé seulement après le contrôle 9**, juste avant l'import. *(P5, T1 : le secret réel entre dans
`/var/tmp` **dès l'étape 0**, dans `$COPIE/telechargements/` (0700) — avant les contrôles 4 à 9. Ce qui tient lieu de
garantie, ce n'est pas son absence de `/var/tmp`, mais **la destruction de `$COPIE` à tout arrêt après l'étape 0**
(AC7 (d)).)* **Le geste qui pose l'exemplaire**, après le contrôle 9 :
`mv "$COPIE/telechargements/<fichier>.keshbackup" "$COPIE/source.keshbackup"` puis `chmod 0600 "$COPIE/source.keshbackup"`
(`mv`, et non copie : le fichier ne subsiste pas sous deux noms). L'original, où qu'il soit conservé (hors de
`$COPIE`), n'est ni déplacé ni supprimé par la procédure ; **seul le repli du constat (s)** — un exemplaire déposé
par le navigateur **hors de `$COPIE`**, dans son dossier de téléchargements — est lui aussi **déplacé** (`mv`) dans
`$COPIE/telechargements/` ; le manuel dit en toutes lettres que cet exemplaire **n'est pas une sauvegarde à conserver**
et sera détruit avec la copie : qui veut le garder en fait d'abord une copie ailleurs. **Tout fichier sensible** — l'exemplaire,
les **exports de souveraineté** de la production et de la copie, tout fichier téléchargé depuis la copie (un PDF de
facture) — **s'enregistre directement dans `$COPIE/telechargements/`, si le navigateur le permet** (il demande l'emplacement,
constat (g)/(s) ; l'étape 0 se joue **après** les contrôles 1 à 3 précisément pour que ce dossier existe, *P4, R3*) ; si le
navigateur a tout de même déposé un exemplaire dans son dossier de téléchargements, il est **déplacé** (`mv`) sans
attendre, et l'arrêt (AC7 (d)) le retire. Aucun ne reste dans le dossier de téléchargements, que Synology Drive Client
peut surveiller. La suite : la lecture du bilan (« pièces restaurées / manquantes ») ; la
reconnexion avec **les identifiants de la production**. **Critère de réussite, tel que le manuel le dit au lecteur**
*(P4, R1/F1, C-31-4-17 ; séparé en P5, T2)* : le bilan annonce **0 pièce manquante**, et **autant de pièces restaurées
que l'export de la production en a annoncé** (l'en-tête `X-Kesh-Backup-Files` ou le message de l'écran d'export,
relevé à l'étape 0) ; et la lecture de l'étape 5 de *Restaurer sur une nouvelle machine* est faite telle quelle :
**une écriture récente au journal, et la liste des factures** que la source portait y figurent. Ce critère vaut pour
toute installation, y compris une production sans aucune pièce (aucun document importé) : le manuel ne présume rien
de *N*. **La répétition suppose une production qui exporte en format 2** (Kesh 0.15.0 ou plus), dont la sauvegarde
annonce son compte de pièces ; le prérequis est vérifié avant le contrôle 1, et une production antérieure se **met
d'abord à jour** — le manuel le dit en tête de la procédure (AC2).
Une sauvegarde de format 1 ne vaut pas répétition : ses pièces ne sont pas dans le fichier et seraient toutes
manquantes *(P6, T2-bis ; corrigé en P7 : le critère réduit d'abord écrit était inatteignable)*. **Ce que le développement exige en plus, et que le manuel ne dit pas** : la sauvegarde
synthétique de la T1 porte *N* ≥ 1 (au moins une pièce y est entrée par l'écran) ; *N* = 0 y serait un **échec de la
préparation de la source** — « 0 manquante » serait alors vrai quelle que soit la qualité de la restauration des
pièces —, et la copie contient en outre l'exercice suivant et l'écriture de régularisation de la T1. **Étapes 2 à 5
de *Restaurer sur une nouvelle machine* : ce qui s'applique ici** : `/setup`, le lien de restauration, l'import, la
reconnexion et le bilan s'appliquent, ainsi que les vérifications de l'étape 5 ; **au développement**, celle d'**un
document fournisseur** porte sur la pièce de la T1, et celle du **PDF d'une facture émise** ne s'applique que si la T1
en a figé un (sinon « non éprouvé », AC10).
Les commandes que ces étapes 2 à 5 écrivent pour l'installation ordinaire (`docker compose logs kesh-api`, étape 5) se
jouent, dans la copie, **selon le *principe directeur*** : la sous-section les redonne sous cette forme plutôt que de
laisser le lecteur recopier la forme courte contre le dossier courant.

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
    fiche dit pourquoi** : le créer (c'est un point que la préparation signale). *(P5, T2 : au manuel, ce geste vaut pour
    l'installation du lecteur ; il y suppose une écriture 1300/2300.)* **Au développement**, le jeu de départ de ce geste
    est celui que la T1 a créé dans la source (exercice suivant et écriture 1300/2300 du dernier jour de l'exercice
    courant) : la démonstration n'en porte aucun (C-31-4-17).
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
l'export de souveraineté et la comparaison de son SHA-256 avec celui de la production — celui-ci produit **sur la
production, à l'étape 0, avant le `.keshbackup`**, et **enregistré dans `$COPIE/telechargements/`** (l'étape 0 se joue
après le contrôle 3, AC4 ; son SHA-256 se note au procès-verbal) ; l'export de la copie y va aussi. Si le constat de T3 montre que les deux empreintes ne
peuvent pas être égales par construction, le manuel le dit et ne prétend pas que la répétition vaut ce test. Aucune
valeur chiffrée de l'installation de Guy n'entre dans le manuel.

**AC7 — la copie ne laisse rien derrière elle.** (a) **Ce qui est détruit**, nommé : les conteneurs, le volume
`kesh-repetition_kesh-mariadb-data`, le réseau, et **le dossier `$COPIE` entier** — qui contient l'exemplaire
`source.keshbackup`, le `.env`, **toute sauvegarde produite par la copie** (`backup/kesh-pre-import-*` et toute autre), les
**exports de souveraineté** (production et copie) et les téléchargements de `$COPIE/telechargements/`. L'original de la
sauvegarde n'est jamais touché. (b) **La destruction**, dans cet ordre : le **contrôle 1 refait** *(P3, R-L1 : le contexte
Docker a pu changer depuis le `up` ; sur un hôte distant, `docker run -v` créerait le dossier sur le NAS)* ; `down -v`
selon le *principe directeur* (`-p kesh-repetition`, `--project-directory`, les deux `-f`) ; puis la suppression de
`$COPIE` — dossiers `log/`, `inbox/`, `documents/`, `backup/` **créés par Docker, en root** : deux chemins écrits et
éprouvés, **`sudo rm -rf "$COPIE"`** et, **sans `sudo`**, `docker run --rm -v "$COPIE":/copie --entrypoint sh <image
mariadb déjà tirée> -c 'rm -rf /copie/log /copie/inbox /copie/documents /copie/backup'` puis `rm -rf "$COPIE"`. Chaque
`rm -rf` et le `docker run -v` sont précédés de la **garde**, écrite **exactement** ainsi et **dans le même listing** que
le geste qu'elle protège ; **les deux gestes du chemin sans `sudo`** (le `docker run … rm -rf /copie/…` puis le
`rm -rf "$COPIE"`) **se trouvent dans un seul `if`** *(P4, R6 : une garde répétée, ou un seul geste protégé sur deux,
laisserait ambigu ce que l'autre exécute)* :

```sh
if [ -n "$COPIE" ] \
   && [ "$(dirname -- "$COPIE")" = /var/tmp ] \
   && case "$(basename -- "$COPIE")" in kesh-repetition-*) true ;; *) false ;; esac \
   && [ -d "$COPIE" ] && [ ! -L "$COPIE" ] \
   && [ "$(realpath -e -- "$COPIE")" = "$COPIE" ]; then
  <geste>
else
  echo "refus : COPIE n'est pas le dossier de la copie"
fi
```

*(P5, T6 : dans le chemin sans `sudo`, `<geste>` est `docker run … rm -rf /copie/…` **`&&`** `rm -rf "$COPIE"` — chaîné
par `&&`, non par une ligne seule : si le `docker run` échoue, le `rm -rf "$COPIE"` ne part pas. **Avant le premier
`up`**, aucun dossier de root n'existe dans `$COPIE` : `rm -rf "$COPIE"` gardé suffit, sans `docker run`.)*

- **La branche `*) false`** du `case` est indispensable *(P3, R2/F2)* : un `case` sans motif correspondant rend le code
  **0**, donc « vrai » ; écrit avec la seule branche `kesh-repetition-*)`, il laisserait passer tout dossier enfant de
  `/var/tmp` (le `docker run -v` y tourne en root). Le manuel le dit.
- `[ -n "$COPIE" ]` — non vide ; `dirname` égal à `/var/tmp` — directement sous `/var/tmp` (donc **absolu** : un chemin
  relatif passé à `-v` crée un volume nommé, et le `rm` tourne à vide) ; `-d` et `! -L` — un dossier existant, **pas un
  lien** (le `-v "$COPIE":/copie` suivrait le lien) ; `realpath -e` égal au chemin écrit — les tests de texte ne lisent que
  la chaîne, ils acceptent un `/var/tmp` qui serait lui-même un lien. Une barre finale fait échouer la comparaison : refus
  sûr, le manuel dit d'écrire `COPIE` sans elle.
- **Ni `exit` ni `return`** (le premier fermerait le terminal du lecteur, le second n'a de sens que dans une fonction) ;
  `fi` sur sa propre ligne (pas de `else echo … fi` en ligne, P3, F8).
- Un `rm -rf "$VAR"` non gardé est la faute dangereuse type ; le dev éprouve la garde par **quatre cas** (AC9 (iii)) :
  variable vide, chemin relatif inexistant, lien symbolique, **nom faux**.

(c) **Contrôle final** écrit : le contrôle 1 est vrai ; le contrôle 2 refait (ni `kesh-api` ni `kesh-mariadb` parmi les
noms ; conteneurs, volumes et réseaux filtrés par l'étiquette `kesh-repetition` : rien — ces mêmes filtres ayant rendu des
lignes au contrôle 9) ; `test ! -e "$COPIE"` ; et **aucun exemplaire de la répétition** (`.keshbackup`, **ZIP de
souveraineté**, PDF) ne subsiste dans le dossier de téléchargements du navigateur. (d) **Arrêt** *(P3, R5 ; précisé en P4,
R2/R3)* : « arrêt », dans tout le manuel, veut dire : détruire (b) puis, si l'on veut reprendre, **repartir du contrôle 1**
avec une nouvelle date — jamais un `mkdir -p`, jamais un `rm` improvisé. Quatre précisions *(P5, T4 : les « trois
précisions » de P4 ne couvraient ni l'arrêt au contrôle 3, ni un dossier resté sans objet Docker)* :
 - **Arrêt avant que `$COPIE` existe** (contrôles 1 et 2, les plus probables) : il n'y a **rien à détruire** — l'étape 0
   n'a pas eu lieu (elle suit le contrôle 3), aucun fichier sensible n'existe encore. On corrige la cause (hôte, contexte)
   ou on abandonne.
 - **Arrêt au contrôle 3** (le `mkdir` a réussi, l'étape 0 n'a pas eu lieu) : `$COPIE` existe et est **vide** ; on le
   supprime par (b) sous garde, sans `down -v`. Si c'est le `realpath -e` qui a échoué, la garde **refuse** aussi (elle comprend le même
   `realpath -e`) : le dossier vide est alors laissé tel quel, **le refus est consigné au journal**, et c'est la
   cause (un `/var/tmp` qui serait un lien) qu'on traite, non le dossier.
 - **Arrêt après l'étape 0** : (b) détruit `$COPIE`, donc ce qui y a été enregistré (avant le premier `up`, **contrôles
   4 à 8**, aucun objet Docker n'existe : (b) se réduit à la suppression de `$COPIE` sous garde, sans `down -v`) ; si un
   exemplaire a atterri dans le dossier de téléchargements du navigateur (repli du constat (s)), il est **retiré à la
   main** de ce dossier (nommément, fichier par fichier) avant de repartir.
 - **Reste d'une répétition antérieure** (le contrôle 2 arrête : copie oubliée, volume « jamais nettoyé » ; **ou le
   `mkdir` du contrôle 3 refuse** parce que le dossier existe alors que le contrôle 2 n'a rien vu faute d'objet Docker)
   *(P4, R2 ; P5, T4)* :
   (i) `ls -d /var/tmp/kesh-repetition-*` (lecture seule) ; **si le dossier de l'ancienne copie existe** (un à la fois),
   on **affecte `COPIE`** à ce dossier, on refait le contrôle 1, et on applique (b) **avec sa garde**, `-f` visant les
   fichiers qu'il contient ; **si ce `down -v` échoue** (`.env` de l'ancienne copie absent ou incomplet : les `:?` du
   compose), on **passe à (ii)** pour les objets, puis (b) pour le dossier ; (ii) **si le dossier a disparu** (nettoyage à 30 jours de `/var/tmp`) alors que des objets
   subsistent, geste nommé, journalisé et limité par l'étiquette : `docker rm -f`, `docker volume rm` et
   `docker network rm` appliqués **aux seuls identifiants** rendus par les trois filtres
   `--filter label=com.docker.compose.project=kesh-repetition` du contrôle 2 (jamais un nom de volume ou de
   conteneur écrit à la main, jamais un `prune`). Ce geste (ii) est **éprouvé à blanc seulement** (`echo`, sur la copie
   vivante avant sa destruction : les filtres désignent exactement ses objets, constat (u)) ; son exécution réelle est
   « non éprouvée » (AC10). Puis on repart du contrôle 1.
 Une copie oubliée **reste** : `/var/tmp` survit au redémarrage (nettoyage à 30 jours sur la station) et le volume de la
 base n'est jamais nettoyé ; le manuel le dit.

**AC8 — *retiré en P3 (C-31-4-15)*.** Le script de fixture, son test shell et son enveloppe Rust sont retirés (le HIGH de
la P3 — la fixture ne pouvait pas figer un PDF sur le preset de test — et la majorité des MEDIUM de P2 et P3 portaient sur
cet échafaudage). **À sa place** : la sauvegarde synthétique est **exportée par l'écran** (*Administration → Sauvegarde
complète*, format 2) depuis une instance de démonstration (État des lieux, ligne « Instance de démonstration »), jouée en
T1. Aucune donnée réelle, aucun relevé de `~/kesh-test/`.

**AC9 — le journal d'exécution, hors dépôt.** `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-<AAAAMMJJ>.log` :
chaque commande du manuel **et sa sortie**, dans l'ordre, horodatée, avec le résultat des contrôles de l'AC3, les
constats de T3, **ce que la sauvegarde source portait** (relevé de T1, nombre de pièces du bilan de l'import) et **trois
essais volontaires du refus**, aucun ne lançant de `up` :
 (i) **le compose Synology** : `docker-compose.prod.yml` du tag, enregistré sous le nom `docker-compose.yml` dans un
     second dossier `/var/tmp/kesh-repetition-essai-<AAAAMMJJ>` (créé par le contrôle 3, comme la copie), avec un `.env`
     **synthétique** dont la `DATABASE_URL` vaut `mysql://kesh:essai@mariadb:3306/kesh` — ce compose n'a aucun `:?`
     (`grep -c ':?' docker-compose.prod.yml` → 0), la `DATABASE_URL` suffit à ce que `config` aboutisse et que seul le
     nombre de services décide — attendu : contrôle 7, `--services` rend `kesh-api` seul → **arrêt** ; le `grep` de
     `@mariadb:3306` **passe** (preuve qu'il ne suffit pas) ; les `source:` relatifs **passent** ; le dossier de l'essai est
     ensuite **détruit** par la garde de l'AC7 **appliquée à ce dossier** (le journal dit la variable employée : `COPIE`
     réaffectée au dossier de l'essai, puis **rétablie**) ;
 (ii) **un nom de projet hérité** : `export COMPOSE_PROJECT_NAME=kesh-essai-314` (jamais `kesh`, ni le nom d'une pile
      existante) — attendu *(P4, R5)* : contrôle 1 le **liste** (constat ; son issue est corrective) ; la variable
      restant posée, `config` avec `-p kesh-repetition` rend `name: kesh-repetition` (constat de préséance) ; puis
      `unset`, et le contrôle 1 refait **ne rend rien**. Ce `config` se joue sur `$COPIE` **après le
      contrôle 5**, dont le `.env` satisfait les `:?` ;
 (iii) **la garde du `rm -rf`, quatre cas** : `COPIE` vide ; relatif **et inexistant** (`kesh-repetition-absent-314`) ; un
      **lien symbolique** `/var/tmp/kesh-repetition-lien-<AAAAMMJJ>` vers un dossier vide créé pour l'essai ; et un
      **nom faux** *(P3, R2/F2)* : un dossier **vide existant** `/var/tmp/essai-sans-prefixe-<AAAAMMJJ>`, créé pour
      l'essai, que la garde doit **refuser** — chaque fois depuis un dossier courant **vide** créé pour l'essai sous
      `/var/tmp`, jamais le worktree ni le dépôt, et le geste d'abord sous la forme `echo rm -rf …` — attendu : la garde
      arrête, rien n'est supprimé ; les restes de l'essai sont ensuite retirés (`rm -- "$LIEN"` sans barre finale, puis
      `rmdir` de chaque dossier créé pour l'essai, dossier courant compris). *(P4, F2, R6.)* **`COPIE` est rétablie après
      chaque cas** et sa valeur **relue au journal** avant le cas suivant et avant le contrôle final de l'AC7 (c) : une
      `COPIE` laissée sur le « nom faux » ferait passer `test ! -e "$COPIE"` à tort alors que la vraie copie subsiste.
      Les quatre cas sont le minimum ; **deux refus de plus se constatent dans le même passage** : une **barre finale**
      (`COPIE=<dossier de l'essai>/`, sur un dossier de l'essai existant) et **`/var/tmp` lui-même** ; le suffixe vide
      (`/var/tmp/kesh-repetition-`) passe le `case`, puis la garde le **refuse** au `-d` tant que ce dossier de ce nom exact
      n'existe pas *(P5, T5)*. Le journal dit les six sorties.
Le journal **ne se verse pas au dépôt** (il peut contenir des chemins et des empreintes). Les **captures** (thème clair
seulement) de l'écran de restauration réussie, du panneau *Préparer la clôture* et de la confirmation de
contre-passation sont prises à l'exécution et **restent hors dépôt**, avec le journal : le manuel d'administration n'a pas
de macro de capture (`\keshscreenshot` est propre au manuel utilisateur) ; s'il en montre une, c'est par
`\includegraphics` dans une `figure`, la capture allant dans `docs/manual/fr/screenshots/`. Les données des captures sont
celles de l'instance de démonstration.

**AC10 — ce qui ne s'éprouve pas sur la station, écrit comme tel.** Le manuel contient un court encadré
(`keshnote`) *Ce que cette procédure n'a pas éprouvé* : **le Synology** (DSM, Container Manager, les ACL des dossiers
partagés, le compose `prod` à réseau externe, la copie du `.keshbackup` depuis le NAS vers la station — gestes décrits
par `Restaurer depuis elle`, dont la procédure est celle de la 31-1b), la **taille réelle** de l'installation (les temps
et l'espace disque de la démonstration ne valent pas ceux de l'installation de Guy ; *(P3, R-L3)* le **moteur Docker**
éprouvé, nommé au journal, Docker Engine en root — sous Docker Desktop, rootless ou snap, le partage de `/var/tmp` avec le
démon n'est pas établi ; le manuel renvoie aux ordres de grandeur
déjà écrits : « jusqu'à environ trois fois la taille du fichier »), et **l'image publiée** si la procédure a été jouée
sur une image construite depuis `main` (C-31-4-4 : au développement, `gcorbaz/kesh:314` affiche `0.14.0` à `/health`
tant que les crates ne sont pas bumpées, et le `pull` est remplacé par `docker image inspect`). *(P4, C-31-4-17.)* Y figurent aussi, **s'ils se confirment à l'exécution** : le **PDF figé d'une facture** (« non éprouvé »
si la T1 n'a pas pu en figer un) et l'**exécution réelle** du geste (ii) de l'AC7 (d) (reste sans dossier), ainsi que le `down -v` du geste (i) lorsque
l'ancienne copie n'a plus que des `-f` partiels *(P5, T4)*. **Comment la procédure
les couvre** : (i) elle ne s'exécute **jamais sur le NAS** ; le seul geste Synology qu'elle contient est la récupération
du fichier, déjà documentée ; (ii) les contrôles de l'AC3 sont **indépendants de l'hôte** (ils lisent l'environnement,
le démon et la sortie de `docker compose config`) ; (iii) le **rejeu sur l'image publiée** est demandé à la release (voir
*Actions demandées*, n° 3). L'encadré **ne nomme pas Hyper Backup** (G16 (e), liste fermée).

**AC11 — *retiré en P3 (C-31-4-15)*.** La garde G22 (présences de jetons dans la sous-section, absence de
`KESH_*_HOST_DIR=`, constante `JETONS_G22`, test de retrait en mémoire) est retirée avec ses lettres (a) à (e) et ses
mutations. Ce qu'elle devait tenir — l'ordre des contrôles, la forme des commandes, la justesse des gestes — est porté par
le **journal d'exécution réelle** (AC9, T3), le **rejeu depuis le PDF** (T8) et la **relecture du manuel en revue**, non
par une garde lexicale. **Restent** les gardes existantes qui relisent la sous-section : **G19** (URL au tag), **G14**
(services nommés ; la forme `-p` n'est pas lue, angle mort écrit), **G15** (volumes, préfixe `kesh-repetition_` accepté) et
**G16** (liste fermée des « Hyper Backup » hors sections Synology) — le binaire `configuration_transmise` et la garde G19
de `textes_coherents.rs` se relancent au gate ciblé (T5).

**AC12 — CHANGELOG.** Entrée `Ajouté` dans la section `[0.15.0] — Non publié` : la répétition de clôture sur une copie,
au manuel d'administration, jouée sur une sauvegarde complète. Et une entrée dans une rubrique `### Corrigé` à créer dans
cette section, **entre `Modifié` et `Interne`** *(P3, F12)* (elle n'en a pas encore ; `CHANGELOG.md` l. 13, 40, 60) : **les
manuels** nomment **Administration → Exercices comptables** (AC13), sur le modèle de l'entrée de #585 en 0.14.0
(`CHANGELOG.md` l. 106) — le libellé dit « les manuels », **non** « et les messages » (AC13), sans lien d'issue, aucune
n'étant ouverte (C-31-4-13). Aucune autre entrée.

**AC13 — le chemin des exercices comptables, écrit tel que le menu le range** *(ajouté en remédiation P2 — C-31-4-13,
élargi par R6).* Les quatre occurrences « \emph{Paramètres} → \emph{Exercices comptables} » — `user-manual.tex` l. 794
(colonne \emph{Actions}) et l. 845 (bouton \emph{Réouvrir}), `admin-manual.tex` l. 1445 et l. 1451 (bouton
\emph{Réouvrir}) — deviennent « \emph{Administration} → \emph{Exercices comptables} », le reste de chaque phrase
inchangé. Contrôle, à zéro résultat : `grep -nE 'Paramètres[^.]{0,60}Exercices comptables' docs/manual/fr/*.tex` ; et,
sur les **deux** PDF régénérés (`user-manual.pdf`, `admin-manual.pdf`), `pdftotext <pdf> - | tr '\n' ' ' | tr -s ' ' |
grep -c 'Paramètres → Exercices comptables'` rend `0` pour chacun (T4). La phrase de conventions de `user-manual.tex`
l. 59 (« \emph{Paramètres} → \emph{Utilisateurs} » donné en exemple) et les chemins internes à la page *Paramètres*
(`Paramètres → Modèles d'e-mail`, `Paramètres → Clés API`, … : des cartes de cette page) ne sont **pas** touchés : ils
désignent bien la page *Paramètres*. Aucune garde de hiérarchie n'est ajoutée (C-31-4-13). **Résidu nommé, hors périmètre**
*(P3, R7/F4, C-31-4-15)* : les **dix sites de l'application** qui écrivent le même chemin — `messages.ftl` des quatre
locales (`error-fiscal-year-missing` et `opening-balances-locked-no-fiscal-year`, ×4 = 8 lignes), `notify.ts` l. 110 et
`settings/opening-balances/+page.svelte` l. 377 — **restent** : la page *Paramètres* porte une carte *Exercices comptables*,
le chemin est un détour valide, et corriger les messages relève du code du produit. Le contrôle de l'AC13 est donc borné
aux **manuels** (`docs/manual/fr/*.tex`), non au dépôt entier.

## La marche prescrite (squelette ; le texte exact est écrit en exécutant)

0. *(Se joue **après** les contrôles 1 à 3 de l'item 1 — P4, R3.)* Relever sur la production : la version (`GET /health`,
   champ `version`) ; si la répétition doit valoir test de restauration (AC6), l'export de souveraineté **et son
   SHA-256** ; **puis** produire le `.keshbackup` (*Administration → Sauvegarde complète*). Ces fichiers
   **s'enregistrent directement dans `$COPIE/telechargements/`, si le navigateur le permet** (AC4, constat (s) ; sinon
   repli par `mv`) : ils ne doivent pas rester dans le dossier de téléchargements du navigateur, que Synology Drive
   Client peut surveiller.
1. Contrôles 1 à 3 (version de Compose et environnement, hôte, dossier neuf sous `/var/tmp`, vide, puis
   `$COPIE/telechargements/`), **puis l'étape 0**. **L'exemplaire `$COPIE/source.keshbackup` n'est pas encore posé**
   (AC4).
2. Contrôles 4 à 8 : téléchargements au tag → `.env` rempli → lignes `HOST_DIR` → première lecture de `config` →
   override et relecture.
3. Contrôle 9 : `pull`, `up -d`, version à `/health` ; **seulement alors**, poser l'exemplaire `$COPIE/source.keshbackup`
   (0600) ; ouvrir `http://127.0.0.1:<port>/setup` ; **renvoi aux étapes 2 à 5** de *Restaurer sur une nouvelle machine*
   (AC4).
4. Le cas échéant, l'export de souveraineté de la copie et son SHA-256 (AC6) ; puis les quatre gestes (AC5) et le constat
   de chacun.
5. Le procès-verbal (AC6).
6. Destruction (AC7 (b)) et contrôle final (AC7 (c)). **Arrêt** à toute étape : AC7 (d).

## Mutations à prévoir

*(Table réécrite en remédiation P3 — C-31-4-15 : toutes les mutations qui visaient G22, la fixture, son test shell ou son
enveloppe sont **retirées**, leurs numéros gardés pour les renvois datés ; **une seule reste active**, M5.)*

| # | Mutation | Attendu |
|---|---|---|
| M1 | *retirée P3 (G22 retirée ; ex-« retirer `config --services` »)* | — |
| M2 | *retirée (P2, C-31-4-14)* | — |
| M3 | *retirée P3 (G22 (c) retirée)* | — |
| M4 | *retirée P3 (G22 (a) retirée)* | — |
| M5 | écrire dans la sous-section une URL `raw.githubusercontent.com/…/main/…` | G19 rouge (garde existante, `textes_coherents.rs`) |
| M6 à M7 | *retirées P3 (fixture)* | — |
| M8, M9 | *retirées (P2)* | — |
| M10 à M11 | *retirées P3 (fixture)* | — |
| M12 à M13 | *retirées P3 (jetons de garde de G22 ; leur survie au contrôle 3 était le MEDIUM R3/F3 de la P3)* | — |
| M14 | *retirée P3 (G22 (c))* | — |
| M15 à M16 | *retirées P3 (fixture, garde contre le test muet)* | — |

**Non mutable, dit ici** : l'exactitude de la procédure elle-même, **garde du `rm -rf` comprise**. Elle n'est portée que par
le **journal d'exécution** (AC9, dont l'essai (iii) à quatre cas), le rejeu depuis le PDF (T8) et la relecture du manuel
en revue ; aucun test ne remplace d'avoir joué les commandes. C'est la raison d'être de la story.

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
   *(**Sans objet** depuis C-31-4-13, annoté en remédiation P2, 2026-10-24 : l'orchestrateur a choisi de faire corriger
   ce chemin par la story, élargi au manuel d'administration — AC13 ; aucune issue n'est demandée. La rubrique ne
   demande plus qu'**une** issue, la n° 1.)*

## Actions demandées à l'orchestrateur

1. **Aucun `closes`** nulle part : aucune issue ne porte la story ; la PR dit `refs` seulement si une issue est ouverte
   pour le n° 1 ~~ou le n° 2~~ ci-dessus *(le n° 2 est sans objet depuis C-31-4-13 — remédiation P2)*.
2. **`sprint-status.yaml`** : la clé `31-4-repetition-a-blanc` reste `ready-for-dev` (fait sur cette branche) ; la clé
   `29-7-repetition-a-blanc` reste `superseded` (déjà).
3. **À la release v0.15.0, après la publication de l'image** : rejouer la procédure du manuel **avec l'image publiée**
   (pas l'image de `main`) et verser le journal à côté du premier (`314-repetition-<date>.log`). C'est ce rejeu qui lève la
   réserve de l'AC10 (« image publiée »). Le rejeu est peu coûteux (quelques minutes, hors verrou de gate hormis le
   `docker pull`) : le manuel est la procédure, le journal de la première exécution en est le témoin. **Cette action n'est pas un critère du développeur** : elle appartient à la release.

## Tâches

- [ ] **T0 — état des lieux rejoué** : `git log -1 origin/main` ; `docker ps -a` (aucune pile `kesh-api` /
      `kesh-mariadb`) ; port libre ; `sudo -n true` (confirmer l'absence de `sudo`) ; base `kesh_314` créée et migrée
      depuis les migrations **du worktree**, port d'instance source attribué par l'orchestrateur ; dossiers de l'instance
      source sous `/home/gcorbaz/devel/kesh-gate-logs/314-source/` ; **moteur Docker** relevé au journal (Docker Engine
      en root, ou Docker Desktop / rootless / snap : le partage de `/var/tmp` avec le démon en dépend — constat (r)).
- [ ] **T1 — la sauvegarde synthétique** (C-31-4-15) : lancer une instance source **ordinaire** (`cargo run`, recette de
      `docs/testing.md` **sans** `KESH_TEST_MODE`, `KESH_COOKIE_SECURE=false`, `KESH_HOST=127.0.0.1`, base `kesh_314`,
      dossiers sous `314-source/`) ; dans l'assistant, choisir la **démonstration**, puis, **à l'écran et au journal**
      *(P4, R1/F1, C-31-4-17 : la démonstration ne porte ni écriture, ni exercice suivant, ni pièce — État des lieux)* :
      (a) **créer l'exercice suivant** (*Administration → Exercices comptables*) ; (b) **saisir une écriture de
      régularisation 1300/2300** datée du **dernier jour de l'exercice courant** ; (c) **faire entrer au moins une pièce
      par l'écran** : déposer un document fournisseur **synthétique** dans le dossier d'import de l'instance source
      (`314-source/inbox/`) — `frontend/tests/e2e/fixtures/spc_e2e_invoice.png`, fixture du dépôt (présente), copiée sous un
      autre nom — puis *Quotidien → Documents à traiter → Importer le dossier* ; (d) **si l'écran le permet**, émettre
      une facture dont le PDF se fige (la société de démonstration porte une adresse, l'IBAN dépend du compte bancaire :
      à constater) ; **sinon l'écrire au journal et au manuel comme « PDF figé non éprouvé »** (AC10). Relever au journal
      ce que la source porte **après** ces gestes (exercices, écriture, nombre de pièces, factures). Exporter par
      **Administration → Sauvegarde complète** ; vérifier que le fichier est de **format 2** : l'en-tête
      `X-Kesh-Backup-Files` se lit dans l'**onglet Réseau** des outils du navigateur (ou le manifeste du ZIP, ouvert
      dans le dossier de la story seulement), et il compte des **pièces**, non un format — sa présence ne vaut « format 2 »
      que si un export plus ancien ne le pose pas : **à constater et à verser au journal**, non à supposer ; noter le
      nombre de pièces. **Critère de préparation (du développement, non du manuel)** : *N* ≥ 1 pièce ; *N* = 0 est un
      **échec de la préparation de la source**, non un critère rempli (AC4, *P5, T2*). Aucune donnée réelle, `~/kesh-test/` jamais ouvert.
- [ ] **T2 — l'image** : construire l'image depuis `main` du worktree (`docker build -t gcorbaz/kesh:314 .`, sous verrou
      et `mem-guard.sh` ; elle n'écrase pas `latest`). Noter l'empreinte au journal.
- [ ] **T3 — jouer la procédure** (AC3 à AC7), **avant** d'écrire le manuel, dans un `$COPIE` neuf hors dépôt ; journal
      AC9 ; les captures utiles ; **les trois essais du refus** (AC9, dont l'essai (iii) à quatre cas) ; la destruction
      avec `sudo` si disponible, sans `sudo` (conteneur jetable) sinon. **Constats à faire et à verser au journal avant
      d'écrire le manuel** (tirés de la documentation de Compose, non exécutés à la validation) :
      (a) `-p kesh-repetition` l'emporte sur un `COMPOSE_PROJECT_NAME` exporté (`config` → `name:`) ;
      (b) avec un `-f` explicite, `docker-compose.override.yml` **n'est pas** lu tant qu'il n'est pas nommé, et l'est
      quand il l'est ;
      (c) `--project-directory "$COPIE"` lit le `.env` de `$COPIE` quand la commande est lancée depuis un autre dossier
      (son dossier parent, jamais le dépôt) ;
      (d) sans `.env`, `config` échoue sur les `:?` (preuve que l'ordre des contrôles 5 et 7 importe) ;
      (e) *retiré P3 (fixture)* ;
      (f) la sortie de `findmnt -T "$COPIE"` et de `docker context inspect` sur la station ;
      (g) où le navigateur dépose un téléchargement, et le réglage qui le fait demander l'emplacement ;
      (h) les SHA-256 de l'export de souveraineté, source et copie : égaux ou non (AC6) ;
      (i) `docker compose ls -a` disponible dans la version installée, **sans** `-p` ;
      (j) `docker compose version --short` sur la station (avec ou sans `v` ; comparaison « 2.24.0 ou plus » par
      `sort -V`, à écrire au contrôle 1) ; le comportement de `ports: !override` sous une Compose < 2.24 **si** une telle
      version est disponible sans installation (sinon : « non éprouvé », au manuel, AC10) ;
      (k) une clé dupliquée dans le `.env` : laquelle l'emporte (pour le dire au lecteur, contrôle 5) ;
      (l) le code de retour de `docker ps -a` quand le démon est injoignable (`DOCKER_HOST=unix:///nonexistent docker ps
      -a`, inoffensif) — l'attendu « réussit (code 0) » du contrôle 2 en dépend ;
      (m) l'étiquette `com.docker.compose.project=kesh-repetition` posée sur les conteneurs, le volume **et** le réseau
      après `up -d` (contrôle 9 : chaque filtre rend une ligne au moins) ;
      (n) la sortie de `config` avec l'override : **une seule** publication de `kesh-api`, `host_ip: 127.0.0.1`, aucune sur
      `mariadb` (contrôle 8), et `restart: "no"` pris en compte ;
      (o) `/var/tmp` sur la station : système de fichiers local, pas un lien (`realpath -e /var/tmp` → `/var/tmp`), et
      `mkdir -m 0700` sans `-p` qui échoue sur un dossier existant (contrôle 3) ;
      *(constats ajoutés en remédiation P3)*
      (p) la sortie de `config` porte `KESH_HOST: 0.0.0.0` (défaut du compose) — **attendu, et sans rapport avec la
      publication** ; le manuel dit de ne **pas** y toucher (R1) ;
      (q) le code de sortie d'un `case` sans motif correspondant (0) et celui de la forme `*) false` (non nul), dans le
      `if` de la garde (R2/F2) ;
      (r) le moteur : `/var/tmp` visible du démon (Docker Engine en root : oui ; Docker Desktop, rootless, snap : à
      constater), pour l'encadré AC10 (R-L3) ;
      (s) que le navigateur réellement employé (Firefox en snap sur la station) peut **envoyer** un fichier de `$COPIE`
      et **enregistrer** dans `$COPIE/telechargements/` ; à défaut, le manuel dit de déplacer par `mv` (R-L2) ;
      (t) la sortie de `config` pour la messagerie : `KESH_SMTP_HOST` vide (contrôle 7, R-L4) ;
      *(constat ajouté en remédiation P4)*
      (u) les filtres `--filter label=com.docker.compose.project=kesh-repetition` de `docker ps -a -q`, `docker volume ls -q`
      et `docker network ls -q`, **sur la copie vivante, avant sa destruction**, désignent **exactement** ses objets (le
      geste (ii) de l'AC7 (d), joué à blanc par `echo docker rm -f …` / `echo docker volume rm …` / `echo docker network
      rm …` — jamais exécuté pour de bon sur la copie réelle).
      Un constat contraire à la fiche **change la procédure** avant d'écrire le manuel, et se dit au Dev Agent Record.
- [ ] **T4 — écrire la sous-section** à partir du journal, pas de la mémoire ; renvois **en prose** au manuel
      utilisateur, `\ref` seulement vers des labels du manuel d'administration (`sec:restauration-machine-neuve`,
      `sec:test-restauration`, posé ici) ; encadré AC10 ; le renvoi de l. 1847 réécrit (AC1) et les quatre chemins des
      exercices corrigés (AC13) ; `make fr` sous verrou — il construit **trois** PDF (`docs/manual/Makefile` l. 35 :
      `admin-manual`, `user-manual`, `marketing-brochure`) : `admin-manual.pdf` et `user-manual.pdf` sont commités ;
      `marketing-brochure.pdf` est **restauré** (`git checkout -- <pdf>` puis `touch`) s'il n'a changé que par ses
      métadonnées. Contrôler le PDF **aplati** (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) : la sous-section, les neuf
      contrôles, l'URL au tag, la phrase « ne se joue jamais sur le NAS », « étapes 2 à 5 », **aucun `??`** dans la
      sous-section ni dans l'encadré de `sec:backup-dsm` (renvoi de l. 1847) ; et, sur **chacun des deux PDF**,
      `grep -c 'Paramètres → Exercices comptables'` sur la sortie aplatie → `0`, avec `grep -nE 'Paramètres[^.]{0,60}Exercices
      comptables' docs/manual/fr/*.tex` → aucun résultat (AC13). *(P4, F7)* **Toute commande Docker du manuel s'écrit dans
      un `lstlisting` ou sous `\keshcommand{…}`, jamais en prose** : G14 lit le source désechappé **en entier**
      (listings compris, sections Synology comprises) et compare au compose le mot qui suit un verbe de `docker compose`
      *(P5, T7)* ; un `lstlisting` ne protège donc pas parce que G14 ne le lirait pas, mais parce que le `[ \t]+` de sa
      regex s'arrête à la fin de ligne : « `docker compose up -d` puis … » **sur une même ligne**, en prose comme en
      listing, le ferait rougir sur « puis ». La forme `docker compose -p <projet> …`, celle de toutes les commandes de
      la copie, n'est pas lue (angle mort écrit) ; seules les commandes sans `-p` (`ls -a`, `version`, la forme courte
      de l'étape 5) sont exposées. T5 relance le binaire pour s'en assurer. **Relecture en revue** : la garde du `rm -rf` du `.tex`
      est comparée mot à mot à la forme de l'AC7 (branche `*) false` comprise).
- [ ] **T5 — gardes existantes** : lancer, au gate ciblé, le binaire `configuration_transmise` (G14, G15, G16) et la garde
      G19 de `textes_coherents.rs` ; M5 (URL `main` → G19 rouge, puis restauration).
- [ ] **T6 — CHANGELOG** (AC12) ; `sprint-status.yaml` ; registre.
- [ ] **T7 — gate** : `fmt`/`clippy` si du code a changé (aucun n'est attendu) ; le gate complet n'est exigé que si un
      fichier de code ou de test change ; **E2E complet non requis** (aucun écran, aucun code produit ne change) — **à dire
      dans le Dev Agent Record, sans l'afficher comme exécuté**.
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
  **source** (démonstration) emploie `kesh_314` du conteneur de dev.
- **Aucune donnée réelle** : la source est une démonstration ; `~/kesh-test/` n'est pas ouvert ; le journal de gate-logs
  ne se verse pas ; les captures ne montrent que la démonstration.
- **Le texte d'un manuel se vérifie contre lui-même** : T8 est le geste qui manque aux fiches qui « documentent » sans
  rejouer (les six tours de l'Epic 24, § *Le prompt d'une passe doit NOMMER le manuel*).
- **Conventions du manuel** : `\keshcommand{}`, `\keshpath{}`, `keshwarning`/`keshnote`/`keshtip`, `lstlisting[language=bash]`.
  Éviter l'emoji ; les exemples de commandes sont ceux du journal, avec les chemins de la copie en `<…>` ou `$COPIE`.
- **Version dans le manuel** : tant que `\keshVersion` vaut `0.14.0`, l'URL est au tag `v0.14.0` (G19). La phrase « format
  2, depuis la 0.15.0 » est déjà celle du manuel (31-1a/31-1b) ; au bump de release, G19 force la mise à jour de l'URL.
- **Précédent de destruction gardée** : `scripts/synology/recette.sh` (noms propres, `trap … EXIT`), à imiter pour
  l'écriture de la garde.
- **Contrôle 8 — `KESH_HOST`** : la sortie de `config` porte `KESH_HOST: 0.0.0.0` (défaut du compose) ; c'est l'adresse
  d'écoute **dans** le conteneur, non une publication. Le manuel dit de ne pas y toucher.

## Hors périmètre

La sauvegarde automatique (31-1c) ; la correction des noms et du port en dur du compose (issue n° 1) ; une répétition
**sur le NAS** ; un E2E de la procédure (elle est une procédure d'exploitation sur Docker, hors du périmètre de
Playwright) ; la traduction (le manuel d'administration n'existe qu'en français) ; le manuel utilisateur **hors du
chemin des exercices** (AC13) ; **les messages de l'application** qui écrivent « Paramètres → Exercices comptables »
(catalogues `messages.ftl` des quatre locales, `notify.ts`, `settings/opening-balances/+page.svelte`) : ils restent
**valides** par la carte *Exercices comptables* de la page *Paramètres* et relèvent du code du produit (C-31-4-13, AC13) ;
une garde de hiérarchie des chemins de menu ; **tout outillage de fixture et toute garde de texte nouvelle** (C-31-4-15) ;
#404 et #235 (P1, nommés à la rétrospective de l'Epic 31) ; #671.

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
  *(Annotation du 2026-10-24, après la P1 et avant la remédiation P2 : l'orchestrateur a ajouté au registre C-31-4-13 —
  le chemin « Paramètres → Exercices comptables » corrigé par la story, ce qui rend l'issue n° 2 sans objet — et
  C-31-4-14. Les recomptes ci-dessus restent ceux de leur date.)*
- 2026-10-24 — **Validation P2** (Sonnet, deux lentilles, contexte frais, lecture seule). Bruts : **R 0/0/5/8**
  (CRITICAL/HIGH/MEDIUM/LOW), **F 0/0/6/9**. **Convergence des deux lentilles sur G22** telle que la P1 l'avait étendue
  (R3/F1 : `manuel::normaliser` aplatit les sauts de ligne, `tests/common/manuel.rs` l. 34-61, si bien que (c) passait à
  vide, (d) était indécidable, (e) illisible ; R2/F2 : (e) rougissait sur `docker compose ls -a`) ⇒ **retrait** décidé
  par l'orchestrateur (C-31-4-14) : G22 réduite à des présences lues dans le source brut, plus d'ordre, de seuils ni de
  critère (e). **Aucun finding réfuté** ; un fait **rectifié** dans F7 : le corps du login est camelCase, le jeton s'y
  lit `accessToken` et non `access_token` (`routes/auth.rs` l. 133-142). **Corrections** : C-31-4-13 reporté **et élargi**
  (R6) — AC13 nouvelle, quatre lignes (`user-manual.tex` l. 794, 845 ; `admin-manual.tex` l. 1445, 1451), deux PDF
  régénérés et contrôlés à zéro résultat, entrée `Corrigé` au CHANGELOG (AC12), issue n° 2 sans objet, *Périmètre* et
  *Hors périmètre* amendés ; **précision de fait** portée au registre : la page *Paramètres* mène aussi aux exercices
  (carte et bouton *Gérer*, `settings/+page.svelte` l. 582-590) — l'ancien chemin est un détour, non une impasse.
  C-31-4-14 reporté : AC11 réécrit (sous-section bornée sur le source sans commentaires, jetons `-p kesh-repetition`,
  `config --services`, `/health`, `realpath -e`, `test ! -L`, `/var/tmp`, absence non ancrée de `KESH_*_HOST_DIR=`,
  constante unique et retrait en mémoire de chaque jeton — R11) ; le jeton littéral `docker compose config` remplacé par
  `config --services`, le *principe directeur* l'interdisant ; contrôle 3 réécrit (`/var/tmp/kesh-repetition-<date>`,
  `mkdir -m 0700` sans `-p`, `test ! -L`, `realpath -e`, vacuité avant le premier fichier — R5/F6, R8, F11) ; garde du
  `rm -rf` réécrite (`dirname`, nom, `test -d`, `test ! -L`, `realpath -e`, dans le même listing — F5). R1/F8 → contrôle 1
  lit `docker compose version` (≥ 2.24), contrôle 8 exige une seule publication, aucun `published: "80"` ni `0.0.0.0`.
  R2/F2 → **sans objet** pour la garde ((e) retirée) ; le *principe directeur* dit que `ls` et `version` ne portent ni
  `-p` ni `--project-directory`. R4/F7 → authentification de la fixture écrite (login du preset en constantes, jeton
  porteur, refus sans appel suivant, cas de test). F3 → contact, compte bancaire principal, `GET /invoices/<id>/pdf`
  (200, `%PDF`), `X-Kesh-Backup-Files: 2` exigé, T1 contrôle deux pièces. F4 → tranché : la sous-section reste après
  `Test de restauration périodique`, le renvoi de l. 1847 devient `\S\ref{sec:test-restauration}`. **LOW traités** :
  R7 (filtres non vides au contrôle 9), R8 (lien symbolique, `mkdir` sans `-p`, ni `exit` ni `return`), R9
  (`restart: "no"`), R10 (constats (j) à (n), remplissage du `.env` dans le fichier), R11 (garde contre le test muet,
  `jq` absent rouge), R12 (dossier de l'essai détruit, `.env` de (ii), essai (iii) borné), R13 (`spc_e2e_invoice.png`) ;
  F7, F8, F10 (commande de l'étape 5 sous le principe), F11, F12, F13 (export de souveraineté à l'étape 0), F14
  (l'exemplaire n'est pas une sauvegarde), F15. **LOW sans objet** : F9 (critère d'ordre (b) retiré). **Ajout propre à
  la remédiation** : `mkdir -m 0700` (`/var/tmp` partagé entre utilisateurs). **Recomptes** (fiche après remédiation,
  `grep` sur la fiche) : AC1 à AC13 (13, étaient 12) ; T0 à T8 (9) ; contrôles de l'AC3 : 9 (inchangé) ; mutations :
  16 numéros M1 à M16, dont 13 actives et 3 retirées (M2, M8, M9) (étaient 11) ; constats de la T3 : (a) à (o), 15
  (étaient 9) ; choix C-31-4-1 à 14 (étaient 12) ; issues à demander : 1 (étaient 2). Validation P3 à lancer (Opus,
  R et F ; `314-validate-prompt-p3.md`).
- 2026-10-24 — **Validation P3** (Opus, deux lentilles, contexte frais, lecture seule). Bruts : **R 0/0/7/8**
  (CRITICAL/HIGH/MEDIUM/LOW), **F 0/1/3/9**. **Signal de recyclage déclaré au Project Lead** (`CLAUDE.md` § *Règle de
  splitting préventif*, amendement D5) : P1 Opus 0/1/19 → P2 Sonnet 0/0/11 → P3 Opus 0/1/10 ; les défauts naissent de ce
  que les remédiations ajoutent **autour** de la procédure (script de fixture, test shell, enveloppe Rust, garde G22 et
  leurs mutations ; le HIGH F1 : l'adresse de la société du preset est vide et l'IBAN de test est un QR-IBAN, le PDF ne peut
  pas se figer). **Sortie par retrait (C-31-4-15)** : fixture, test shell, enveloppe Rust, G22 et leurs mutations, tâches,
  AC et constats sont retirés, leurs numéros gardés (AC8, AC11, M1 à M16 sauf M5, constat (e)) ; la sauvegarde synthétique
  s'exporte par l'écran depuis une instance de démonstration ; la justesse est portée par l'exécution journalisée, le rejeu
  depuis le PDF (T8) et la relecture du manuel en revue. **Sans objet du fait du retrait** : F1 (HIGH) ; R3/F3 (M12/M13 ne
  tuent rien) ; R6 (cas de refus du test shell) ; F5 (casse des en-têtes de l'export) ; F6, F7 (motif et jetons de G22) ;
  F10 (en-tête de table des mutations, la table est réécrite) ; F11 est au contraire corrigé (ligne 53 : constante en l. 2785, volumes préfixés en l. 2762)  ;
  R-L6, R-L8 ; les lignes d'État des
  lieux sur `_test/seed`, l'authentification et le gel du PDF, supprimées. **Corrigés** : R1 (contrôle 8 borné au bloc
  `ports:` de `kesh-api` ; `KESH_HOST: 0.0.0.0` attendu, constat (p)) ; R2/F2 (garde du `rm -rf` écrite **exactement**
  avec `*) false`, essai (iii) à **quatre cas** dont le nom faux ; constat (q)) ; R4 (exports de souveraineté déplacés dans
  `$COPIE/telechargements/`, nommés à la destruction et au contrôle final) ; R5 (exemplaire réel posé **après le
  contrôle 9** ; « arrêt » défini à l'AC7 (d)) ; R7/F4 (dix sites de l'application nommés, **hors périmètre**, entrée
  `Corrigé` bornée aux manuels, contrôle de l'AC13 borné aux `.tex`) ; F8 (`fi` sur sa ligne) ; F9 (renvoi « validation P2,
  R10 ») ; F12 (`Corrigé` entre `Modifié` et `Interne`) ; F13 → constats (j), (r) ; R-L1 (contrôle 1 refait à la
  destruction) ; R-L2 → constat (s) ; R-L3 (copie oubliée qui reste ; moteur Docker nommé à l'AC10, constat (r)) ; R-L4 →
  constat (t) et `KESH_SMTP_HOST` vide au contrôle 7 ; R-L5 (`make fr` : trois PDF, la brochure restaurée) ; R-L7 (restes
  des essais retirés, variable de l'essai (i) dite). **Recomptes** (fiche après remédiation, commandes à l'appui) : AC1
  à AC13 : 13 numéros, dont 2 retirés (AC8, AC11) ; T0 à T8 : 9 (T1 et T5 réécrites) ; contrôles de l'AC3 : 9 ;
  mutations M1 à M16 : 16 numéros, 1 active (M5), 15 retirées ; constats de la T3 (a) à (t) : 20 lettres, 1 retirée
  ((e)) ; choix C-31-4-1 à 16 ; issues à demander : 1. Validation P4 à lancer (Sonnet, R et F ; `314-validate-prompt-p4.md`).
- 2026-10-24 — **Validation P4** (Sonnet, deux lentilles, contexte frais, lecture seule). Bruts : **R 0/0/3/5**
  (CRITICAL/HIGH/MEDIUM/LOW), **F 0/0/1/6**. **Aucun finding réfuté** : chacun vérifié à la lecture —
  `crates/kesh-seed/src/lib.rs` (`seed_demo` : société, plan, un exercice, réglages de facturation, taux de TVA ; aucune
  écriture ni facture), `.env.example` (`#MARIADB_ROOT_PASSWORD=` l. 374, `#MARIADB_PASSWORD=` l. 377, `MARIADB_DATABASE`
  et `MARIADB_USER` actives), `frontend/tests/e2e/fixtures/spc_e2e_invoice.png` (présente, employée par `inbox-import.spec.ts`
  et `supplier-documents.spec.ts`). **Corrigés** (C-31-4-17) : R1/F1 (MEDIUM) → T1 prescrit des gestes à l'écran, journalisés
  (exercice suivant, écriture 1300/2300 du dernier jour, une pièce par l'import du dossier, PDF figé si possible), critère
  de l'AC4 *N* ≥ 1 et 0 manquante, renvoi aux étapes 2 à 5 précisé, AC5 geste 2 rattaché à la T1, ligne « Instance de
  démonstration » établie au code ; R2 (MEDIUM) → AC7 (d) : trois cas d'arrêt dont le reste d'une répétition antérieure
  (dossier existant : `COPIE` y est affectée ; dossier disparu : identifiants rendus par les filtres d'étiquette, éprouvé à
  blanc, constat (u), AC10) ; R3 (MEDIUM) → l'étape 0 se joue **après** les contrôles 1 à 3 et enregistre dans
  `$COPIE/telechargements/`, l'arrêt retire un exemplaire atterri chez le navigateur. **LOW traités** : R4, R5, R6 (un seul
  `if`, deux refus de plus à l'essai (iii)), R7, R8, F2, F3, F4, F5, F6, F7. **R7, annotation datée** (sans réécrire les
  lignes actuelles) : la remédiation P3 avait remplacé deux annotations de la fiche — `*(Amendé en remédiation P2,
  2026-10-24 — R2/F2.)*` au principe directeur, devenue `*(Amendé en remédiation P2, puis P3.)*`, et `*(Ajouté en
  remédiation P2, 2026-10-24 — C-31-4-13 élargi par R6, et F4.)*` au périmètre, devenue `(C-31-4-13)` ; les textes
  d'origine sont consignés ici, et les lignes actuelles gardent leur forme. **Recomptes** (fiche après remédiation, commandes
  à l'appui, périmètre : la fiche seule) : AC1 à AC13 : 13 numéros, dont 2 retirés (`grep -oE '^\*\*AC[0-9]+' | sort -u | wc -l`) ;
  T0 à T8 : 9 (`grep -cE '^- \[ \] \*\*T[0-9]'`) ; contrôles de l'AC3 : 9 (`grep -nE '^ [1-9]\. \*\*'` : lignes 197 à 272) ;
  mutations M1 à M16 : 16 numéros, 1 active (M5), 15 retirées (inchangé) ; constats de la T3 (a) à (u) : 21 lettres, 1 retirée
  ((e)), (u) ajouté ; choix C-31-4-1 à 17 (`grep -oE 'C-31-4-[0-9]+'` sur le registre) ; issues à demander : 1. Validation P5
  ciblée à lancer (Opus, une lentille, sur le seul commit de cette remédiation ; `314-validate-prompt-p5-ciblee.md`).
- 2026-10-24 — **Validation P5 ciblée** (Opus, une lentille T, contexte frais, lecture seule, sur le seul commit de la
  remédiation P4 `51095569`). Bruts : **T 0/0/2/5** (CRITICAL/HIGH/MEDIUM/LOW). **Aucun finding réfuté** ; les deux
  MEDIUM sont nés de la remédiation P4 elle-même (R3 non propagé ; critère du développement écrit au manuel).
  **Corrigés** (C-31-4-18) : T1 → R3 propagé à tous les passages (contrôle 3, AC4, AC6, *La marche*) : le secret réel
  entre dans `/var/tmp` dès l'étape 0, dans `$COPIE/telechargements/` ; le geste qui pose `source.keshbackup` est écrit
  (`mv` depuis `telechargements/`, après le contrôle 9, puis `chmod 0600`) ; la garantie d'un arrêt est la destruction de
  `$COPIE` (AC7 (d)), non « aucun secret dans `/var/tmp` » ; T2 → critère du manuel (0 pièce manquante, autant de pièces
  restaurées que l'export en a annoncé, écriture récente et factures de l'étape 5) séparé de l'exigence du développement
  (T1 : *N* ≥ 1) ; AC5 geste 2 et renvoi aux étapes 2 à 5 réécrits dans le même sens. **LOW** : T3 (« directement, si le
  navigateur le permet », constat (s)), T4 (arrêt au contrôle 3, déclencheur du `mkdir` refusé, « contrôles 4 à 8 »,
  passage de (i) à (ii) si `down -v` échoue, (i) rangé à l'AC10 — quatre précisions), T5 (« passe le `case`, puis la
  garde refuse au `-d` »), T6 (`&&` entre les deux gestes ; avant le premier `up`, `rm -rf` gardé seul), T7 (G14 lit
  tout le source désechappé ; la protection d'un listing tient au `[ \t]+` de sa regex). **Recomptes** (fiche après
  remédiation, périmètre : la fiche seule) : AC1 à AC13 : 13 numéros, dont 2 retirés ; T0 à T8 : 9 ; contrôles de
  l'AC3 : 9 ; constats (a) à (u) : 21 lettres, 1 retirée ; mutations : 16 numéros, 1 active ; choix C-31-4-1 à 18
  (`grep -oE 'C-31-4-[0-9]+'` sur le registre). Validation P6 ciblée à lancer (Haiku, une lentille, sur le seul commit de
  cette remédiation ; `314-validate-prompt-p6-ciblee.md`).

## Dev Agent Record

*(à remplir au développement : gates réellement exécutés avec leurs chiffres, journal de répétition, constats de T3,
captures, mutations tuées, écarts à la fiche.)*

### Validation P6 ciblée (2026-10-24) — Haiku, lentille T sur `2d3394aa` : 0/0/1/3 ; MEDIUM remédié par l'orchestrateur

- **T2-bis (MEDIUM)**, vérifié à la lecture par l'orchestrateur : le critère « autant de pièces restaurées que l'export en
  a annoncé » était déclaré valable pour le format 1, qui n'annonce aucun compte. Corrigé à l'AC4 : sans compte annoncé
  (production antérieure à la 0.15.0), le critère se réduit à 0 pièce manquante et à la lecture de l'étape 5.
- LOW L1 à L3 laissés au développement (L1 : réserve « si le navigateur le permet » à propager aux l. 226, 231, 348 ; L2 :
  la regex de G14 ne lit qu'une liste fermée de verbes ; L3 : « l'original » de l'étape 0 à définir).
- Passe suivante : ciblée P7 (Sonnet) sur ce commit.

### Validation P7 ciblée (2026-10-24) — Sonnet, lentille T sur `8aa08d3b` : 0/0/1/1 ; MEDIUM remédié par l'orchestrateur

- **MEDIUM**, né de la remédiation P6 de l'orchestrateur : le critère réduit « 0 pièce manquante » pour un export de
  format 1 était **inatteignable** dès que la production porte des documents (le format 1 ne contient aucune pièce,
  `compter_les_pieces_manquantes` les compte toutes manquantes ; contredit la l. 188). **Retrait** : la répétition
  suppose une production en 0.15.0 ou plus (format 2) ; une production antérieure se met d'abord à jour. Le cas du
  format 1 sort du critère.
- LOW : la T4 renvoie à l'AC4 en bloc pour le texte du manuel (suffisant).
- *Troisième remédiation de l'orchestrateur sur l'Epic 31 qui introduit le défaut suivant* (31-1 P5, P6 ; 31-4 P6) : à la
  rétrospective.
- Passe suivante : ciblée P8 (Opus), **dernière passe permise**.

### Validation P8 ciblée (2026-10-24) — Opus, lentille T sur `HEAD~2..HEAD` : 0/0/2/3 — **plafond de huit passes atteint**

- **M1** (résidu de la remédiation P7, de l'orchestrateur) : l'AC2 disait encore « une sauvegarde de format 1 se restaure
  aussi, ses pièces manquent » ; corrigé — le format 1 ne vaut pas répétition, une production antérieure se met d'abord à
  jour (renvoi en prose à la section *Mise à jour*, `sec:mise-a-jour`).
- **M2** : le prérequis était exprimé par la **version**, que l'image de développement (construite depuis `main`, crates
  0.14.0) ne peut pas afficher ; il s'exprime désormais par le **format constaté** (export de format 2, en-tête
  `X-Kesh-Backup-Files` présent), et se vérifie **avant le contrôle 1**, avant de rien créer. Pendant le développement
  (T8), l'image de `main` exporte en format 2 : le prérequis est rempli par le format, la version affichée étant une
  substitution déjà déclarée (T8, AC10).
- LOW laissés au développement : « toutes manquantes » vaut parce que le dossier des documents de la copie est neuf (à
  dire au manuel) ; chapeau de la fiche arrêté à la P4 ; « Corrigé » de l'entrée P6 périmé (non réécrit).
- **Fin de la validation au plafond.** Trend : P1 Opus R 0/1/9/7 F 0/0/10/9 → P2 Sonnet R 0/0/5/8 F 0/0/6/9 → P3 Opus R
  0/0/7/8 F 0/1/3/9 (signal de recyclage, retrait C-31-4-15) → P4 Sonnet R 0/0/3/5 F 0/0/1/6 → P5 ciblée Opus 0/0/2/5 →
  P6 ciblée Haiku 0/0/1/3 → P7 ciblée Sonnet 0/0/1/1 → P8 ciblée Opus 0/0/2/3. **Les deux MEDIUM de la P8 sont corrigés
  par l'orchestrateur sans passe de vérification** : l'agent de développement les recontrôle à son étape 0, et la revue de
  code relit le manuel contre la fiche. Les P6 à P8 n'ont trouvé que des défauts des remédiations elles-mêmes.
