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
  *(Précisé en remédiation P2, 2026-10-24 — R1, F8 : la version ≥ 2.24 devient un contrôle (`docker compose version
  --short`, contrôle 1) ; le contrôle 8 exige **une seule** publication de `kesh-api`, `host_ip: 127.0.0.1`, aucun
  `published: "80"` ni `0.0.0.0` — sous une Compose qui fusionnerait les listes, `80:80` subsisterait et l'ancien
  contrôle passait. R9 LOW : l'override pose aussi `restart: "no"`.)*
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
  *(Remplacé en validation P2, 2026-10-24, par C-31-4-14 : G22 se réduit à des présences dans le source brut ; ordre,
  seuils et (e) sont retirés, et la forme de chaque commande n'est plus tenue que par le journal et la T8.)*
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
  *(Précisé en remédiation P2, 2026-10-24 — F4 : la place « juste après » est **maintenue**, et le renvoi relatif
  « cf. sous-section précédente » de l'encadré de `sec:backup-dsm` (`admin-manual.tex` l. 1847), qui désignerait la
  nouvelle sous-section, devient `\S\ref{sec:test-restauration}`. Écartée : placer la sous-section après
  `sec:backup-dsm` — elle s'éloignerait du test qu'elle peut remplacer, et un renvoi relatif resterait à la merci du
  prochain déplacement.)*
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
  *(Précisé en remédiation P2, 2026-10-24 — R4/F7 : la fixture s'authentifie par `POST /api/v1/auth/login` avec
  `admin`/`admin123`, identifiants du preset écrits en **constantes** du script (preset de test, instance en mode test
  sur la boucle locale : pas un secret), et passe `accessToken` (corps camelCase) en `Authorization: Bearer` ; un login
  refusé arrête tout, cas ajouté au test (M15). Écartés : un quatrième argument ou une variable d'environnement — le
  mot de passe d'un preset de test n'a rien à protéger, et l'environnement est ce que les trois arguments excluent.
  F3 : la validation d'une facture ne fige pas son PDF ; la fixture crée un contact et un compte bancaire **principal**,
  puis appelle `GET /api/v1/invoices/<id>/pdf` (200, `%PDF`) et exige `X-Kesh-Backup-Files: 2` à l'export. R13 :
  le document fournisseur est `frontend/tests/e2e/fixtures/spc_e2e_invoice.png`, copié. R11 : l'enveloppe Rust reprend
  la garde contre le test muet du modèle.)*
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
- *(Élargi et précisé en remédiation P2, 2026-10-24 — R6.)* Le même chemin s'écrit dans le **manuel d'administration**,
  `admin-manual.tex` l. 1445 et 1451 : la story le corrige aussi (AC13) ; `grep -nE 'Paramètres[^.]{0,60}Exercices'
  docs/manual/fr/*.tex` rend ces quatre lignes et elles seules ; T4 régénère et contrôle `user-manual.pdf` et
  `admin-manual.pdf` (grep à zéro résultat sur le PDF aplati). **Précision de fait, à porter à l'orchestrateur** :
  l'ancien chemin n'est pas une impasse — la page *Paramètres* (`/settings`, rangée sous *Administration*) porte une carte
  *Exercices comptables* et son bouton *Gérer* (`routes/(app)/settings/+page.svelte` l. 582-590, Story 3.7). Le défaut
  est l'écart entre deux chemins pour un même écran, celui des quatre lignes n'étant pas l'entrée du menu ; la
  correction reste juste. Les chemins `Paramètres → Modèles d'e-mail`, `→ Clés API`… désignent des cartes de cette page
  et ne sont pas touchés. Entrée `Corrigé` au CHANGELOG (AC12).

### C-31-4-14 — Validation P2 : la garde G22 réduite, le dossier de la copie fixé hors de `$HOME` (choix de l'orchestrateur, 2026-10-24)

- **Contexte** : la validation P2 (Sonnet) rend 5 MEDIUM (R) et 6 MEDIUM (F) ; les deux lentilles convergent sur la garde
  G22 telle que la remédiation P1 l'a étendue (lecture « par position », seuil de 60 lignes et 8 commandes, critère (e)
  « chaque `docker compose` porte `-p` et `--project-directory` ») : `manuel::normaliser` aplatit les lignes, (c)/(d)/(e)
  deviennent indécidables, (e) rougirait sur `docker compose ls`/`version`. C'est le même motif que le garde de la 31-2
  (C-31-2-15) : une garde lexicale qu'on étend pour qu'elle « prouve » la procédure finit par ne rien prouver.
  Et le contrôle 3 (`findmnt`) ne peut pas reconnaître un dossier synchronisé par Synology Drive Client (dossier local).
- **Retenu** :
  1. **G22 se réduit à des présences**, lues dans le source brut de la sous-section (`manuel::section`, avant
     normalisation) : les jetons `-p kesh-repetition`, `docker compose config`, `--services`, `/health`, la garde du
     `rm -rf`, et l'**absence** de toute ligne active qui assigne une variable `KESH_*_HOST_DIR`. Plus d'ordre par position,
     plus de seuils, plus de critère (e). Ce que G22 ne prouve pas — l'ordre et la justesse des gestes — est porté par le
     **journal d'exécution réelle** (T3, T8), et la fiche le dit.
  2. **Le dossier de la copie vit sous `/var/tmp/kesh-repetition-<date>`**, créé par `mkdir` **sans** `-p` (échec s'il
     existe), hors de `$HOME` — donc hors de tout dossier que Synology Drive Client synchronise ; le contrôle 3 vérifie
     ce préfixe et l'absence de lien symbolique (`test ! -L`), et la garde du `rm -rf` compare le `realpath` à ce préfixe.
     Les téléchargements du navigateur y sont déplacés avant usage.
- **Écartées** : poursuivre l'extension de G22 (chaque passe en trouve le défaut suivant) ; détecter la synchronisation
  (aucun moyen fiable).
- **Réversible** : oui.
- *(Précisé au report dans la fiche, remédiation P2, 2026-10-24.)* (1) Le jeton littéral `docker compose config` ne
  peut pas figurer dans un manuel conforme au *principe directeur* (`-p` et `--project-directory` s'intercalent entre
  `compose` et `config`) : G22 exige `config --services` à sa place, avec `-p kesh-repetition`, `/health`, et pour la
  garde du `rm -rf` les jetons `realpath -e`, `test ! -L` et `/var/tmp`. Les jetons tiennent dans une constante unique
  et un test les retire un à un **en mémoire** (R11). Les commentaires LaTeX sont retirés avant la lecture
  (`manuel::sans_commentaires`, F1). Le motif d'absence est non ancré (`export KESH_…_HOST_DIR=` vu, R3).
  (2) Le dossier se crée par `mkdir -m 0700` : `/var/tmp` est partagé entre utilisateurs et la copie porte les données
  réelles. La garde du `rm -rf` compare `dirname` à `/var/tmp`, le nom au préfixe, exige `test -d`, `test ! -L` et
  l'égalité du `realpath -e` au chemin écrit (F5, R8) ; la vacuité se vérifie au contrôle 3, avant le premier fichier
  posé (F11). (3) Mutations retirées : M2 (ordre), M8 (critère (e)), M9 (jeton « étapes 2 à 5 », non retenu par ce
  choix ; le renvoi se contrôle au PDF) ; ajoutées : M12 à M16.

### C-31-4-15 — Signal de recyclage en validation P3 : retrait de l'échafaudage, la procédure seule (choix de l'orchestrateur, 2026-10-24)

- **Contexte** : P1 Opus 0/1/19 MEDIUM → P2 Sonnet 0/0/11 → P3 Opus R 0/0/7, F 0/1/3 : la sévérité ne baisse plus, et la
  grande majorité des findings de P2 et P3 portent sur ce que les remédiations ont **ajouté autour** de la procédure — le
  script de fixture et son test shell, l'enveloppe Rust qui le lance, la garde G22, leurs mutations (le HIGH de P3 : la
  fixture ne peut pas figer un PDF sur le preset de test). C'est le **recyclage** du `CLAUDE.md` (§ *Règle de splitting*,
  amendement D5) ; le signal est **déclaré au Project Lead** dans le compte rendu de l'orchestrateur. La 31-4 est une story
  de **documentation** : son livrable est une procédure **exécutée**, non un outillage.
