# Story 31.4 : La répétition à blanc d'une clôture (ex-29-7, documentation exécutée)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie ; remédiée le même jour après la validation P1 (Opus, lentilles R et F),
puis après la validation P2 (Sonnet, lentilles R et F ; choix C-31-4-13 et C-31-4-14 de l'orchestrateur).
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
| Gardes qui lisent déjà le manuel d'administration | `crates/kesh-api/tests/configuration_transmise.rs` : **G14** (services nommés par `docker compose <verbe> <service>` ; la forme `docker compose -p <projet> …` **n'est pas lue**, angle mort écrit l. 2660-2662), **G15** (volumes cités ; le préfixe `kesh-repetition_` est accepté, l. 2757-2759), **G16 (e)** (liste **fermée** des « Hyper Backup » hors des sections Synology, l. 2779-2793 : nommer Hyper Backup dans la sous-section la ferait rougir) |
| Parcours déjà automatisé | `frontend/tests/e2e/fiscal-year-closing-journey.spec.ts` (29-1b) joue : exercices 2025 et 2026, comptes `1300`/`1020`, compte bancaire lié, relevé CAMT `camt053_v04_year_end.xml`, préparation (K2 et K3 en attention), rapprochement manuel, régularisation de 250.00 au 31.12.2025, contre-passation, clôture, réouverture. **La répétition en est la version « copie du NAS »**, pas un nouveau test fonctionnel. `backup-restore-new-install.spec.ts` exporte **par l'écran** (l. 103-105, `exportByScreen`), non par l'API ; la facture du helper `createAndValidateInvoiceViaApi` est **datée du jour** (`helpers/api-fixtures.ts` l. 105-106) ; le dépôt dans la boîte d'import écrit dans `KESH_INBOX_DIR`, par défaut `/tmp/kesh-e2e/inbox` (`helpers/inbox.ts` l. 27, 100), avec une poignée de main (l. 153-160) |
| Hors mode test | les routes `/api/v1/_test` ne sont montées qu'en `test_mode` (`lib.rs` l. 1153-1157) ; hors de ce mode, la requête tombe sur le repli `ServeDir` (l. 175-176, 1167), et `test_endpoints_e2e.rs` l. 97-104 admet **404 ou 405** : le refus de la fixture ne peut pas reposer sur un « 404 » |
| Authentification de l'API (validation P2, R4/F7) | `_test/seed` est monté hors du routeur protégé (`lib.rs` l. 1153-1157 : `main_router.nest("/api/v1/_test", …)` après `merge(protected)`) : **sans jeton**. Les autres routes de la fixture en exigent un : `inbox-import` Comptable+, `full-export` Admin et refusé aux jetons d'accès personnels (`lib.rs` l. 249-251). Le preset `with-company-no-fy` crée deux administrateurs, `admin`/`admin123` et `changeme`/`changeme` (`kesh-db/src/test_fixtures.rs`, doc de `seed_accounting_company_no_fy`). `POST /api/v1/auth/login` (`lib.rs` l. 1118) prend `{"username","password"}` (`routes/auth.rs` l. 107-110) et rend un corps **camelCase** dont `accessToken` (`LoginResponse`, `rename_all = "camelCase"`, l. 133-142 — et non `access_token`), utilisable en `Authorization: Bearer` (`middleware/auth.rs`) ; un jeton de session n'est pas un jeton d'accès personnel |
| Gel du PDF d'une facture (validation P2, F3) | `routes/issued_invoice_pdf.rs` l. 1-14 : le PDF émis est **figé au premier rendu après validation** ; la validation seule ne fige rien. Les déclencheurs : `GET /api/v1/invoices/{id}/pdf` (`lib.rs` l. 863) et l'envoi par courriel. Le rendu exige une facture `validated`, son **contact**, et un **compte bancaire principal** de la société (`routes/invoice_pdf_service.rs`, `find_primary`) — `fiscal-year-closing-journey.spec.ts` l. 181 crée le compte avec `isPrimary: true`, `helpers/api-fixtures.ts` l. 39 crée le contact par `POST /api/v1/contacts`. Sans ce `GET`, la sauvegarde de la fixture ne porterait qu'**une** pièce |
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
- `$COPIE` est un chemin **absolu**, posé une fois, dont le dernier élément commence par `kesh-repetition-` —
  *(précisé en remédiation P2, C-31-4-14)* et qui vit directement sous `/var/tmp` : `COPIE=/var/tmp/kesh-repetition-<AAAAMMJJ>`.
- **Avant tout**, la procédure neutralise les variables du shell qui l'emportent (`COMPOSE_*`, `DOCKER_HOST`,
  `DOCKER_CONTEXT`, `KESH_*`, `MARIADB_*`) et vérifie le contexte Docker (contrôle 1).

