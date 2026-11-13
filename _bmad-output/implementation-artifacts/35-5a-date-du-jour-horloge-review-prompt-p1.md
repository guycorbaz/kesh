# Prompt — revue de code P1 de la Story 35-5a (refs #1022, #907, #929), trois lentilles, première passe complète

Tu relis la **Story 35-5a** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **la date du jour devient celle de Zurich** pour ce qui écrit dans les livres ou les borne — solde
d'ouverture d'un compte bancaire et complément des *Soldes de départ* (#1022), verrou de période (#907, serveur et écran),
contre-passation et annulations qui contre-passent (règlement client, facture fournisseur, règlement fournisseur, rapprochement),
premier exercice de l'assistant (#929, en partie). La plomberie (horloge injectable, `kesh-db` qui reçoit la date) est celle de la
35-5a0, close ; cette story **bascule** `Horloge::aujourd_hui()` sur `jour_suisse` (`chrono-tz`), corrige l'écran du verrou, convertit
les tests qui atteignent ces sites, et prouve la nuit **par exécution** (`KESH_TEST_INSTANT`). **Aucune migration.** Kesh tient des
livres réels : une date fausse ici est une écriture dans le mauvais jour, voire le mauvais exercice, la nuit du 1er janvier.

Worktree `/home/gcorbaz/devel/kesh-e355a`, branche `story/35-5a-date-du-jour-horloge`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e355a diff eabafed5f c5972ca13` (`eabafed5f` = 35-5a0 close, base ; `c5972ca13` = dernier commit de
code ; le commit suivant ne porte que la fiche, le registre, le PDF, `sprint-status.yaml` et ce prompt). Lis la production en entier
(elle est courte), les tests neufs en entier, les conversions par échantillon raisonné (axe 4). Langue du rapport : français. Modèle :
**Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-5a-date-du-jour-horloge.md` en entier (*Ce qui se lit au code*, D1 à D7, AC1 à AC12, le verdict
des 38 fichiers et le second tableau, mutations, *Dev Notes*, **Change Log** — entrée « Développement » — et **Dev Agent Record**) ; la
section *Transmis à la 35-5a* et la synthèse de clôture de `35-5a0-horloge-plomberie.md` ; les registres `e35-choix-autonomes-e350.md`
(C-e350-16, -22, -23, -28, -29) et `e35-choix-autonomes-e355a.md` (C-e355a-1 à -7). Ne prends pas le Dev Agent Record pour argent
comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e355a/` : `rouges.log` et
`rouges-*.log`, `verts-*.log`, **`nuit.log`** (synthèse d'AC9) et `nuit-{N0,N1,N2,C0…C9}.log` (environnement et commit en tête),
`mut/*.log`, `gate-*.log`, `mcp-isole-*.log`, `captures.log`, `manuel-make.log`. Les scripts `*.sh`, `*.py`, `*.cjs` sont à **lire**,
jamais à exécuter.

**Livrables visibles à contrôler** : le manuel `docs/manual/fr/user-manual.tex` (§ *La date du jour est celle de Zurich, même la nuit*,
`:737` ; renvois `:711`, `:1013`, `:2488`) **et son PDF** `docs/manual/fr/user-manual.pdf` — à lire aplati :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (lecture seule, sortie à l'écran, sans redirection) ; les
captures `kesh-gate-logs/e355a/captures/*.png` (thème clair, écran du verrou à 00 h 30 le 11 mars, à 00 h 30 le 1er janvier, et de
jour) ; `docs/api-external.md:272` ; `CHANGELOG.md`, section `## [0.17.7] — Non publié`.

⚠️ **L'E2E (Playwright) n'a pas encore tourné** sur cette story : il se joue à la clôture, avec `gate-ci`. Le compte rendu le dit.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ;
  ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`,
  ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**,
  `sed -i`, `playwright`, `vitest`. `pdftotext` est permis, sortie à l'écran.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext … -`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **`jour_suisse` (D1, AC1)** : le sens instant → jour local n'est-il jamais ambigu ? `chrono_tz::Europe::Zurich` est-il la base
   précompilée (aucune lecture de `TZ` ni de `/usr/share/zoneinfo`), et la version au `Cargo.lock` celle de `rxing` ? Les douze
   instants d'AC1 couvrent-ils les deux changements d'heure et les bornes à la seconde ? Un instant **à la seconde même** du passage
   à l'heure d'été ou d'hiver (01:00:00Z) manque-t-il ? Le compte de la garde de `horloge.rs` reste-t-il 1 ?
2. **Un seul jour par requête, et le bon** : la bascule touche-t-elle bien **tous** les neuf sites et **eux seuls** ? Les 19 sites de
   la 35-5b (UTC) sont-ils intacts ? Une requête qui mêlerait un site 35-5a (jour suisse) et un site 35-5b (jour UTC) dans une même
   décision (ex. échéance jugée UTC, contre-passation jugée suisse ; accueil UTC et exercice du jour suisse) : la nuit, que se
   passe-t-il ? Cherche-la au code (`state.horloge.aujourd_hui()` contre `Utc::now()` dans un même handler ou service).
3. **L'écran du verrou (D7, AC4)** : `maxLockDate` = veille de `todayLocal` sur les champs locaux — le 1er mars, le 1er janvier, le
   changement d'heure ? Le Vitest épingle-t-il le fuseau et ne fausse-t-il que `Date` ? L'allowlist de `local-date-guard.test.ts`
   (huit → sept) et ses sondes, qui désignaient l'entrée par **position** (`ALLOWLIST[3]`) et la désignent désormais par fichier :
   chaque sonde vise-t-elle encore ce qu'elle dit viser ? L'écran (jour du poste) et le serveur (jour de Zurich) peuvent-ils
   diverger pour un poste réglé ailleurs, et le manuel le dit-il ?
4. **Les conversions de tests (AC9)** : les huit fichiers convertis à `aujourd_hui_test()` (`ada7e1d61`) et `period_lock_e2e`
   (`c5972ca13`) : une date **de pièce** fixe a-t-elle été remplacée par le jour (ce qui changerait ce que le test prouve) ? Un test
   qui fixe l'horloge de son état **et** appelle `aujourd_hui_test()` (deux horloges) ? `period_lock_e2e` : l'exercice du montage qui
   part du 1er janvier d'il y a trente jours change-t-il un attendu (nom, bornes, « hors exercice » à 1990) ?
5. **Les tests neufs** (`jour_suisse_nuit_e2e.rs`, AC5 dans `fiscal_years_e2e.rs`, AC7 dans `horloge_etat.rs`, le test retourné de
   `horloge_routes_e2e.rs`) : chaque assertion peut-elle rougir sur le code d'avant (le relevé `rouges.log` le dit-il, test par
   test) ? Le montage de deux exercices posé en SQL (exercice ramené à 2025) est-il cohérent avec les gardes du dépôt (exercices qui
   se chevauchent, clôtures dans l'ordre) ? Le refus d'AC3 (`400 FISCAL_YEAR_INVALID`) et AC7 sans exercice 2026 (C-e355a-3, écart à
   la lettre) : la preuve vise-t-elle encore le rang 5 **au jour suisse** ?
6. **L'inventaire par exécution (AC9, `nuit.log`)** : refais le tri à partir des journaux — chaque rouge de C2, C2b, C3, C4, C8, C9
   est-il classé avec la ligne lue, et le classement tient-il au code ? La « sonde positive » (N1 à `c2e3e788c`, 2 rouges) est-elle
   réelle (journal, commit, absence sans instant) ? L'écart à la lettre d'AC9 (C-e355a-7 : rouges des deux instants de la fiche
   attribués aux dates fixes de 2026) est-il juste, ou l'un d'eux cache-t-il un vrai mélange de deux jours ? Les angles morts sont-ils
   complets ? Les « bombes de 2027 » sont-elles bien indépendantes de cette story ?
7. **La garde (AC8)** : 39 → 31 entrées, recompte-les ; chaque fichier retiré compte-t-il 0 à la garde ; l'entrée de
   `journal_entries.rs` requalifiée 35-5c avec la ligne lue ; aucune forme neuve dans les fichiers de la story (`jour_suisse_nuit_e2e.rs`
   compte 0 ?).
8. **Docs et manuel (AC11)** : la phrase unique « la date du jour est celle de Zurich, même la nuit » est-elle vraie pour **chaque**
   élément qu'elle énumère, et fausse pour aucun site resté en UTC (les sites 35-5b, qu'elle dit « suivront ») ? Le PDF porte-t-il le
   texte du `.tex` (aplati) et aucun « temps universel » / « encore la veille » ? Les renvois `\S\ref{sec:date-du-jour}` mènent-ils à
   la bonne section (numéro rendu dans le PDF) ? Le CHANGELOG dit-il vrai (ce qui change, ce qui reste en UTC) ? `api-external.md`
   ailleurs : une phrase « datée du jour » qui devrait maintenant dire Zurich ?
9. **Mutations** : M1, M2, M3, M4, M5, M7, M9 tuées chacune par le test **nommé** au journal (`mut/`) ? Propose des mutations
   plausibles non jouées (fuseau `Europe/Paris` — même décalage : la mutation serait équivalente, est-ce dit ? ; l'écran à `d - 2` ;
   `unlock_books` au jour UTC).

## Lentille B — Blind (le diff seul, sans la fiche)

Relis `kesh-core/src/horloge.rs`, `kesh-core/Cargo.toml`, `+page.svelte` (`maxLockDate`), `jour_suisse_nuit_e2e.rs`,
`common/horloge.rs`, les conversions. Cherche : une date du jour lue deux fois ou par deux horloges ; un test qui passe **à vide** (son
attendu vrai avec ou sans la story) ; une date de pièce prise pour le jour ; une sonde qui ne vise plus rien.

## Lentille E — Edge (chaque branche, chaque borne)

00:59:59 / 01:00:00 Zurich (hiver) et 01:59:59 / 02:00:00 (été) ; la nuit du changement d'heure ; le 1er janvier avec et sans
exercice suivant ouvert ; un exercice décalé (1er juillet – 30 juin) la nuit du 1er juillet ; le verrou posé « jusqu'à hier » à minuit
pile ; l'écran sur un poste en `America/New_York` ; le Vitest exécuté dans une CI en UTC. Chaque cas : comportement au code, test qui
le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC12 : livré, testé, au lieu prescrit ? Les écarts déclarés (AC7 sans exercice 2026, AC9 et les instants de la
fiche, aucune spec E2E convertie) sont-ils exacts et sans effet sur ce que l'AC veut prouver ? L'étape 0 (LOW V5-1 à V5-3, points
transmis par la 35-5a0, F3, inventaire D5) citée ligne lue ? Les gates du passage en `review` (`gate-test-fast.log`, `gate-ci.log`,
`gate-front-*.log`) : chaque chiffre au journal, au commit `c5972ca13`, horodaté après lui ? Le Dev Agent Record n'affirme-t-il que ce
qui a tourné (E2E : **non lancé**) ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e355a-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque
rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue
copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la
liste**.
