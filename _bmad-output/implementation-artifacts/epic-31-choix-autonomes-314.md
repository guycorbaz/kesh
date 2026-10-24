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
- **Écartées** : (a) saisir la fixture à la main par l'interface — non reproductible ; (b) une spec Playwright
  « fixture » — alourdit la suite E2E pour un besoin d'exploitation, et la suite a déjà ses 7 à 10 échecs attendus ;
  (c) exporter un jeu de `~/kesh-test/` — jamais (données réelles).
- **Réversible** : oui (le script est un fichier).

### C-31-4-3 — La répétition ne se joue jamais sur l'hôte qui porte la production

- **Contexte** : `docker-compose.yml` fixe `container_name: kesh-api` / `kesh-mariadb` ; le compose Synology fixe
  `kesh-api`. Deux piles ne coexistent pas sur un hôte, et une collision partielle est le terrain exact de R1.
- **Retenu** : la procédure se joue sur une **autre machine** ; contrôle d'hôte (`docker ps -a`) avant tout ; le nom de
  projet Compose est posé (`COMPOSE_PROJECT_NAME=kesh-repetition`) pour que `down -v` ne vise que la copie.
- **Écartées** : surcharger les `container_name` et le réseau par un override pour cohabiter sur un même hôte — **non éprouvé** (la fiche ne l'affirme ni ne le promet) et plus risqué que d'avoir un hôte distinct, la collision partielle étant le terrain de R1 ; le défaut (noms et port en dur) est demandé en issue.
- **Réversible** : oui.

### C-31-4-4 — Au développement, l'image est celle de `main` ; l'image publiée est rejouée à la release

- **Contexte** : le format 2 (31-1a, 31-1b) n'existe dans aucune image publiée avant la 0.15.0. La procédure s'écrit
  pourtant maintenant, et avant la clôture de 2026.
- **Retenu** : le `docker-compose.override.yml` épingle `image:` ; au développement, sur `gcorbaz/kesh:314` construite
  depuis le worktree ; le manuel dit « la version relevée » ; le **rejeu sur l'image publiée** est demandé à la
  release (action à l'orchestrateur), et la réserve est écrite au manuel (AC10) tant qu'il n'a pas eu lieu.
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
- **Écartées** : changer `KESH_PORT` — ne change pas la publication ; éditer le compose téléchargé — le contrôle de
  l'AC3 porte alors sur un fichier modifié à la main et perd son sens.
- **Réversible** : oui.

### C-31-4-7 — Une garde G22 épingle les contrôles, pas l'exactitude

- **Contexte** : `textes_coherents.rs` tient déjà G19 (URL au tag) et G16/G20 sur des jetons du manuel.
- **Retenu** : G22 exige les jetons `@mariadb:3306`, `docker compose config`, `COMPOSE_PROJECT_NAME`, l'ordre
  contrôle-avant-`up`, l'absence de `*_HOST_DIR` assigné dans un listing, et une positive adossée (sous-section non vide).
  Elle est dite *limitée* : l'exactitude de la procédure n'est portée que par le journal d'exécution.
- **Écartées** : aucune garde (voir C-31-4-1) ; une garde qui « exécute » les commandes du manuel — elle exigerait
  Docker en CI et un réseau.
- **Réversible** : oui.

### C-31-4-8 — La restauration est un renvoi, pas une recopie

- **Contexte** : la 31-1b a écrit `Restaurer sur une nouvelle machine` (`sec:restauration-machine-neuve`), qui décrit
  `/setup`, l'assistant et l'import.
- **Retenu** : la répétition renvoie à cette section et ne redit que ce qui est propre à la copie ; placée juste après
  `Test de restauration périodique`, qu'elle complète (même exigence OLICo art. 10 al. 1).
- **Écartées** : recopier la marche (deux textes à tenir d'accord — DRY) ; la placer en *Mise à jour* (elle n'est pas une
  mise à jour).
- **Réversible** : oui.

### C-31-4-9 — Indépendance de la 31-1c

- **Contexte** : la sauvegarde automatique se développe en parallèle.
- **Retenu** : la procédure prend un `.keshbackup` quel qu'il soit ; elle ne cite pas la 31-1c. Si elle est mergée avant le
  développement **et vue fonctionner**, le dev peut ajouter une phrase ; sinon, rien.
- **Écartées** : décrire une fonction non mergée — le manuel affirmerait ce qui n'a pas tourné.
- **Réversible** : oui.