Le manuel peut écrire les commandes sur deux lignes (`\`), jamais les raccourcir par une fonction ou un alias : une
session où la fonction manque retomberait sur les défauts implicites (C-31-4-10 (c)). Ces comportements de Compose viennent
de sa documentation : **ils sont des constats à faire en T3, avant d'écrire le manuel** (voir T3).

*(Amendé en remédiation P2, 2026-10-24 — R2/F2.)* Le principe vaut pour les commandes **qui désignent un projet** :
`config`, `pull`, `up`, `ps`, `logs`, `exec`, `down`. `docker compose ls -a` (contrôle 2, qui doit voir **tous** les
projets) et `docker compose version` (contrôle 1) n'en désignent aucun et ne portent **ni** `-p` **ni**
`--project-directory`. La contradiction relevée par R2/F2 visait G22 (e), qui exigeait ces drapeaux de *chaque*
`docker compose` : **(e) est retirée** (C-31-4-14), le point est sans objet pour la garde ; il reste dit ici pour le
lecteur. Plus aucune garde ne lit chaque commande : leur forme est tenue par le journal d'exécution et la T8 (AC11).

## Périmètre

**Dedans** : une sous-section du manuel d'administration (`.tex` **et** PDF), et un `\label` posé sur la sous-section
voisine `Test de restauration périodique` (mécanique, pour le renvoi) ; un script de **fixture synthétique** versionné
(`scripts/repetition-fixture.sh`) qui fabrique la sauvegarde servant à écrire et à rejouer la procédure, avec son test
(`scripts/tests/repetition-fixture.test.sh`) rattaché au gate par une enveloppe Rust ; une **garde** de texte (G22) qui
épingle les contrôles de la copie ; l'entrée `CHANGELOG`. **La procédure est jouée réellement au développement**, journal
à l'appui.

*(Ajouté en remédiation P2, 2026-10-24 — C-31-4-13 élargi par R6, et F4.)* **Dedans aussi** : le chemin des exercices
comptables écrit tel que le menu le range, dans les **deux** manuels (`user-manual.tex` l. 794, 845 ; `admin-manual.tex`
l. 1445, 1451 ; AC13), avec la régénération de `user-manual.pdf` ; et le renvoi relatif « cf. sous-section précédente »
de `admin-manual.tex` l. 1847, remplacé par `\S\ref{sec:test-restauration}` (AC1). Corrections mécaniques : deux `.tex`
et deux PDF, sous le seuil de la règle de splitting.

**Dehors** (voir *Hors périmètre*) : toute modification du code du produit, des compose ou de `.env.example` (les noms
fixes sont **signalés**, non corrigés, n° 1) ; la répétition **sur le NAS** (impossible, ci-dessous) ; le manuel
utilisateur **hors du chemin des exercices** (AC13) *(amendé en remédiation P2 : la fiche écrivait « le manuel
utilisateur (le défaut de chemin de menu est signalé, n° 2) »)* ; une garde de hiérarchie des chemins de menu (C-31-4-13) ;
les trois autres langues (le manuel d'administration n'existe qu'en français, `docs/manual/{de,en,it}/` ne porte que des
`README.md`) ; la sauvegarde automatique (31-1c).

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
qui ne porte pas la production, avec Docker et Compose v2 ≥ 2.24 ; **jamais le NAS** : noms de conteneurs fixes, § État
des lieux ; un dossier **local**, jamais synchronisé ni monté depuis le NAS — `/var/tmp`, hors de `$HOME`, AC3
contrôle 3) ; *ce qu'elle suppose* (une installation en
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
    - `test -z "$(ls -A -- "$COPIE")"` réussit — dossier **vide**, vérifié **avant** le premier fichier qu'on y pose
      (l'exemplaire `source.keshbackup` de *La marche*, étape 1, puis les téléchargements du contrôle 4 — F11) ;
    - second filet seulement : `findmnt -T "$COPIE"` rend un système de fichiers local (ni `cifs`, ni `nfs*`, ni
      `fuse.*`) — il écarte un `/var/tmp` monté depuis le réseau, **pas** un dossier synchronisé, et le manuel le dit.
    Les pièces réelles restaurées et la sauvegarde de sécurité vivront dans ce dossier : synchronisé, il les ferait
    remonter vers le NAS et ses sauvegardes. Le fichier `.keshbackup` téléchargé par le navigateur y est **déplacé**
    avant usage (AC4).
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
    fichier**, en éditant la ligne existante de `.env.example` (`KESH_COOKIE_SECURE=true` l. 155 devient `false`), jamais
    en ajoutant une seconde ligne pour la même clé (laquelle l'emporte : constat T3 (k)) ni par un `export` dans le shell,
    que le contrôle 1 vient de neutraliser.
 6. **Aucune ligne `HOST_DIR` active.** `grep -nE '^[[:space:]]*KESH_[A-Z_]*HOST_DIR[[:space:]]*=' "$COPIE/.env"` ne rend
    rien (les trois lignes **commentées** de `.env.example`, l. 232, 233, 262, ne comptent pas ; un `grep 'HOST_DIR'` nu
    les rendrait et apprendrait au lecteur à passer outre un contrôle — validation P1, F2).
 7. **Première lecture de la configuration** (le `.env` est posé : les `:?` sont satisfaits). Avec le seul premier `-f` :
    `config --services` rend **exactement** `mariadb` et `kesh-api`, en **ensemble** (l'ordre de sortie n'est pas établi —
    R10 (n)) — **c'est le contrôle décisif** : le compose Synology
    n'a qu'un service ; `config` rend en première ligne `name: kesh-repetition` ; sa `DATABASE_URL` contient
    `@mariadb:3306` (**second filet seulement** : le manuel prescrit cette forme au Synology aussi, l. 269) ; tous ses
    `source:` sont sous `$COPIE`, hormis le volume nommé `kesh-mariadb-data`.
 8. **Override, puis configuration relue.** `$COPIE/docker-compose.override.yml` : `image: gcorbaz/kesh:<version relevée>`
    et `ports: !override ["127.0.0.1:<port libre>:80"]` — la copie n'écoute que sur la boucle locale (C-31-4-6) ; et
    `restart: "no"` sur les deux services *(ajouté en remédiation P2, R9 LOW : une copie oubliée ne repart pas avec les
    données réelles au prochain démarrage de Docker ; prise en compte à constater en T3)*. Les commandes portent
    désormais les **deux** `-f`. Le contrôle 7 est **refait sur la sortie finale**, et en plus : l'`image:` est celle
    épinglée ; *(précisé en remédiation P2, R1/F8)* le service `kesh-api` porte **une seule** publication de port, avec
    `host_ip: 127.0.0.1` et `published: "<port choisi>"` ; la sortie ne contient **aucun** `published: "80"`, **aucun**
    `0.0.0.0`, et le service `mariadb` **aucune** publication. Deux publications, ou une sans `host_ip`, signifient que
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
**Administration → Sauvegarde complète** —, dont la répétition travaille sur **un exemplaire placé dans
`$COPIE/source.keshbackup`** (mode 0600) : l'original, où qu'il soit conservé, n'est ni déplacé ni supprimé par la
procédure, et un exemplaire **créé pour l'occasion** hors de `$COPIE` (un téléchargement du navigateur) y est
**déplacé** — *(ajouté en remédiation P2, F14 LOW)* le manuel dit en toutes lettres que cet exemplaire **n'est pas
une sauvegarde à conserver** et sera détruit avec la copie : qui veut le garder en fait d'abord une copie ailleurs ;
tout fichier téléchargé depuis la copie (un PDF de facture) va dans `$COPIE/telechargements/` ; la lecture
du bilan (« pièces restaurées / manquantes ») ; la reconnexion avec **les identifiants de la production**. **Critère de
réussite** nommé : le bilan annonce *N* pièces restaurées et **0 manquante** pour une sauvegarde de format 2 ; la facture
la plus récente de la production est dans la liste ; son PDF se télécharge ; un document fournisseur s'affiche.
*(Ajouté en remédiation P2, F10 LOW.)* Les commandes que ces étapes 2 à 5 écrivent pour l'installation ordinaire
(`docker compose logs kesh-api`, étape 5) se jouent, dans la copie, **selon le *principe directeur*** : la sous-section
les redonne sous cette forme plutôt que de laisser le lecteur recopier la forme courte contre le dossier courant.

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
l'export de souveraineté et la comparaison de son SHA-256 avec celui de la production — celui-ci produit **sur la
production, à l'étape 0, avant le `.keshbackup`** (*La marche* ; ajouté en remédiation P2, F13 LOW : sans lui, l'empreinte
de la copie n'a rien à quoi se comparer). Si le constat de T3 montre que les
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
`rm -rf "$COPIE"`. Chaque `rm -rf` et le `docker run -v` sont précédés de la **garde**, *(réécrite en remédiation P2 —
C-31-4-14, R8, F5)* **dans le même listing** que le geste qu'elle protège :
- `$COPIE` est non vide ;
- `[ "$(dirname -- "$COPIE")" = /var/tmp ]` — directement sous `/var/tmp` (donc **absolu** : un chemin relatif passé à
  `-v` crée un volume nommé, et le `rm` tourne à vide — validation P1, R L2) ;
- `case "$(basename -- "$COPIE")" in kesh-repetition-*) … ;; esac` — dernier élément nommé ;
- `test -d "$COPIE"` et `test ! -L "$COPIE"` — un dossier existant, **pas un lien** (le `-v "$COPIE":/copie` suivrait le
  lien, et le `rm -rf` du conteneur détruirait les homonymes du dossier visé) ;
- `[ "$(realpath -e -- "$COPIE")" = "$COPIE" ]` — le chemin résolu **est** le chemin écrit. Les tests de texte
  (`dirname`, `basename`) ne lisent que la chaîne : ils acceptent un `/var/tmp` qui serait lui-même un lien vers un autre
  disque, et ne voient pas un remplacement du dossier par un lien entre deux commandes. Une barre finale
  (`…/kesh-repetition-x/`) fait aussi échouer la comparaison : refus sûr, le manuel dit d'écrire `COPIE` sans elle.
La garde s'écrit en un seul bloc `if … then <geste> else echo "refus : …" fi`, **sans `exit` ni `return`** (le premier
fermerait le terminal du lecteur, le second n'a de sens que dans une fonction, que le *principe directeur* écarte).
Consigne : un `rm -rf "$VAR"` non gardé est la faute dangereuse type ; le dev l'éprouve avec une variable vide, un chemin
relatif **inexistant** et un lien symbolique (AC9 (iii)), et lit que la garde arrête. G22 exige la présence des jetons
`realpath -e` et `test ! -L` (AC11) ; l'exactitude de la garde est portée par l'essai (iii) et la T8. (c) **Contrôle final** écrit : le contrôle 2 refait (ni `kesh-api` ni
`kesh-mariadb` parmi les noms ; conteneurs, volumes et réseaux filtrés par l'étiquette `kesh-repetition` : rien — ces
mêmes filtres ayant rendu des lignes au contrôle 9) ;
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
- **Authentification** *(ajoutée en remédiation P2 — R4/F7)* : après `_test/seed` (seul appel sans jeton, et celui qui
  crée les utilisateurs), `POST /api/v1/auth/login` avec `{"username":"admin","password":"admin123"}` — les identifiants
  du preset `with-company-no-fy`, **constantes du script**, écrites en tête avec leur source (`test_fixtures.rs`) : ce ne
  sont pas des secrets (preset de test, instance en `KESH_TEST_MODE` sur la boucle locale) et elles ne s'héritent pas de
  l'environnement, ce que les trois arguments interdisent. Le script lit `accessToken` du corps (camelCase) et le passe
  en `Authorization: Bearer <jeton>` à **chaque** appel suivant, export compris (jeton de session, non jeton d'accès
  personnel). Un login qui ne rend pas un 2xx, ou un corps sans `accessToken` non vide → refus, **sans appel suivant**.
  Ces identifiants sont aussi ceux de la reconnexion de la répétition jouée sur la fixture (AC4).
- **Contenu** : `POST /api/v1/_test/seed {"preset": "with-company-no-fy"}` ; exercices **2025 et 2026** ; comptes
  `1300 Actifs transitoires`, `1020 Banque` ; compte bancaire lié (IBAN **de test** de `fiscal-year-closing-journey.spec.ts`,
  `CH4431999123000889012`), créé **principal** (`isPrimary: true`, l. 181 de la spécification : le rendu du PDF l'exige)
  et relevé CAMT `camt053_v04_year_end.xml` (fixture existante, **non rapprochée** : la
  préparation doit la signaler) ; la **régularisation** de 250.00 au 31.12.2025 (débit 1300) ; **un contact client**
  (`POST /api/v1/contacts`, repris de `helpers/api-fixtures.ts` l. 39) ; **une facture client à ce contact, validée**,
  datée d'une **date fixe de 2026** (`2026-06-15`), non du jour comme le helper `createAndValidateInvoiceViaApi`, **puis
  figée** *(ajouté en remédiation P2 — F3 : la validation ne fige rien)* par `GET /api/v1/invoices/<id>/pdf`, qui doit
  rendre 200 et un corps commençant par `%PDF` — sinon refus ; c'est ce PDF figé qui est la seconde pièce du format 2 ;
  **un document fournisseur** — la fixture versionnée `frontend/tests/e2e/fixtures/spc_e2e_invoice.png`, **copiée** par
  le script dans la boîte (il ne fabrique aucune image ; R13 LOW) — importé (`POST /api/v1/inbox-import`), qui doit
  revenir dans `accepted[]` — sinon refus : c'est la poignée de main de `helpers/inbox.ts` l. 153-160, sans sonde
  distincte ; puis `GET /api/v1/admin/full-export` (`lib.rs` l. 251) vers `<dossier de sortie>/fixture.keshbackup`, dont
  l'en-tête `X-Kesh-Backup-Files` (nombre de pièces emportées, `routes/admin.rs` l. 39-40, 97) doit valoir **2** et
`X-Kesh-Backup-Files-Not-Included` **0** (sinon refus : une pièce n'a pas été emportée). Les routes et les corps sont **repris** de `fiscal-year-closing-journey.spec.ts`
  et de `backup-restore-new-install.spec.ts` ; l'appel d'export est **écrit sur la route** (les deux spécifications
  exportent par l'écran). Rien ne dépend du jour d'exécution. **Aucune donnée réelle**, aucun relevé du dossier
  `~/kesh-test/` (jamais ouvert).
- **Test** : `scripts/tests/repetition-fixture.test.sh`, sur le modèle de `scripts/tests/prepare-release.test.sh` (un
  `curl` **factice** en tête du `PATH`, qui trace ses appels, l. 6-13 du modèle) : argument manquant, URL non locale,
  sortie non vide, boîte non vide, instance injoignable (code de `curl` 7) ; `_test/seed` en **404** puis en **405** →
  refus **et aucun appel suivant** dans la trace ; en **200** → le script passe à l'appel suivant (la trace en porte un
  second) ; *(ajouté en remédiation P2, R4)* `_test/seed` en 200 puis **login en 401** → refus, et la trace s'arrête au
  login. Le test est lancé au gate par une enveloppe `crates/kesh-api/tests/repetition_fixture_script.rs`, calquée sur
  `crates/kesh-db/tests/prepare_release_script.rs` (un test shell que rien ne lance serait un test muet), **garde contre
  le test muet comprise** — l'enveloppe exige le résumé final du test shell (le modèle exige « aucune assertion en échec »)
  — ; un `jq` absent fait **rougir** le test, il ne le saute pas (R11 LOW). Le chemin nominal est exercé **au
  développement**, sur l'instance source réelle.

**AC9 — le journal d'exécution, hors dépôt.** `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-<AAAAMMJJ>.log` :
chaque commande du manuel **et sa sortie**, dans l'ordre, horodatée, avec le résultat des contrôles de l'AC3, les
constats de T3, et **trois essais volontaires du refus**, aucun ne lançant de `up` :
 (i) **le compose Synology** : `docker-compose.prod.yml` du tag, enregistré sous le nom `docker-compose.yml` dans un
     second dossier `/var/tmp/kesh-repetition-essai-<AAAAMMJJ>` (créé par le contrôle 3, comme la copie), avec un `.env`
     **synthétique** dont la `DATABASE_URL` vaut `mysql://kesh:essai@mariadb:3306/kesh` — ce compose n'a aucun `:?`
     (`grep -c ':?' docker-compose.prod.yml` → 0), la `DATABASE_URL` suffit à ce que `config` aboutisse et que seul le
     nombre de services décide (F15 LOW) — attendu : contrôle 7, `--services` rend `kesh-api` seul → **arrêt** ; le
     `grep` de `@mariadb:3306` **passe** (preuve qu'il ne suffit pas) ; les `source:` relatifs **passent** ; le dossier de
     l'essai est ensuite **détruit** par la garde de l'AC7 (R12, F15) ;
 (ii) **un nom de projet hérité** : `export COMPOSE_PROJECT_NAME=kesh-essai-314` (jamais `kesh`, ni le nom d'une pile
      existante) — attendu : contrôle 1 le liste → arrêt ; puis, la variable posée, `config` avec `-p kesh-repetition`
      rend `name: kesh-repetition` (constat de préséance) ; puis `unset`. Ce `config` se joue sur `$COPIE` **après le
      contrôle 5**, dont le `.env` satisfait les `:?` — sans quoi il échouerait pour une raison étrangère au constat (R12) ;
 (iii) **la garde du `rm -rf`** avec `COPIE` vide, puis relatif **et inexistant** (`kesh-repetition-absent-314`), puis
      un **lien symbolique** `/var/tmp/kesh-repetition-lien-<AAAAMMJJ>` vers un dossier vide créé pour l'essai — chaque
      fois depuis un dossier courant **vide** créé pour l'essai sous `/var/tmp`, jamais le worktree ni le dépôt, et le geste
      d'abord sous la forme `echo rm -rf …` — attendu : la garde arrête, rien n'est supprimé ; le lien et son dossier
      sont ensuite retirés (`rm` du lien, `rmdir` du dossier) (R12, F12 LOW).
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

**AC11 — la garde G22 : des présences, rien de plus.** *(Réécrit en remédiation P2 — C-31-4-14 ; la version de la P1,
par position, à seuils et à critère (e), est retirée : `manuel::normaliser` aplatit les sauts de ligne, si bien que ses
critères (c), (d) et (e) étaient indécidables ou passaient à vide — R3/F1 —, et (e) rougissait sur `docker compose ls`
— R2/F2.)* `crates/kesh-api/tests/textes_coherents.rs` gagne un test **G22** (ligne du tableau d'en-tête, issue : aucune
→ « Story 31-4 ») qui lit `admin-manual.tex`, en retire les commentaires LaTeX (`manuel::sans_commentaires`, l. 145 :
un jeton qui ne vivrait qu'en commentaire est absent du PDF — F1), borne la sous-section par
`manuel::section(&source, "sec:repetition-cloture")` (l. 84) **sur ce source brut, sans `manuel::normaliser`**, et en
extrait les listings par `manuel::listings` (l. 125), lignes conservées. Elle exige :
(a) **la sous-section existe** : `manuel::section` la trouve (il panique sinon : rouge, jamais « vert à vide ») ;
(b) **les présences**, chacune au moins une fois dans les listings : `-p kesh-repetition` ; `config --services` ;
    `/health` ; et la garde du `rm -rf` : `realpath -e`, `test ! -L` et `/var/tmp` ;
(c) **une absence** : aucune ligne de listing qui, hors commentaire shell (ligne dont le premier caractère non blanc est
    `#`), assigne une variable `KESH_*_HOST_DIR` — motif `(?m)^[^#\n]*\bKESH_[A-Z_]*HOST_DIR\s*=`, non ancré sur
    `KESH_` en tête de ligne, pour que `export KESH_LOG_HOST_DIR=…` ou `env KESH_X_HOST_DIR=…` soient vus (R3) ; nommer
    la variable en prose, pour dire de ne pas la poser, reste permis.
Les jetons de (b) vivent dans **une seule** constante (`JETONS_G22`), lue par une fonction pure
`manques_g22(&str) -> Vec<&'static str>` ; un test unitaire **retire en mémoire chaque jeton** de la sous-section réelle
et vérifie que `manques_g22` le rapporte (R11 : aucun jeton exigé sans sa mutation, sans éditer le `.tex` jeton par
jeton) ; un second test vérifie qu'une ligne `# KESH_LOG_HOST_DIR=…` ne déclenche pas (c), et qu'une ligne
`export KESH_LOG_HOST_DIR=…` la déclenche.
Le jeton littéral `docker compose config` de C-31-4-14 **n'est pas** exigé tel quel : le *principe directeur* place
`-p` et `--project-directory` entre `compose` et `config`, si bien qu'il n'apparaîtrait jamais dans un manuel juste ;
`config --services` en tient lieu *(précision de remédiation P2, versée au registre)*.
**Ce que G22 ne prouve pas, dit ici** : l'**ordre** des contrôles, la **forme** de chaque commande (drapeaux présents
partout), la **justesse** des gestes. Ces trois-là sont portés par le **journal d'exécution réelle** (AC9, T3) et par le
rejeu depuis le PDF (T8) — non par une garde lexicale, qu'on ne réétend pas (C-31-4-14). G19 (URL au tag) couvre déjà
l'URL de la sous-section, sans test de plus. **G14** ne lit pas les commandes de la sous-section (forme `-p`, angle mort
écrit de G14), et **plus aucune garde ne les lit une à une** : angle mort assumé, couvert par la T8. **G15** et **G16**
relisent la sous-section : lancer le binaire `configuration_transmise` au gate ciblé (T5).

