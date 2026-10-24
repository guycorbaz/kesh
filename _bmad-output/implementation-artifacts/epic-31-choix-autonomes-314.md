# Epic 31 — registre des choix autonomes de la Story 31-4 (la répétition à blanc d'une clôture)

Versé par union dans `epic-31-choix-autonomes.md` au merge.

## Story 31-4

### C-31-4-1 — Une story de documentation exécutée, avec un script et une garde, sans code produit

- **Contexte** : le cahier des charges de la 29-7 est « doc seulement, écrite en l'exécutant ». Deux manques en
  découlent : il faut une sauvegarde synthétique de format 2 reproductible (sans quoi la procédure ne se rejoue pas à
  la release), et rien n'empêche que les contrôles anti-écrasement (R1, R2) s'effacent du manuel au fil des éditions.
- **Retenu** : le manuel (`.tex` et PDF), un script de fixture (`scripts/repetition-fixture.sh`) et son test sans
  serveur, une garde de texte G22, l'entrée CHANGELOG. Aucune modification du produit, des compose ni de `.env.example`.
- **Écartées** : (a) le manuel seul, sans garde — R1 (HIGH) est le défaut qui a fait retirer la story ; le laisser tenir
  par la seule vigilance reproduit le mode d'échec ; (b) corriger les compose (noms et port en dur) dans cette story —
  cela touche l'installation de Guy et sort du périmètre ; demandé en issue.
- **Réversible** : oui.

### C-31-4-2 — La sauvegarde synthétique se fabrique par un script contre une instance source en `KESH_TEST_MODE`