- **Retenu — retrait** :
  1. **Plus de script de fixture, ni de test shell, ni d'enveloppe Rust, ni de garde G22** (G19, existante, suffit pour
     les URL). La sauvegarde synthétique est **exportée par l'écran** (*Administration* → sauvegarde) depuis une instance
     de **démonstration** de la station (« Charger la démo »), au format 2 ; le journal d'exécution dit combien de pièces
     elle porte — aucune exigence de PDF figé.
  2. **La justesse de la procédure est portée par son exécution réelle**, journalisée hors dépôt (T3, T8), et par la
     relecture du manuel (`.tex` et PDF) en revue de code.
  3. **Restent exigés**, parce qu'ils protègent la production : la garde du `rm -rf` écrite exactement (`case … in
     kesh-repetition-*) ;; *) false ;; esac` dans le `if`, `test -d`, `test ! -L`, `realpath -e`, parent `/var/tmp`), essai
     avec un **nom faux** ; le contrôle 8 borné au bloc `ports:` de `kesh-api` (le `0.0.0.0` de `KESH_HOST` n'est pas une
     publication) ; tous les fichiers sensibles (exemplaire `.keshbackup`, exports de souveraineté de la production et de la
     copie, téléchargements) **déplacés dans `$COPIE`** et nommés à la destruction ; l'**arrêt** défini (destruction de
     l'AC7, puis reprise au contrôle 1) et l'exemplaire réel posé **après** le contrôle 9.
  4. **C-31-4-13** : les chemins « Paramètres → Exercices comptables » de l'**application** (messages ×4, `notify.ts`,
     `opening-balances`) restent valides — la page *Paramètres* porte une carte *Exercices comptables* — et sont écrits
     **hors périmètre** ; seuls les manuels sont corrigés, et l'entrée `Corrigé` du CHANGELOG dit « les manuels ».
- **Écartées** : redécouper la 31-4 (le recyclage est confiné à l'échafaudage) ; poursuivre (chaque passe trouvait le
  défaut suivant de ce qu'on venait d'ajouter).
- **Réversible** : oui — une fixture scriptée pourra revenir avec sa propre story.

### C-31-4-16 — Validation P3 remédiée : ce que le retrait de C-31-4-15 emporte, et ce qui reste (agent de remédiation, 2026-10-24)

- **Contexte** : C-31-4-15 retire l'échafaudage. Il en découle que C-31-4-2 (script de fixture), C-31-4-7 (G22), C-31-4-11
  (arguments et refus de la fixture) et le point 1 de C-31-4-14 (G22 réduite) sont **sans objet** ; ils restent écrits
  ci-dessus comme relevés datés, non réécrits.
- **Retenu** :
  1. **Source de la sauvegarde** : une instance de démonstration ordinaire (assistant, étape *démonstration*), exportée par
     l'écran. Ce que la démonstration porte n'est pas établi à la fiche : le développeur le relève au journal (T1) et saisit
     à l'écran une régularisation 1300/2300 si elle manque. Aucune exigence de PDF ni de nombre de pièces ; le critère de
     l'import devient « 0 pièce manquante ».
  2. **Les gardes qui restent** relisent la sous-section sans qu'on les étende : G19 (M5), G14, G15, G16 (T5).
  3. **Aucune dette nouvelle** : les dix sites de l'application (C-31-4-15, point 4) sont déjà nommés ; aucune issue
     n'est demandée (budget d'issues, C-31-4-13), le résidu est écrit en *Hors périmètre* et à l'AC13.
- **Écartées** : conserver M12/M13 réécrites en « toutes les occurrences » (elles protégeaient une garde retirée) ;
  rattacher une issue P4 aux dix sites (le compte des issues ne baisse pas).
- **Réversible** : oui.

### C-31-4-17 — Validation P4 remédiée : la source de la sauvegarde se prépare à l'écran, et rien de sensible ne précède `$COPIE` (agent de remédiation, 2026-10-24)

- **Contexte** : P4 (Sonnet), R 0/0/3/5 et F 0/0/1/6, aucun finding réfuté. Trois constats de fond, tous nés de C-31-4-15/16
  ou de leur lecture. (1) `kesh_seed::seed_demo` (`crates/kesh-seed/src/lib.rs` l. 139-143, 272-296) ne pose que la société,
  le plan PME, **un exercice**, les réglages de facturation et les taux de TVA : ni écriture, ni exercice suivant, ni pièce ;
  C-31-4-16 point 1 (« 0 pièce manquante », *N* possiblement nul) rendait le critère de l'AC4 **vrai par construction** et
  la source inapte au geste 2 de l'AC5. (2) L'« arrêt » sur le reste d'une répétition antérieure visait un `$COPIE` qui
  n'existe pas encore. (3) L'étape 0 déposait le `.keshbackup` de la production et l'export de souveraineté dans le
  dossier de téléchargements **avant** tout contrôle.
- **Retenu** :
  1. **T1 prescrit des gestes à l'écran, journalisés**, sur l'instance de démonstration avant l'export : exercice suivant ;
     écriture 1300/2300 du dernier jour de l'exercice courant ; au moins une pièce (fixture synthétique du dépôt
     `spc_e2e_invoice.png`, déposée dans le dossier d'import de l'instance source puis *Documents à traiter → Importer le
     dossier*) ; PDF figé d'une facture **si l'écran le permet**, sinon « non éprouvé ». **Critère de l'AC4 : *N* ≥ 1 et 0
     manquante** ; *N* = 0 est un échec de préparation de la source. Le renvoi aux étapes 2 à 5 de *Restaurer sur une
     nouvelle machine* dit lesquelles de leurs vérifications s'appliquent. Cela **remplace le point 1 de C-31-4-16**
     (l'entrée reste écrite, non réécrite).
  2. **L'étape 0 se joue après les contrôles 1 à 3** et enregistre ses fichiers directement dans
     `$COPIE/telechargements/` (créé par `mkdir -m 0700`, sans `-p`, après la vérification de vacuité). Un arrêt aux
     contrôles 1 ou 2 n'a donc rien à nettoyer ; après l'étape 0, l'arrêt retire à la main un exemplaire atterri dans le
     dossier du navigateur (repli du constat (s)).
  3. **L'arrêt sur reste d'une répétition antérieure** a un chemin écrit : dossier existant → `COPIE` y est affectée et
     (b) s'applique avec sa garde ; dossier disparu → `docker rm -f` / `volume rm` / `network rm` appliqués aux seuls
     identifiants rendus par les filtres d'étiquette, éprouvé **à blanc** (constat (u)), exécution réelle « non éprouvée »
     (AC10).
  4. LOW : R4 (`mkdir` de `telechargements`), R5 (contrôle 1 seule issue corrective), R6 (un seul `if` pour les deux
     gestes sans `sudo` ; deux refus de plus constatés à l'essai (iii)), R7 (annotation datée au Change Log), R8 (trois
     variables `DOCKER_*` de plus), F2 (`COPIE` rétablie après chaque cas), F3 (port ≠ 80, > 1023), F4 (ôter le `#`
     des deux `MARIADB_*`), F5 (machine Linux), F6 (en-tête lu à l'onglet Réseau, à constater), F7 (commandes en
     listing).