**AC12 — CHANGELOG.** Entrée `Ajouté` dans la section `[0.15.0] — Non publié` : la répétition de clôture sur une copie,
au manuel d'administration, jouée sur une sauvegarde complète. ~~Aucune autre entrée.~~ *(Amendé en remédiation P2,
R6 item 6.)* Et une entrée dans une rubrique `### Corrigé` à créer dans cette section (elle n'en a pas encore,
`CHANGELOG.md` l. 13, 40, 60) : les manuels nomment **Administration → Exercices comptables** (AC13), sur le modèle de
l'entrée de #585 en 0.14.0 (`CHANGELOG.md` l. 106) — sans lien d'issue, aucune n'étant ouverte (C-31-4-13). Aucune
autre entrée.

**AC13 — le chemin des exercices comptables, écrit tel que le menu le range** *(ajouté en remédiation P2 — C-31-4-13,
élargi par R6).* Les quatre occurrences « \emph{Paramètres} → \emph{Exercices comptables} » — `user-manual.tex` l. 794
(colonne \emph{Actions}) et l. 845 (bouton \emph{Réouvrir}), `admin-manual.tex` l. 1445 et l. 1451 (bouton
\emph{Réouvrir}) — deviennent « \emph{Administration} → \emph{Exercices comptables} », le reste de chaque phrase
inchangé. Contrôle, à zéro résultat : `grep -nE 'Paramètres[^.]{0,60}Exercices comptables' docs/manual/fr/*.tex` ; et,
sur les **deux** PDF régénérés (`user-manual.pdf`, `admin-manual.pdf`), `pdftotext <pdf> - | tr '\n' ' ' | tr -s ' ' |
grep -c 'Paramètres → Exercices comptables'` rend `0` pour chacun (T4). La phrase de conventions de `user-manual.tex`
l. 59 (« \emph{Paramètres} → \emph{Utilisateurs} » donné en exemple) et les chemins internes à la page *Paramètres*
(`Paramètres → Modèles d'e-mail`, `Paramètres → Clés API`, … : des cartes de cette page) ne sont **pas** touchés : ils
désignent bien la page *Paramètres*. Aucune garde de hiérarchie n'est ajoutée (C-31-4-13).