- **Contexte** : il faut un `.keshbackup` de format 2, avec deux sortes de pièces, sans aucune donnée réelle.
- **Retenu** : un script `bash` + `curl` + `jq`, qui reprend à la lettre les appels HTTP de
  `fiscal-year-closing-journey.spec.ts` et de `backup-restore-new-install.spec.ts` (`_test/seed` d'abord), contre une
  instance dev `cargo run` en mode test sur la base `kesh_314`. Il refuse de tourner hors `KESH_TEST_MODE`.
  *(Amendé en validation P1, 2026-10-24 — R7, R8, F8, F9 : « à la lettre » devient « repris » ; voir C-31-4-11.)*
- **Écartées** : (a) saisir la fixture à la main par l'interface — non reproductible ; (b) une spec Playwright
  « fixture » — alourdit la suite E2E pour un besoin d'exploitation, et la suite a déjà ses 7 à 10 échecs attendus ;
  (c) exporter un jeu de `~/kesh-test/` — jamais (données réelles).
- **Réversible** : oui (le script est un fichier).

### C-31-4-3 — La répétition ne se joue jamais sur l'hôte qui porte la production

- **Contexte** : `docker-compose.yml` fixe `container_name: kesh-api` / `kesh-mariadb` ; le compose Synology fixe
  `kesh-api`. Deux piles ne coexistent pas sur un hôte, et une collision partielle est le terrain exact de R1.
- **Retenu** : la procédure se joue sur une **autre machine** ; contrôle d'hôte (`docker ps -a`) avant tout ; le nom de
  projet Compose est posé (`COMPOSE_PROJECT_NAME=kesh-repetition`) pour que `down -v` ne vise que la copie.
  *(Amendé en validation P1, 2026-10-24 — R1 HIGH, F3, F4 : le nom de projet n'est plus confié au `.env`, que le
  téléchargement de `.env.example` écrasait et qu'un `COMPOSE_PROJECT_NAME` du shell dominait ; il est porté par `-p
  kesh-repetition` sur chaque commande (C-31-4-10). Le contrôle d'hôte vérifie le code de retour de `docker ps`, les
  noms exacts `kesh-api`/`kesh-mariadb` et les étiquettes du projet, au début **et** à la fin.)*
- **Écartées** : surcharger les `container_name` et le réseau par un override pour cohabiter sur un même hôte — **non éprouvé** (la fiche ne l'affirme ni ne le promet) et plus risqué que d'avoir un hôte distinct, la collision partielle étant le terrain de R1 ; le défaut (noms et port en dur) est demandé en issue.
- **Réversible** : oui.

### C-31-4-4 — Au développement, l'image est celle de `main` ; l'image publiée est rejouée à la release

- **Contexte** : le format 2 (31-1a, 31-1b) n'existe dans aucune image publiée avant la 0.15.0. La procédure s'écrit
  pourtant maintenant, et avant la clôture de 2026.
- **Retenu** : le `docker-compose.override.yml` épingle `image:` ; au développement, sur `gcorbaz/kesh:314` construite
  depuis le worktree ; le manuel dit « la version relevée » ; le **rejeu sur l'image publiée** est demandé à la
  release (action à l'orchestrateur), et la réserve est écrite au manuel (AC10) tant qu'il n'a pas eu lieu.
  *(Précisé en validation P1, 2026-10-24 — F, faits vérifiés : `gcorbaz/kesh:314` affiche `0.14.0` à `/health` tant que
  les crates ne sont pas bumpées, alors que `gcorbaz/kesh:0.14.0` publiée refuse le format 2 (`manifest.rs` l. 25) ; au
  développement, l'image locale se substitue à `<version relevée>` et `docker image inspect` au `pull` (T8). L'argument
  « l'anti-downgrade refuse une copie plus ancienne » est retiré : voir C-31-4-12.)*
- **Écartées** : attendre la 0.15.0 pour écrire la fiche — fait glisser une story qui précède la clôture de 2026 sans
  gain ; écrire sans exécuter — c'est le mode d'échec que la story existe pour éviter.
- **Réversible** : oui.

### C-31-4-5 — La destruction offre un chemin sans `sudo`

- **Contexte** : les dossiers `log/ inbox/ documents/ backup/` de la copie sont créés par Docker, en root ; le cahier
  des charges ne prévoit que `sudo` (F5). La station des agents n'a pas de `sudo` (`sudo -n true` refusé) ; un lecteur du
  manuel peut ne pas en avoir non plus.
- **Retenu** : deux chemins écrits : `sudo rm -rf` et un conteneur jetable (`docker run --rm -v … --entrypoint sh <image
  déjà tirée> -c 'rm -rf …'`), garde de chemin non vide. L'image est celle de MariaDB, déjà présente sur la machine.
- **Écartées** : un `chown` récursif par `sudo` — même besoin de `sudo` ; faire tourner le conteneur sous l'utilisateur
  de l'hôte (`user:` dans l'override) — change le comportement du produit (le conteneur écrit en root par conception).
- **Réversible** : oui.

### C-31-4-6 — La copie n'écoute que sur la boucle locale

- **Contexte** : `docker-compose.yml` publie `"80:80"` sur toutes les interfaces ; la copie porte les données réelles de
  Guy sur sa station. Le port 80 peut aussi être pris.
- **Retenu** : `ports: !override ["127.0.0.1:<port>:80"]` dans le `docker-compose.override.yml` de la copie
  (Compose ≥ 2.24 ; essai à blanc de `docker compose config` réussi en 2.40.3) ; le manuel nomme la version minimale.
  *(Précisé en validation P1, 2026-10-24 : l'override est **nommé** par un second `-f` — avec un `-f` explicite,
  Compose ne le charge plus d'office (constat T3 (b)).)*
- **Écartées** : changer `KESH_PORT` — ne change pas la publication ; éditer le compose téléchargé — le contrôle de
  l'AC3 porte alors sur un fichier modifié à la main et perd son sens.
- **Réversible** : oui.

### C-31-4-7 — Une garde G22 épingle les contrôles, pas l'exactitude

- **Contexte** : `textes_coherents.rs` tient déjà G19 (URL au tag) et G16/G20 sur des jetons du manuel.
- **Retenu** : G22 exige les jetons `@mariadb:3306`, `docker compose config`, `COMPOSE_PROJECT_NAME`, l'ordre
  contrôle-avant-`up`, l'absence de `*_HOST_DIR` assigné dans un listing, et une positive adossée (sous-section non vide).
  Elle est dite *limitée* : l'exactitude de la procédure n'est portée que par le journal d'exécution.
  *(Amendé en validation P1, 2026-10-24 — R3, F10, R L3 : jetons `config --services`, `name: kesh-repetition`,
  `@mariadb:3306` (second filet), `docker context`, `/health`, `étapes 2 à 5` ; ordre lu **par position** dans les seuls
  listings, avec une relecture `config` exigée entre l'override et `up -d` ; positive chiffrée (60 lignes, 8 commandes) ;
  (e) nouvelle : chaque `docker compose` des listings porte `-p kesh-repetition` et `--project-directory`. G14 étant
  aveugle à la forme `-p` (angle mort écrit), c'est G22 (e) qui tient ces commandes.)*
- **Écartées** : aucune garde (voir C-31-4-1) ; une garde qui « exécute » les commandes du manuel — elle exigerait
  Docker en CI et un réseau.
- **Réversible** : oui.

### C-31-4-8 — La restauration est un renvoi, pas une recopie

- **Contexte** : la 31-1b a écrit `Restaurer sur une nouvelle machine` (`sec:restauration-machine-neuve`), qui décrit
  `/setup`, l'assistant et l'import.
- **Retenu** : la répétition renvoie à cette section et ne redit que ce qui est propre à la copie ; placée juste après
  `Test de restauration périodique`, qu'elle complète (même exigence OLICo art. 10 al. 1).
  *(Amendé en validation P1, 2026-10-24 — R4, F6 : le renvoi ne vaut que pour les **étapes 2 à 5** de la section ; son
  étape 1 reprend `KESH_SMTP_*` et les dossiers de l'ancien `.env`, l'inverse de la copie, et le manuel le dit en toutes
  lettres. R L5 : la répétition ne vaut test OLICo que si l'export de souveraineté et son SHA-256 sont faits avant les
  gestes de clôture.)*
- **Écartées** : recopier la marche (deux textes à tenir d'accord — DRY) ; la placer en *Mise à jour* (elle n'est pas une
  mise à jour).
- **Réversible** : oui.

### C-31-4-9 — Indépendance de la 31-1c

- **Contexte** : la sauvegarde automatique se développe en parallèle.
- **Retenu** : la procédure prend un `.keshbackup` quel qu'il soit ; elle ne cite pas la 31-1c. Si elle est mergée avant le
  développement **et vue fonctionner**, le dev peut ajouter une phrase ; sinon, rien.
- **Écartées** : décrire une fonction non mergée — le manuel affirmerait ce qui n'a pas tourné.
- **Réversible** : oui.

### C-31-4-10 — Chaque commande Compose porte `-p`, `--project-directory` et des `-f` explicites (validation P1)

- **Contexte** : R1 (HIGH) de la validation P1 — le nom de projet posé dans le `.env` était écrasé par le téléchargement
  de `.env.example` et dominé par un `COMPOSE_PROJECT_NAME` du shell (que `scripts/synology/recette.sh` l. 33 exporte) ;
  sans `-f`, Compose cherche un compose dans les dossiers parents ; `DOCKER_HOST` ou un contexte distant fait jouer la
  copie ailleurs. Une copie dont le projet prendrait le nom `kesh` viserait le service `mariadb` du compose de dev.
- **Retenu** : `docker compose -p kesh-repetition --project-directory "$COPIE" -f "$COPIE/docker-compose.yml" [-f
  "$COPIE/docker-compose.override.yml"] …` sur **chaque** commande ; `$COPIE` absolu, nommé `kesh-repetition-*` ;
  neutralisation préalable de `COMPOSE_*`, `DOCKER_HOST`, `DOCKER_CONTEXT`, `KESH_*`, `MARIADB_*` et lecture de
  `docker context inspect` (adresse `unix://`). Les comportements de Compose invoqués sont des **constats T3**.
- **Écartées** : (a) `COMPOSE_PROJECT_NAME` dans le `.env` — c'est le défaut ; (b) `name: kesh-repetition` dans
  l'override seul — une variable d'environnement l'emporte ; (c) une fonction ou un alias shell qui porte les options —
  plus court, mais la garde G22 (e) ne lirait plus chaque commande, et une session sans la fonction retomberait sur les
  défauts implicites.
- **Réversible** : oui.

### C-31-4-11 — La fixture prend sa boîte en argument, refuse tout non-2xx, et son test tourne au gate (validation P1)

- **Contexte** : R7, R8, F8, F9 — le dépôt d'un document écrivait dans `KESH_INBOX_DIR`, par défaut
  `/tmp/kesh-e2e/inbox` (partagé) ; hors mode test, `_test/seed` rend 404 **ou** 405 (repli `ServeDir`) ; la facture du
  helper est datée du jour ; les spécifications exportent par l'écran ; M6 n'avait aucun test qui la tue ; un test shell
  que rien ne lance se tait.
- **Retenu** : trois arguments (`<url> <sortie> <boîte>`) sans défaut ni héritage ; refus de l'URL non locale, des
  dossiers non vides, de l'instance injoignable et de **tout** code non 2xx de `_test/seed` ; poignée de main sur
  `accepted[]` ; facture au `2026-06-15` ; export écrit sur `GET /api/v1/admin/full-export` ; test au `curl` factice
  (404, 405, 200) lancé par `crates/kesh-api/tests/repetition_fixture_script.rs`, calqué sur
  `crates/kesh-db/tests/prepare_release_script.rs`.
- **Écartées** : (a) refuser sur « 404 » — un 405 passerait ; (b) un faux serveur HTTP — plus lourd que le `curl` factice
  du précédent `prepare-release.test.sh` ; (c) laisser le test hors gate, lancé à la main en T7 — test muet.
- **Réversible** : oui.

### C-31-4-12 — La version de la copie se contrôle à `/health`, elle ne se déduit pas de l'anti-downgrade (validation P1)

- **Contexte** : R9, F1 — l'import ne refuse que si `kesh_version_min_required` de la source dépasse le binaire
  (`kesh-db/src/version.rs` l. 270-275) ; une sauvegarde de 0.15.1 s'importe dans une 0.15.0 sans table nouvelle. La
  fiche affirmait le contraire.
- **Retenu** : contrôle 9 de l'AC3 — après `up -d`, `version` de `/health` égale **exactement** la version relevée sur la
  production ; sinon `down -v` et arrêt. `pull kesh-api` avant `up` pour qu'une version mal écrite échoue au tirage, et
  non par une construction tentée dans `$COPIE`.
- **Écartées** : (a) garder l'argument — faux ; (b) n'exiger qu'une version « supérieure ou égale » — la répétition
  montrerait d'autres gestes que ceux de la production, sans le signaler.
- **Réversible** : oui.

### C-31-4-13 — Le chemin « Paramètres → Exercices comptables » du manuel utilisateur, corrigé par la 31-4 (choix de l'orchestrateur, 2026-10-24)

- **Contexte** : la remédiation P1 demandait une issue (P3, `bug`) : `docs/manual/fr/user-manual.tex` l. 794 et 845 écrit
  « \emph{Paramètres} → \emph{Exercices comptables} », alors que le menu range l'écran dans le groupe **Administration**
  (`frontend/src/routes/(app)/+layout.svelte`, groupe `nav-administration`, item `nav-fiscal-years`). Vérifié par
  l'orchestrateur. C'est la famille de #585 (Story 30-3), que la garde `les_chemins_cites_aux_catalogues_existent` ne voit
  pas : elle contrôle que chaque libellé existe, non qu'il appartienne au groupe cité.
- **Retenu** : la 31-4, qui renvoie justement à ces pages du manuel utilisateur, corrige les deux occurrences (et toute
  autre « Paramètres → <item du groupe Administration> » que le grep par la forme rendrait), PDF régénéré. Correction
  mécanique (budget d'issues, `CLAUDE.md` § *Priorités des défauts*). Une garde de hiérarchie n'est **pas** ajoutée ici.
- **Écartée** : ouvrir une issue — le compte est à 137 pour un plafond de « moins de 136 » au tag.
- **Réversible** : oui.