- **Écartées** : un script de remise à zéro des restes (C-31-4-15 : aucun outillage) ; un `docker compose -p kesh-repetition
  down -v` sans `-f` (le geste que le principe directeur s'interdit ; à constater peut-être en T3, non autorisé ici) ;
  `prune`.
- **Réversible** : oui.

### C-31-4-18 — Validation P5 remédiée : R3 propagé jusqu'au bout, et le critère du manuel n'est pas celui du développement (agent de remédiation, 2026-10-24)

- **Contexte** : P5 (Opus, passe ciblée sur le seul commit de la remédiation P4), T 0/0/2/5, aucun finding réfuté. (1) R3
  (étape 0 après le contrôle 3, enregistrement direct dans `$COPIE/telechargements/`) laissait trois énoncés à l'ancien
  état : la parenthèse « aucun secret réel dans `/var/tmp` » (fausse depuis R3), l'exemplaire « créé hors de `$COPIE` »,
  le « déplacé » du contrôle 3. (2) C-31-4-17 avait écrit à l'AC4, qui décrit **le manuel**, un critère (*N* ≥ 1, échec de
  préparation, renvois à la pièce et au PDF de la T1) qui ne vaut que pour la sauvegarde synthétique : une production sans
  pièce y aurait constaté un « échec », et la vérification des écritures et des factures de l'étape 5 avait disparu.
