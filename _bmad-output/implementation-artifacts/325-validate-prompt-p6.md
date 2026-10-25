# Prompt — validation P6, Story 32-5 (les clés MCP et les réglages du serveur, en base) — passe ciblée

*Versionné le 2026-10-25. **Passe ciblée** (`CLAUDE.md`, *La passe ciblée*) : **une seule lentille, regression hunter**,
contexte frais, **Opus**, braquée sur **le seul commit de la remédiation P5**. Lecture seule stricte. Passes : **6 sur
8**. Trend : P1 (Sonnet) R 0/2/3/7, F 0/0/5/4 ; P2 (Opus) R 0/0/3/7, F 0/1/4/10 ; P3 (Sonnet) R 0/0/0/7, F 0/0/1/9 ;
P4 ciblée (Opus) 0/0/1/6 ; P5 ciblée (Sonnet) 0/0/1/4.*

Dépôt `/home/gcorbaz/devel/kesh-32-5`, branche `story/32-5`. **Le commit à relire, désigné par son SHA** :
`b7505390979e1391fc547798327c11f69b6c227a` — `git show b7505390979e1391fc547798327c11f69b6c227a` (parent :
`00475d19`). ⚠️ Ne le retrouve **pas** par `git log --grep` : la P5 a constaté qu'un tel motif peut rendre un commit de
`main`. Il ne touche que des artefacts BMAD : la fiche `_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md`,
le registre `epic-32-choix-autonomes-325.md`, `sprint-status.yaml` et `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md`
(la précision datée de la puce Q4). Rapport de la P5 qu'il remédie : `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p5.md` ;
l'entrée *Validation P5* du Change Log de la fiche et le choix C-32-5-28 disent ce qui a été fait.

## La lentille — regression hunter

La sévérité se déplace vers ce qu'on vient d'écrire : sur cette story, la phrase sur la restauration d'une sauvegarde
ancienne a recyclé **quatre fois** (R2 en P2, F6 en P3, H1 en P4, P5-1 en P5) ; la remédiation P5 l'a **réduite** à trois
énoncés vérifiables, sans procédure. Ta question n'est pas « la story est-elle bonne ? » mais **« ce commit a-t-il
introduit un défaut, ou laissé un finding de la P5 à moitié corrigé ? »**. Axes, tous à exercer :

1. **Chaque finding de la P5 est-il réellement remédié ?** P5-1 à P5-5. Pour chacun, le texte nouveau dit-il ce que le
   rapport et la consigne de l'orchestrateur demandaient (l'AC-a13 (1), C-32-5-28 et la précision du fichier d'epic
   disent **seulement** : mêmes tables ; une sauvegarde plus ancienne se restaure avec la version qui l'a produite, dont
   le manuel se trouve dans le dépôt, au tag de cette version ; ensuite, on met à jour) ? Chaque énoncé est-il
   **vérifiable au dépôt** ? Ouvre les sites (`sed -n`, `grep -nF`, `git ls-tree`, `git show <tag>:<chemin>`) et copie la
   sortie — en particulier `.github/workflows/release.yml:161-165`, `git ls-tree v0.14.0 docs/manual/fr/`,
   `crates/kesh-api/tests/textes_coherents.rs:1240-1285` (G19), `docs/manual/fr/admin-manual.tex:2246` et `:2430`
   (#746, à ne pas corriger ici), le bloc `32-5*` de `sprint-status.yaml`.
2. **Ce que le commit a pu casser** : une phrase vivante de la fiche, du registre ou du fichier d'epic qui prescrit
   encore une procédure, une « base vide », un fichier de release ou une section d'un manuel ancien — greppe la
   **valeur** (`base vide`, `joint à la release`, `PDF de la release`, `procédure`, `assistant`, `rien de plus`, `manuel
   de cette version`) et distingue le vivant (AC, choix, points à Guy, *Ce que reçoivent…*) des entrées **datées** du
   Change Log et des contextes historiques ; une contradiction entre l'AC-a13 (1) et AC-a13 (5), le CHANGELOG prescrit ou
   le point 7 à Guy ; une affirmation de l'entrée *Validation P5* que le diff ne tient pas — **refais les greps qu'elle
   déclare** ; un décompte faux (`grep -cE '^[0-9]+\. \*\*AC-'`, `'^- \[ \] \*\*T-'`, `'^\| M[abcd][0-9]'` sur la
   fiche, `'^### C-32-5-'` sur le registre) ; un YAML cassé (lis-le, sans l'exécuter).
3. **Le manuel** : le commit n'en change pas le texte, mais ce que la fiche prescrit d'y écrire. Vérifie que la
   prescription réduite ne contredit pas le manuel actuel (`:2246`, § *Restaurer sur une nouvelle machine*) **ni le PDF
   aplati** (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}Compatibilité.{160}"`),
   et qu'elle reste écrivable sans URL que G19 refuserait.
4. **Le reste n'est pas ton périmètre** : ne refais pas la validation complète ; un défaut hors du diff ne se rapporte
   que s'il rend faux une phrase du diff.

## Ce que tu rends

Findings `P6-1…` (numérotés à neuf), sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche (ou du registre), site du dépôt, preuve
(commande et sortie copiée), correction proposée. Un bilan d'en-tête cohérent avec le corps. ⛔ **La liste des axes
exercés ET non exercés** (1 à 3) — sans elle, « 0 finding » ne compte pas. Rapport écrit **d'un seul jet**, un seul
fichier : `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p6.md`. Dernier message : chemin, bilan par sévérité, une
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
