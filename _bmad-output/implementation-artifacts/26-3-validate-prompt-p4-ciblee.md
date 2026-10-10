# Prompt — validation P4 CIBLÉE de la spec, Story 26-3 (manuels et garde de kesh-restore)

*Versionné le 2026-10-10. **Passe ciblée** (§ « La passe ciblée » du `CLAUDE.md`) : **une seule lentille**, **Haiku**,
contexte frais, lecture seule, braquée sur **le seul commit de la remédiation P3**. Haiku est réservé à ce type de passe
(décision D6, rétrospective de l'Epic 25).*

Dépôt `/home/gcorbaz/devel/kesh-26-3`, branche `story/26-3`. Fiche :
`_bmad-output/implementation-artifacts/26-3-manuel-et-restauration-synology.md`. **Commit à relire, et lui seul :
`e591b066`.** Lis-le **aplati**, en un seul diff contre son parent — `git show e591b066` (un seul commit : pas de
risque d'indexation multi-commit) — puis, pour chaque ligne du diff, le **fichier final** (`sed -n` sur la fiche et le
registre `epic-26-choix-autonomes.md`) : les numéros de ligne du diff ne sont pas ceux du fichier.

Contexte utile, à lire seulement s'il éclaire le diff : rapports de la P3
`/home/gcorbaz/devel/kesh-gate-logs/26-3-validate-p3-{R,F}.md` ; Change Log de la fiche (entrée « Validation P3
remédiée »). Le motif chassé : *la sévérité se déplace vers ce qu'on vient d'écrire* — cherche les défauts que
`e591b066` **introduit**, pas ceux de la conception d'origine.

## Lentille unique — chasseur de régression de la remédiation P3

Axes, dans cet ordre :

1. **AC 6 (au premier chef)** — la section 0 de la recette contre le faux `docker` :
   - la portée de la fonction : « définie **et** exportée dans le sous-shell de chaque cas », `FAUX_*` exportées,
     `SAUVEGARDE_*` désarmées puis posées, `2>&1`, assertion `type -t docker` = `file` — y a-t-il une contradiction
     entre ces règles, ou avec la ligne de lancement écrite plus bas dans l'AC 6 ?
   - `RECETTE_SECTION=0` : la recette sort-elle avant toute commande Docker (la `trap nettoyer EXIT` de
     `scripts/synology/recette.sh:32-37` reste-t-elle inoffensive dans ce mode) ?
   - les **23 cas S1-S23** : pour **chacun**, déroule à la main la garde prescrite par l'AC 5 (lecture
     `IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE` ; ligne vide ignorée ; ligne illisible refusée ; service =
     `SAUVEGARDE_SERVICE` → compteur `N=$((N+1))` et `case` d'état ; sinon image
     `^(docker\.io/)?gcorbaz/kesh([:@].*)?$` → `case` d'état ; après la boucle, refus si `N` = 0) et vérifie la
     **classe** annoncée ;
   - le **tableau des huit mutations** (O, F, F′, Z, I, R, D, T) : pour chacune, l'ensemble des cas rouges est-il
     **exact** — ni un de plus, ni un de moins — et le total rouges + verts vaut-il 23 ? La mutation (O) est la garde
     de `git show fe162504:scripts/synology/kesh-restore.sh` (lignes 75-79), qui ne lit que l'étiquette `kesh-api` et
     admet `exited|created|dead`.
2. **AC 11.2 (au premier chef)** — chaque commande du tableau : rejoue-la **avec son code de sortie**
   (`git grep … -- . ':(exclude)_bmad-output'; echo "code $?"`) et compare à la colonne « attendu » (en tenant compte
   de ce qui ne sera vrai qu'après le développement). La forme `':(exclude)…'` est-elle employée **partout** où un
   périmètre `B` est annoncé ? L'attendu de `service=kesh-api` (`recette.sh:67`) est-il juste ?
3. **Le reste du diff** : AC 7.6 (d) et 7.8 (G14 lit `up` : la commande d'inventaire `grep -nE "docker compose.* up\b"
   docs/manual/fr/admin-manual.tex` rend-elle bien 6 lignes qui nomment `kesh-api`, toutes hors sections Synology ?),
   AC 9 (`[\s~]+`), AC 8.1/8.6 (base hors Docker), AC 4 (`<title>` au tiret long), AC 2.3/2.4 ; registre C-26-3-8, 13,
   16, 18, 19, 20 ; Change Log : décomptes de la P3 (R 0H/2M/9L, F 0H/4M/8L, « 4 MEDIUM distincts ») contre les deux
   rapports, et correction en place du Change Log P2 (« 9 MEDIUM distincts »).

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée),
correction proposée. **Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée**, sur le fichier
**final** (pas sur le diff). Pour tout CRITICAL ou HIGH affirmant qu'une correction « n'a pas été appliquée », la sortie
`grep -nF` qui le prouve est **obligatoire**. ⛔ **La liste des axes exercés ET non exercés**, et pour l'axe 1 le
**tableau de tes 23 déroulés** (cas, classe obtenue) et de tes huit ensembles de mutation — un « 0 finding » sans eux ne
compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/26-3-validate-p4-ciblee.md` ; dernier message : le chemin,
le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/synology/recette.sh`, `kesh-restore.sh`, `kesh-dump.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `docker`
(y compris `docker ps`), `bash` sur un script du dépôt ou sur une fonction qui l'imite, `gh` en écriture, SQL. Les
déroulés de l'axe 1 sont **raisonnés**. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`grep`,
`gh api` en lecture.