- **Retenu** :
  1. Le secret réel est dans `/var/tmp` dès l'étape 0 ; la garantie est la **destruction de `$COPIE` à tout arrêt**
     (AC7 (d)). `source.keshbackup` se pose par `mv` depuis `$COPIE/telechargements/` après le contrôle 9, puis `chmod
     0600`. Le « hors de `$COPIE` » ne reste que pour le repli du constat (s).
  2. **Deux critères séparés** : au manuel, 0 pièce manquante et autant de pièces restaurées que l'export en a annoncé,
     plus les vérifications de l'étape 5 (écriture récente, liste des factures) ; au développement (T1, Dev Notes), *N*
     ≥ 1. Cela **précise** le point 1 de C-31-4-17 (l'entrée reste écrite, non réécrite).
  3. LOW T3 à T7, mécaniques (voir le Change Log) ; le repli (i) → (ii) est écrit et (i) rangé parmi les « non
     éprouvés » de l'AC10.
- **Écartées** : une garde lexicale sur l'ordre des étapes (C-31-4-15 : aucun outillage) ; abandonner l'égalité avec le
  compte de l'export, plus forte que *N* ≥ 1 et valable dans les deux cas (une lecture de deux nombres à l'écran).
- **Réversible** : oui.

### C-31-4-19 — Étape 0 du développement : le prérequis « exporte en format 2 » se lit sans rien télécharger (agent de développement, 2026-10-24)

