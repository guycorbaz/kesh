# Prompt — validation P8, Story 32-5 (les clés MCP et les réglages du serveur, en base) — passe ciblée

*Versionné le 2026-10-25. **Passe ciblée** (`CLAUDE.md`, *La passe ciblée*) : **une seule lentille, regression hunter**,
contexte frais, **Opus**, braquée sur **le seul commit de la remédiation P7**.

⛔ **C'est la DERNIÈRE passe permise** (`CLAUDE.md`, *Review Iteration Rule* : 8 passes au plus). Aucune remédiation
écrite après elle ne sera relue : rapporte ce que tu trouves, et dis pour chaque finding s'il se corrige sans risque
par une annotation ou un mot, ou s'il doit être **retiré** ou **renvoyé à l'étape 0 du développement**. Lecture seule stricte. Passes : **8 sur
8**. Trend : P1 (Sonnet) R 0/2/3/7, F 0/0/5/4 ; P2 (Opus) R 0/0/3/7, F 0/1/4/10 ; P3 (Sonnet) R 0/0/0/7, F 0/0/1/9 ;
P4 ciblée (Opus) 0/0/1/6 ; P5 ciblée (Sonnet) 0/0/1/4 ; P6 ciblée (Opus) 0/0/2/1 ; P7 ciblée (Sonnet) 0/0/1/5.*

Dépôt `/home/gcorbaz/devel/kesh-32-5`, branche `story/32-5`. **Le commit à relire, désigné par son SHA** :
`3a57bbce9b3b9d2429f674041672f144aa8e39fd` — `git show 3a57bbce9b3b9d2429f674041672f144aa8e39fd` (parent :
`b12a24df`). ⚠️ Ne le retrouve **pas** par `git log --grep` : la P5 a constaté qu'un tel motif peut rendre un commit de
`main`. Il ne touche que des artefacts BMAD : la fiche `_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md`,
le registre `epic-32-choix-autonomes-325.md`, `sprint-status.yaml` et `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md`
(la précision datée de la puce Q4). Rapport de la P7 qu'il remédie : `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p7.md` ;
l'entrée *Validation P7* du Change Log de la fiche et le choix C-32-5-28 disent ce qui a été fait.

## La lentille — regression hunter

La sévérité se déplace vers ce qu'on vient d'écrire : sur cette story, la phrase sur la restauration d'une sauvegarde
ancienne a recyclé **six fois** (R2 en P2, F6 en P3, H1 en P4, P5-1 en P5, P6-1 en P6, P7-1 en P7). La remédiation P7
l'a traitée **par classe** : l'AC-a13 (5) fait relire en entier deux § du manuel et ferme la liste des mentions de
format et de version par un grep, au lieu d'énumérer des lignes. Ta question n'est pas « la story est-elle bonne ? » mais **« ce commit a-t-il
introduit un défaut, ou laissé un finding de la P7 à moitié corrigé ? »**. Axes, tous à exercer :

1. **Chaque finding de la P7 est-il réellement remédié ?** P7-1 (classe : l'AC-a13 (5) fait relire *Restaurer sur une
   nouvelle machine* et *Compatibilité*, réécrire toute mention de format ou de version, et fermer la liste par le grep
   `format [12]|0\.15\.0|ou plus récente|égale` sur le `.tex` et le PDF aplati ; `:2373` premier site connu) — le grep
   prescrit est-il bien formé, rend-il **aujourd'hui** `:2373` et les autres mentions des deux §, et serait-il assez
   large pour fermer la classe (`grep -nE` sur `docs/manual/fr/admin-manual.tex`, copie la sortie) ? P7-2 (point 7),
   P7-3 (annotation de C-32-5-21 déplacée, entrée P6 annotée), P7-4 (précision du fichier d'epic annotée, non réécrite —
   compare à `git show 481116dd:_bmad-output/planning-artifacts/epic-32-serveur-mcp.md`), P7-5 (en-tête), P7-6 (reporté à
   *À recontrôler à l'étape 0 du développement* : la section existe-t-elle, et dit-elle juste ?).
2. **Ce que le commit a pu casser** : une contradiction entre l'AC-a13 (1), (5), l'AC-a14, le contrôle du PDF, le
   CHANGELOG prescrit et le point 7 ; une annotation qui réécrit l'histoire ; une affirmation de l'entrée *Validation
   P7* que le diff ne tient pas — **refais les greps qu'elle déclare** ; un décompte faux (`grep -cE '^[0-9]+\. \*\*AC-'`,
   `'^- \[ \] \*\*T-'`, `'^\| M[abcd][0-9]'` sur la fiche, `'^### C-32-5-'` sur le registre) ; un YAML cassé (lis-le,
   sans l'exécuter).
3. **Toute la fiche, texte aplati** : relis l'intégralité de la fiche et du registre (`tr '\n' ' ' < fichier | tr -s ' '
   | grep -oiE ".{80}(sauvegarde plus ancienne|version qui l.a produite|installer|0\.15\.0|ou plus récente|format [12]).{80}"`)
   à la recherche d'une **prescription vivante** restante pour une sauvegarde ancienne ailleurs qu'à l'AC-a13 (1) et à
   la classe d'AC-a13 (5). Distingue le vivant des entrées datées et des contextes historiques.
4. **Le manuel** : la prescription ne doit contredire ni le manuel actuel ni le PDF aplati (`pdftotext
   docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}format 2 \(0.15.0.{120}"`), et rester
   écrivable sans URL que G19 refuserait (`crates/kesh-api/tests/textes_coherents.rs:1240-1285`).
5. **Le reste n'est pas ton périmètre** : hors de l'axe 3, ne refais pas la validation complète ; un autre défaut hors
   du diff ne se rapporte que s'il rend faux une phrase du diff.

## Ce que tu rends

Findings `P8-1…` (numérotés à neuf), sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche (ou du registre), site du dépôt, preuve
(commande et sortie copiée), correction proposée. Un bilan d'en-tête cohérent avec le corps. ⛔ **La liste des axes
exercés ET non exercés** (1 à 5) — sans elle, « 0 finding » ne compte pas. Rapport écrit **d'un seul jet**, un seul
fichier : `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p8.md`. Dernier message : chemin, bilan par sévérité, une
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
