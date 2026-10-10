# Prompt — validation P2 de la spec, Story 26-3 (manuels et garde de kesh-restore)

*Versionné le 2026-10-10. Passe **complète**, deux lentilles (**Opus**), contexte frais chacune, en lecture seule.
Rotation (D6, rétro 25) : P1 Sonnet → P2 Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-26-3`, branche `story/26-3` (base `fe162504`, = v0.13.0 + kickoff de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/26-3-manuel-et-restauration-synology.md`. **Remédiation à relire :
commit `f5c4a958`** (`git show f5c4a958`) — elle applique les findings de la P1 (rapports
`/home/gcorbaz/devel/kesh-gate-logs/26-3-validate-p1-{R,F}.md` ; bilan au Change Log de la fiche). Epic :
`_bmad-output/planning-artifacts/epic-26-recette-v0-13.md`. Issues #616, #617, #619 (`gh api
repos/guycorbaz/kesh/issues/N` et `.../N/comments`). Registre : `epic-26-choix-autonomes.md` (C-26-3-1 à 12 ; 9 à 12
nés de la P1). Règles : `CLAUDE.md` (§ Review Iteration Rule, § Le prompt d'une passe doit NOMMER le manuel, § Inventorier
les sites NON RÉSOLUS, § Propagation post-patch). Fiches voisines : `../kesh-26-1`, `../kesh-26-2` (lecture seule).

**Le motif à chasser d'abord** : sur ce projet, *la sévérité se déplace vers ce qu'on vient d'écrire*. La P1 n'a
trouvé que des défauts d'origine ; la P2 doit chercher en priorité ceux que la remédiation `f5c4a958` a **introduits**.

Axes prioritaires :

1. **Exécution raisonnée de la garde prescrite (AC 5), cas par cas — axe non exercé en P1.** Sans rien exécuter,
   écris la boucle telle que l'AC 5.2-5.4 la décrit et déroule-la à la main sur des sorties `docker ps --format
   '{{.State}}|{{.Image}}|{{.Label "com.docker.compose.service"}}|{{.Names}}'` que tu construis : hôte sans conteneur
   (liste vide), conteneur hors compose (étiquette vide), image affichée par identifiant (cas réel relevé au poste :
   `running|407fccb51e71|mariadb|kesh-mariadb-dev`), `docker.io/gcorbaz/kesh@sha256:…`, deux projets avec le même
   service, `restarting`, `paused`, `removing`, `created`. La règle « ligne à cinq champs = refus » et le champ
   `RESTE` : un `|` peut-il apparaître légitimement dans l'un des champs ? `set -euo pipefail` + `read` qui rend 1 à
   la dernière ligne sans saut de ligne + `<<<` : la boucle se termine-t-elle comme prévu, et le compteur survit-il ?
   Les libellés de l'AC 5.3/5.4 respectent-ils G16 (c) (`(?m)(?:^\s*|[;&|]\s*)docker compose\b`, un seul `$1`) —
   le libellé de l'AC 5.4 contient-il un `|` ou un `;` suivi de `docker compose` ?
2. **La section 4-bis de la recette (AC 6), réécrite en P1** : chaque cas est-il **montable** avec les fonctions
   prescrites (`compose_factice(service, image, commande)`, orphelin sans `--remove-orphans`, `down`) ? Déroule
   l'état des conteneurs du projet `kesh-recette-synology` cas après cas (R1 → R7, R5-bis) ; le refus en aval par
   `SAUVEGARDE_BASE=autre` est-il **atteint** dans les cas « la garde passe » (étapes 1 et 2 du script : verrou
   libre ? `.cnf` présent ?) et **jamais** dans les cas de refus ; les motifs grepés sont-ils discriminants ;
   l'état de sortie suffit-il aux sections 5 à 9 (`recette.sh:180-288`) ; les trois mutations annoncées ne
   donnent-elles bien que les rouges listés ? Le pré-vol et l'étiquette `gcorbaz/kesh:recette-synology-<8 hex>` :
   sûrs pour une image `gcorbaz/kesh` préexistante sur le poste ? `docker rmi` d'une étiquette sur une image encore
   référencée par un conteneur arrêté : échoue-t-il, et `nettoyer()` dans quel ordre ?
3. **G16 et G20 telles qu'amendées (AC 7, AC 9)** : la forme sensible à la casse de G20 attrape-t-elle toutes les
   écritures du défaut (`\textbf{Compte de banque}`, `« Compte de banque »` avec `~`, espaces fines, retour à la
   ligne au milieu) et **aucune** forme juste ? La lecture des **quatre** catalogues (continuation de lignes,
   anti-test-muet > 1000 clés : 2242 clés par locale sur `fe162504`) ; chaque mutation de G16 ne fait-elle rougir que
   sa clause (G16 agrège ses erreurs : la mutation (a) touche-t-elle aussi l'ordre des étapes ou (g) ?).
4. **Sémantique DE / EN / IT — axe non exercé en P1** : les cibles de l'AC 4 pour `bank-import-labels-drop-zone` et
   `bank-import-labels-page-title` dans `de-CH`, `en-CH`, `it-CH` (valeurs actuelles aux lignes `:1016`, `:1018`) ;
   la fiche dit seulement « traductions de même sens » — est-ce implémentable sans deviner ? Cohérence avec
   `docs/i18n-glossaire.md` (colonnes DE/IT/EN, `:151`) et avec `nav-bank-import` dans les quatre locales.
5. **Le manuel, contre le code — et le PDF** : chaque phrase que l'AC 1, 2, 3, 8 fait écrire est-elle vraie
   (formulaires du plan comptable, `/bank-accounts`, écran d'import, présélection du profil, `docker compose up
   --no-start` sur Compose v2 de DSM, `docker inspect` sur le nom du conteneur, préfixe de projet des réseaux) ?
   Inventaire du symptôme sur les **PDF aplatis** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, non fait en P1
   pour la brochure) ; les valeurs `occ` « avant » de l'AC 10 (refais-en au moins trois avec la fonction `occ` de
   la fiche, sortie standard seulement).
6. **Propagation de la remédiation** : la fiche et le registre se contredisent-ils encore (AC 5 vs Dev Notes vs
   Technique vs C-26-3-3/5/8 amendés ; « R1-R6 » résiduel ; `docker compose create` ; G20 « insensible ») ?
   Recompte : 12 AC, tâches T0-T8, décomptes du Change Log (R 2H/5M/7L, F 2H/5M/6L ; « 2 HIGH et 8 MEDIUM distincts,
   13 LOW ») contre les rapports P1.
7. **Implémentable et gates** : CHANGELOG à une seule section `[0.13.1]` après fusion, conflits 26-1/26-2 (lis leurs
   fiches), E2E complet au dernier commit de code, recette au dernier commit qui touche `scripts/synology/`.

## Lentilles

- **R — Auditeur d'acceptation** : `.claude/skills/bmad-create-story/checklist.md` ; chaque finding P1 réellement
  résolu par `f5c4a958` (vérifie-les un par un, avec preuve) ; chaque phrase des trois issues couverte par un AC
  testable, chaque AC par une tâche, chaque tâche par un test ; faits cités revérifiés.
- **F — Full-scope adversary** : la conception remédiée ; ce que la garde laisse encore passer ; ce que la recette
  prouverait à vide ; ce que le manuel corrigé affirmerait de faux ; défauts **introduits** par `f5c4a958`.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée),
correction proposée, et pour chacun : **origine** — défaut d'origine, ou introduit par `f5c4a958`. Toute affirmation
de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un
« 0 finding » sans elle ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/26-3-validate-p2-<R|F>.md` ;
dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh` et les scripts Synology `kesh-dump.sh`, `kesh-restore.sh`,
`recette.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`, `cargo`,
`npm`, `npx`, `docker` (y compris `docker ps` : le déroulé de l'axe 1 est **raisonné**), `bash` sur un script du
dépôt, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture,
`pdftotext` vers la sortie standard.