- **Contexte** : la P8 (validation terminée au plafond) a laissé deux MEDIUM corrigés par l'orchestrateur sans passe de
  vérification. Recontrôle : la correction de M2 proposait de vérifier le prérequis par « la version affichée » (fausse au
  développement : l'image de `main` affiche `0.14.0` et exporte en format 2) ou par « un export d'essai » (il déposerait
  un secret hors de `$COPIE`, avant le contrôle 1).
- **Retenu** : le prérequis se lit sur la page *Administration → Sauvegarde complète* de la production, dont la
  description nomme « les pièces justificatives » depuis la 31-1a (absente à la 0.14.0, vérifié par `git show v0.14.0`).
  Le nombre de pièces annoncé se lit à l'onglet Réseau (`X-Kesh-Backup-Files`) ou dans `manifest.json` : l'écran ne
  l'affiche pas. LOW mécaniques des P6 et P8 appliqués ; le reste consigné au Dev Agent Record.
- **Écartées** : la version (≥ 0.15.0) comme seul critère (imposerait une substitution de plus au développement, et rien
  ne la lit sans `/health`) ; l'export d'essai.
- **Réversible** : oui (texte de fiche et de manuel).

### C-31-4-20 — Développement : ce que la répétition réelle a changé à la procédure (agent de développement, 2026-10-24)

- **Contexte** : la procédure jouée réellement (constats à blanc, répétition, second passage depuis le PDF) a contredit la
  fiche en six points (Dev Agent Record, *Développement*).
- **Retenu** :
  1. contrôle 1 étendu à `DATABASE_URL` (exportée par le profil du shell de la station ; le compose Synology la lit) ;
     filtre écrit en deux lignes (`COMPOSE_|KESH_|MARIADB_|DATABASE_URL=` puis `DOCKER_…`) pour tenir dans la page ;
  2. contrôles 7 et 8 lus en JSON (`config --format json | jq`), override en YAML « en ligne » : le texte d'un PDF ne
     conserve pas les espaces ; `jq`, `unzip`, `curl`, `openssl` contrôlés au contrôle 1 ;
  3. boucle d'attente de `/health` après `up -d` ;
  4. nombre de pièces : onglet Réseau, ou `unzip -l … | grep -cE ' files/[^ ]+$'` ; exports de souveraineté nommés
     `souverainete-production.zip` / `souverainete-copie.zip` et comparés **table par table** (le SHA-256 global diffère
     par construction) ; la répétition ne prétend pas remplir l'étape 3 du test périodique à la lettre ;
  5. pilotage de l'écran par Chromium de Playwright (Firefox snap non pilotable ; son accès à `/var/tmp` par le portail
     « non éprouvé ») ; facture de la démonstration validée à 0 % ;
  6. listings de la sous-section en `\footnotesize`, dans un groupe local.
- **Écartées** : ajouter une garde lexicale (C-31-4-15) ; corriger l'étape 3 du test périodique dans cette story (défaut
  voisin, non mécanique : issue demandée) ; régler le compte de TVA de la démonstration (code du produit : issue demandée).
- **Réversible** : oui (texte du manuel).

### C-31-4-21 — Revue de code P1 : le commit `9c59dec4` retiré, l'étape 3 du test périodique renvoyée à #709 (orchestrateur, 2026-10-24)

- **Contexte** : `9c59dec4` réécrivait l'étape 3 de `sec:test-restauration` (comparaison table par table au lieu du
  SHA-256 de l'archive) et ajoutait une seconde entrée « Corrigé » au CHANGELOG — exactement ce que C-31-4-20 écartait
  (défaut voisin non mécanique). La lentille A (MEDIUM-1 à 3) l'a relevé : contradiction avec la fiche, l'AC6 et l'AC12,
  gate complet antérieur au commit, critère d'un contrôle légal (OLICo art. 10 al. 1) relâché sur la foi d'une seule
  exécution de démonstration.
