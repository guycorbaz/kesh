# Prompt — validation P7, Story 32-5 (les clés MCP et les réglages du serveur, en base) — passe ciblée

*Versionné le 2026-10-25. **Passe ciblée** (`CLAUDE.md`, *La passe ciblée*) : **une seule lentille, regression hunter**,
contexte frais, **Sonnet**, braquée sur **le seul commit de la remédiation P6** — et, pour un axe, sur **toute la fiche**. Lecture seule stricte. Passes : **7 sur
8**. Trend : P1 (Sonnet) R 0/2/3/7, F 0/0/5/4 ; P2 (Opus) R 0/0/3/7, F 0/1/4/10 ; P3 (Sonnet) R 0/0/0/7, F 0/0/1/9 ;
P4 ciblée (Opus) 0/0/1/6 ; P5 ciblée (Sonnet) 0/0/1/4 ; P6 ciblée (Opus) 0/0/2/1.*

Dépôt `/home/gcorbaz/devel/kesh-32-5`, branche `story/32-5`. **Le commit à relire, désigné par son SHA** :
`2e0ea04c70aa82d7e98db9ccaaa9df30d365d2a7` — `git show 2e0ea04c70aa82d7e98db9ccaaa9df30d365d2a7` (parent :
`481116dd`). ⚠️ Ne le retrouve **pas** par `git log --grep` : la P5 a constaté qu'un tel motif peut rendre un commit de
`main`. Il ne touche que des artefacts BMAD : la fiche `_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md`,
le registre `epic-32-choix-autonomes-325.md`, `sprint-status.yaml` et `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md`
(la précision datée de la puce Q4). Rapport de la P6 qu'il remédie : `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p6.md` ;
l'entrée *Validation P6* du Change Log de la fiche et le choix C-32-5-28 disent ce qui a été fait.

## La lentille — regression hunter

La sévérité se déplace vers ce qu'on vient d'écrire : sur cette story, la phrase sur la restauration d'une sauvegarde
ancienne a recyclé **cinq fois** (R2 en P2, F6 en P3, H1 en P4, P5-1 en P5, P6-1 en P6) ; la remédiation P6 a retiré la
dernière prescription connue (AC-a13 (5)) et décidé que **seule l'AC-a13 (1)** porte la phrase sur une sauvegarde
ancienne ; elle a aussi rétabli, comme phrase distincte, le rappel « prendre une sauvegarde complète juste après la
mise à jour ». Ta question n'est pas « la story est-elle bonne ? » mais **« ce commit a-t-il
introduit un défaut, ou laissé un finding de la P6 à moitié corrigé ? »**. Axes, tous à exercer :

1. **Chaque finding de la P6 est-il réellement remédié ?** P6-1 (retrait : l'étape 1 de *Restaurer sur une nouvelle
   machine*, `admin-manual.tex:2376`, ne nomme plus aucune version à installer, se réduit à la règle des mêmes tables et
   renvoie à *Compatibilité*), P6-2 (rétablissement : « prendre une sauvegarde complète juste après la mise à jour »,
   phrase distincte du manuel et du CHANGELOG, contrôlée au PDF), P6-3 (annotations datées de « base vide »). Le texte
   nouveau est-il vérifiable au dépôt ? Ouvre les sites et copie la sortie — `docs/manual/fr/admin-manual.tex:2246`,
   `:2371-2392`, `:2345` (fréquence par défaut de la sauvegarde automatique), le bloc `32-5*` de `sprint-status.yaml`, la
   puce Q4 du fichier d'epic et ses précisions datées.
2. **Ce que le commit a pu casser** : une contradiction entre l'AC-a13 (1), (5), le CHANGELOG prescrit, le point 7 à
   Guy, C-32-5-21 et C-32-5-28 ; une annotation datée qui réécrit l'histoire au lieu de l'annoter ; une affirmation de
   l'entrée *Validation P6* que le diff ne tient pas — **refais les greps qu'elle déclare** ; un décompte faux
   (`grep -cE '^[0-9]+\. \*\*AC-'`, `'^- \[ \] \*\*T-'`, `'^\| M[abcd][0-9]'` sur la fiche, `'^### C-32-5-'` sur le
   registre) ; un YAML cassé (lis-le, sans l'exécuter).
3. **Toute la fiche, hors du diff — axe exigé** : relis **l'intégralité** de
   `_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md` (AC, tâches, *Ce que les gardes ne voient pas*,
   Dev Notes, *Ce que reçoivent les stories suivantes*, *Points à soumettre à Guy*) **et** le registre
   `epic-32-choix-autonomes-325.md`, à la recherche de toute **prescription vivante restante** d'une procédure, d'une
   version à installer ou d'un chemin pour restaurer une sauvegarde ancienne, ailleurs qu'à l'AC-a13 (1). Ne te fie pas
   à un `grep` mono-ligne : les phrases sont coupées par le retour à la ligne (aplatis : `tr '\n' ' ' < fichier | tr -s
   ' ' | grep -oiE ".{80}(sauvegarde plus ancienne|version qui l.a produite|installer|mettre à jour|met à jour).{80}"`).
   Distingue le vivant des entrées **datées** du Change Log et des contextes historiques des choix ; une mention datée
   non annotée et trompeuse est un LOW.
4. **Le manuel** : la fiche prescrit ce qu'il dira ; vérifie que la prescription ne contredit pas le manuel actuel ni le
   PDF aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE
   ".{80}Compatibilité.{160}"`), et qu'elle reste écrivable sans URL que G19 refuserait
   (`crates/kesh-api/tests/textes_coherents.rs:1240-1285`).
5. **Le reste n'est pas ton périmètre** : hors de l'axe 3, ne refais pas la validation complète ; un autre défaut hors
   du diff ne se rapporte que s'il rend faux une phrase du diff.

## Ce que tu rends

Findings `P7-1…` (numérotés à neuf), sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche (ou du registre), site du dépôt, preuve
(commande et sortie copiée), correction proposée. Un bilan d'en-tête cohérent avec le corps. ⛔ **La liste des axes
exercés ET non exercés** (1 à 5) — sans elle, « 0 finding » ne compte pas. Rapport écrit **d'un seul jet**, un seul
fichier : `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p7.md`. Dernier message : chemin, bilan par sévérité, une
ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`). Aucun `git checkout`/`switch` ; aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`** et **`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`,
`rustup`, `npm`, `npx`, `docker`, `curl`, `python3`, `python`, `perl`, `ruby`, `node`, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et **toute commande `gh`**. Aucune donnée réelle de Guy (rien
de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre. Autorisés : lecture,
`grep`, `sed -n`, `git log|show|diff|blame`, `pdftotext … -`, et les filtres `tr`, `sort`, `cut`, `head`, `awk`, `wc`,
`uniq` vers la sortie standard.