## La marche prescrite (squelette ; le texte exact est écrit en exécutant)

Le manuel écrit, dans cet ordre, et **ne change rien à cet ordre sans l'avoir rejoué** :

0. Relever sur la production : la version (`GET /health`, champ `version`) ; si la répétition doit valoir test de
   restauration (AC6), l'export de souveraineté **et son SHA-256** *(ajouté en remédiation P2, F13)* ; **puis** produire
   le `.keshbackup` (*Administration → Sauvegarde complète*).
1. Contrôles 1 à 3 (version de Compose et environnement, hôte, dossier neuf sous `/var/tmp`, vide) ; **ensuite**
   seulement, placer l'exemplaire `$COPIE/source.keshbackup` (0600) (AC4).
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
*(Table amendée en remédiation P2 — C-31-4-14 : les mutations qui visaient l'ordre (M2), le critère (e) (M8) et un
jeton retiré de G22 (M9) sont **retirées**, leurs numéros gardés pour les renvois ; les lettres de G22 suivent l'AC11
réécrit.)*

| # | Mutation | Attendu |
|---|---|---|
| M1 | retirer du `.tex` **toutes** les occurrences de `config --services` dans la sous-section | G22 (b) rouge |
| M2 | *retirée (C-31-4-14 : G22 ne lit plus l'ordre)* | — |
| M3 | ajouter `KESH_DOCUMENTS_HOST_DIR=/volume1/kesh/documents` dans un `lstlisting` de la sous-section | G22 (c) rouge |
| M4 | renommer le label `sec:repetition-cloture` | G22 (a) rouge (sous-section introuvable, pas « vert à vide ») |
| M5 | écrire dans la sous-section une URL `raw.githubusercontent.com/…/main/…` | G19 rouge |
| M6 | réduire le refus de `_test/seed` à « 404 seulement » | `repetition-fixture.test.sh` rouge (cas 405), donc l'enveloppe Rust rouge |
| M7 | retirer du script le refus « dossier de sortie non vide » | idem |
| M8 | *retirée (C-31-4-14 : critère (e) retiré)* | — |
| M9 | *retirée (C-31-4-14 : « étapes 2 à 5 » n'est plus un jeton de G22 ; le renvoi se contrôle au PDF, T4)* | — |
| M10 | retirer du script le refus « boîte de dépôt non vide » | `repetition-fixture.test.sh` rouge |
| M11 | retirer du script le refus « URL hors boucle locale » | idem |
| M12 | retirer du `.tex` le `realpath -e` de la garde du `rm -rf` (F5) | G22 (b) rouge |
| M13 | retirer du `.tex` le `test ! -L` de la garde | G22 (b) rouge |
| M14 | ajouter `export KESH_LOG_HOST_DIR=/volume1/kesh/log` dans un `lstlisting` de la sous-section | G22 (c) rouge (motif non ancré, R3) |
| M15 | retirer du script le refus sur un login non 2xx | `repetition-fixture.test.sh` rouge (cas login 401), donc l'enveloppe Rust rouge |
| M16 | faire sortir le test shell en 0 avant son résumé final | l'enveloppe Rust rouge (garde contre le test muet, R11) |

Les autres jetons de G22 (b) — `-p kesh-repetition`, `/health`, `/var/tmp` — sont tués par le test unitaire de retrait en
mémoire (AC11), qui couvre **tous** les jetons de `JETONS_G22`, M1, M12 et M13 compris : les trois mutations de fichier
restent comme preuve de bout en bout sur le vrai `.tex`.

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
   réserve de l'AC10 (« image publiée »). Le script de fixture le rend peu coûteux (quelques minutes, hors verrou de gate
   hormis le `docker pull`). **Cette action n'est pas un critère du développeur** : elle appartient à la release.

## Tâches

- [ ] **T0 — état des lieux rejoué** : `git log -1 origin/main` ; `docker ps -a` (aucune pile `kesh-api` /
      `kesh-mariadb`) ; port libre ; `sudo -n true` (confirmer l'absence de `sudo`) ; base `kesh_314` créée et migrée
      depuis les migrations **du worktree**, port d'instance source attribué par l'orchestrateur ; dossiers de l'instance
      source sous `/home/gcorbaz/devel/kesh-gate-logs/314-source/` (AC8).
- [ ] **T1 — la fixture** (AC8) : écrire `scripts/repetition-fixture.sh`, **son test au `curl` factice** et l'enveloppe
      `crates/kesh-api/tests/repetition_fixture_script.rs` (rouge d'abord, M6, M7, M10, M11, M15, M16) ; la jouer contre
      l'instance source (`cargo run`, `KESH_TEST_MODE=true`, recette de `docs/testing.md`, avec la boîte propre) ; vérifier
      que le `.keshbackup` produit est de **format 2** et porte **deux pièces** — le document fournisseur et le PDF figé
      de la facture : en-tête `X-Kesh-Backup-Files: 2`, et deux entrées dans la liste des pièces du manifeste (lu, sans
      l'ouvrir ailleurs que dans le dossier temporaire de la story).
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
      (i) `docker compose ls -a` disponible dans la version installée, **sans** `-p` ;
      *(constats ajoutés en remédiation P2 — R1, R7, R9, R10, F8, C-31-4-14)*
      (j) `docker compose version --short` sur la station ; le comportement de `ports: !override` sous une Compose
      < 2.24 **si** une telle version est disponible sans installation (sinon : « non éprouvé », au manuel, AC10) ;
      (k) une clé dupliquée dans le `.env` : laquelle l'emporte (pour le dire au lecteur, contrôle 5) ;
      (l) le code de retour de `docker ps -a` quand le démon est injoignable (`DOCKER_HOST=unix:///nonexistent docker ps
      -a`, inoffensif) — l'attendu « réussit (code 0) » du contrôle 2 en dépend ;
      (m) l'étiquette `com.docker.compose.project=kesh-repetition` posée sur les conteneurs, le volume **et** le réseau
      après `up -d` (contrôle 9 : chaque filtre rend une ligne au moins) ;
      (n) la sortie de `config` avec l'override : **une seule** publication de `kesh-api`, `host_ip: 127.0.0.1`, aucune sur
      `mariadb` (contrôle 8), et `restart: "no"` pris en compte ;
      (o) `/var/tmp` sur la station : système de fichiers local, pas un lien (`realpath -e /var/tmp` → `/var/tmp`), et
      `mkdir -m 0700` sans `-p` qui échoue sur un dossier existant (contrôle 3).
      Un constat contraire à la fiche **change la procédure** avant d'écrire le manuel, et se dit au Dev Agent Record.
- [ ] **T4 — écrire la sous-section** à partir du journal, pas de la mémoire ; renvois **en prose** au manuel
      utilisateur, `\ref` seulement vers des labels du manuel d'administration (`sec:restauration-machine-neuve`,
      `sec:test-restauration`, posé ici) ; encadré AC10 ; *(ajouté en remédiation P2)* le renvoi de l. 1847 réécrit
      (AC1, F4) et les quatre chemins des exercices corrigés (AC13) ; `make fr` sous verrou — il régénère
      **`admin-manual.pdf` et `user-manual.pdf`**, tous deux commités ; contrôler le PDF **aplati**
      (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) : la sous-section, les neuf contrôles, l'URL au tag, la phrase « ne se
      joue jamais sur le NAS », « étapes 2 à 5 », **aucun `??`** dans la sous-section ni dans l'encadré de
      `sec:backup-dsm` (renvoi de l. 1847) ; et, sur **chacun des deux PDF**, `grep -c 'Paramètres → Exercices comptables'`
      sur la sortie aplatie → `0`, avec `grep -nE 'Paramètres[^.]{0,60}Exercices comptables' docs/manual/fr/*.tex` → aucun
      résultat (AC13).
- [ ] **T5 — G22** (AC11), tests rouges d'abord (M1, M3, M4, M5, M12, M13, M14 ; le test de retrait en mémoire de
      chaque jeton), puis verts ; G19 relancée ; binaire `configuration_transmise` (G14, G15, G16) relancé.
      *(Liste amendée en remédiation P2 : elle portait « M1 à M5, M8, M9 », dont M2, M8 et M9 sont retirées.)*
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

La sauvegarde automatique (31-1c) ; la correction des noms et du port en dur du compose (issue n° 1) ; une répétition
**sur le NAS** ; un E2E de la procédure (elle est une procédure d'exploitation sur Docker, hors du périmètre de
Playwright) ; la traduction (le manuel d'administration n'existe qu'en français) ; le manuel utilisateur **hors du
chemin des exercices** (AC13) ; une garde de hiérarchie des chemins de menu ; toute extension de G22 au-delà des
présences (C-31-4-14) ; #404 et #235 (P1, nommés à la rétrospective de l'Epic 31) ; #671. *(Amendé en remédiation P2 :
« le chemin de menu du manuel utilisateur (issue n° 2) » et « le manuel utilisateur » en sortent, C-31-4-13.)*

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

## Dev Agent Record

*(à remplir au développement : gates réellement exécutés avec leurs chiffres, journal de répétition, constats de T3,
captures, mutations tuées, écarts à la fiche.)*