- **Retenu** : retrait par `728012ba` (revert) ; le point devient l'**issue #709** (étape 3, SHA-256 inatteignable).
  Les findings A-1, A-2, A-3, E-3 (MEDIUM-3) et la partie de B-9 qui visaient ce commit sont **sans objet**. Le manuel
  revient à la rédaction de C-31-4-20 : la répétition ne prétend pas remplir l'étape 3 à la lettre.
- **Écartées** : garder le commit et amender fiche, AC et registre (l'arbitrage du critère revient à Guy, non à une passe).
- **Réversible** : oui (texte du manuel ; #709 tranche).

### C-31-4-22 — Revue de code P1 remédiée : les variables lues dans le fichier, les comparaisons qui ne se taisent plus, les cibles nommées (agent de remédiation, 2026-10-24)

- **Contexte** : P1 (Sonnet, lentilles B, E, A), hors `9c59dec4` : quatre MEDIUM retenus (E-1/B-3, E-2/B-2, B-1, E-4).
- **Retenu** :
  1. **Variables interpolées : l'ensemble clos, lu dans le fichier.** Au tag `v0.14.0`, `git show v0.14.0:docker-compose.yml
     | grep -oE '\$\{[A-Za-z_]+' | sort -u` rend 45 noms : 39 `KESH_*`, 4 `MARIADB_*`, `RUST_LOG`, et `NOM` (dans un
     commentaire, l. 108) ; aucune variable sans accolades, aucune entrée `environment:` sans valeur (qui transmettrait la
     valeur du terminal), pas d'`env_file`. Seule `RUST_LOG` échappait au filtre. Le filtre du contrôle 1 (et du contrôle
     final) gagne `RUST_LOG=` ; surtout, le contrôle 4 relève, **dans le compose téléchargé**, chaque `${NOM` / `$NOM`
     et affiche ceux que le terminal exporte (`printenv`) — attendu : rien. Ce n'est plus une énumération de préfixes,
     c'est l'inventaire des sites du fichier même, valable pour le compose de la 0.15.0 et des suivantes. Essayé sur une
     copie synthétique du compose du tag (`env -i … RUST_LOG=debug KESH_LANG=fr`) : `posée : KESH_LANG`, `posée : RUST_LOG`.
  2. **Comparaison table par table** : la boucle écrit `production : N tables` / `copie : N tables` par `test -s` et
     `wc -l` ; une ligne absente = fichier d'empreintes vide = comparaison **non faite** (le `diff` de deux fichiers vides
     se tait) ; deux `N` différents = exports de tables différentes. Essayé sur deux ZIP synthétiques : cas normal (deux
     lignes, une différence `audit_log.csv`) et archive de la copie absente (une seule ligne).
  3. **Cibles nommées** : l'étape 0 nomme le `.keshbackup` `production.keshbackup` (commandes de comptage et de pose
     reprises) ; les replis `mv` nomment leur cible (`souverainete-production.zip`, `production.keshbackup`,
     `souverainete-copie.zip`) — le navigateur propose le même nom aux deux exports de souveraineté (journal T8). `mv -n`
     écarté : sous coreutils 9.7, il ne déplace pas et rend 0 sans rien dire (essayé).
  4. **Étape 0 ordonnée** : l'export de souveraineté puis, aussitôt, le `.keshbackup`, sans écriture entre eux ; une
     écriture intercalée apparaît comme la différence d'une autre table qu'`audit_log.csv` (dit aux deux endroits).
  5. LOW traités : « Arrêt » renvoie à *En cas d'arrêt*, reprise avec un nouveau dossier (date ou suffixe `-2`) (B-4,
     E LOW-1) ; trois listings séparés pour la destruction — sans `sudo`, avec `sudo`, `rm -rf` gardé seul avant le premier
     démarrage —, « un seul se tape » (B-5, L5) ; exemples `0.14.0` déclarés tels face au prérequis 0.15.0 — l'URL reste au
     tag `v0.14.0`, G19 l'exige (B-8) ; volume et réseau nommés (L1) ; espace disque renvoyé à sa source, « jusqu'à
     environ trois fois » dans `/var/tmp`, décompte corrigé (L3) ; « sept formes » (L4) ; « suivez chaque lien » (L7) ;
     « ne modifie aucune donnée » au lieu de « ne touche jamais » (B-6).
- **Écartées** : les LOW consignés au Change Log (B-7, B-10, E LOW-2 à LOW-5, A L2, L6).
- **Réversible** : oui (texte du manuel).
