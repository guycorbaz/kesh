# Prompt — validation P5, Story 32-5 (les clés MCP et les réglages du serveur, en base) — passe ciblée

*Versionné le 2026-10-25. **Passe ciblée** (`CLAUDE.md`, *La passe ciblée*) : **une seule lentille, regression hunter**,
contexte frais, **Sonnet**, braquée sur **le seul commit de la remédiation P4**. Lecture seule stricte. Trend des passes
complètes : P1 (Sonnet) R 0/2/3/7, F 0/0/5/4 ; P2 (Opus) R 0/0/3/7, F 0/1/4/10 ; P3 (Sonnet) R 0/0/0/7, F 0/0/1/9 ;
passe ciblée P4 (Opus) 0/0/1/6.*

Dépôt `/home/gcorbaz/devel/kesh-32-5`, branche `story/32-5`. **Le commit à relire** : le dernier de la branche dont le
message contient « validation P4 » — `git log -1 --format=%H --grep "validation P4"` ; son diff :
`git show <ce commit>` (parent : `bafe2c9b`). Il ne touche que des artefacts BMAD :
`_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md` (la fiche), `epic-32-choix-autonomes-325.md` (le
registre), `sprint-status.yaml`, `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (une précision datée à la puce Q4), et ce
prompt. Rapport de la P4 qu'il remédie : `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p4.md` ; l'entrée *Validation P4*
du Change Log de la fiche et le choix C-32-5-28 du registre disent ce qui a été fait.

## La lentille — regression hunter

La sévérité se déplace vers ce qu'on vient d'écrire : sur cette story, la P2 a déclaré cinq findings nés de la
remédiation P1, la P3 deux nés de la remédiation P2, et la procédure de restauration d'une sauvegarde ancienne a
recyclé trois fois (R2, F6, H1) avant d'être **retirée** (C-32-5-28). Ta question n'est pas « la story est-elle bonne ? » mais **« ce
commit a-t-il introduit un défaut, ou laissé un finding de la P4 à moitié corrigé ? »**. Axes, tous à exercer :

1. **Chaque finding de la P4 est-il réellement remédié ?** Pour H1 à H7 : le texte nouveau dit-il ce que le rapport
   et la décision de l'orchestrateur demandaient, et le dit-il **juste au code** ? Ouvre chaque site cité par le diff
   (`sed -n`, `grep -nF`) et copie la sortie — en particulier `crates/kesh-api/tests/textes_coherents.rs:1240-1285`
   (G19 : ce que la prescription de l'AC-a13 (1) permet d'écrire au manuel sans la faire rougir), `services/acces.rs:57-58`,
   `admin_pat_denied_e2e.rs:795-797`, `:830-836`, `:881-888`, `docs/migrations-idempotence-audit.md:99-100`,
   `middleware/rbac.rs:28-33`, `lib.rs:221-402` (les routes des réglages et celles des clés), le bloc de
   `sprint-status.yaml` des clés `32-5*`.
2. **Ce que le commit a pu casser** : le **retrait** de l'AC-a13 (1) laisse-t-il ailleurs une phrase qui prescrit encore
   un chemin propre à une version ancienne — AC-a13 (5), le CHANGELOG prescrit, le point 7 à Guy, C-32-5-21, l'annotation
   du fichier d'epic, *Ce que reçoivent les stories suivantes* ? Greppe la **valeur** (« assistant », « base vide »,
   « amorçage », « Restaurer / Importer », « égale ou plus récente ») sur la fiche, le registre et le fichier d'epic. Les
   chiffres de H7 (39 → 35, `(9, 26)`, `discriminating` 8 → 7) se tiennent-ils contre `admin_head_paths`
   (`admin_pat_denied_e2e.rs:107-113`) et la ventilation GET/mutantes des quatre routes des clés (AC-c1) ? Les comptes
   de modules (onze distincts, dix-huit en somme) contre le tableau du découpage et le point 4 à Guy ? Un décompte faux
   (`grep -cE '^[0-9]+\. \*\*AC-'`, `'^- \[ \] \*\*T-'`, `'^\| M[abcd][0-9]'` sur la fiche, `'^### C-32-5-'` sur le
   registre) ; une affirmation de l'entrée *Validation P4* que le diff ne tient pas — **refais les greps qu'elle
   déclare** ; un YAML cassé (lis-le, sans l'exécuter).
3. **Le manuel** : le commit n'en change pas le texte, mais il change ce que la fiche **prescrit** d'y écrire. Vérifie
   que la prescription retirée ne contredit pas le manuel actuel (§ *Restaurer sur une nouvelle machine*, l. 2371-2392,
   `:2246`) **ni le PDF aplati** (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE
   ".{80}Compatibilité.{160}"`), et qu'elle reste écrivable sans URL que G19 refuserait.
4. **Le reste n'est pas ton périmètre** : ne refais pas la validation complète ; un défaut hors du diff ne se rapporte
   que s'il rend faux une phrase du diff.

## Ce que tu rends

Findings `P5-1…` (numérotés à neuf), sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche (ou du registre), site du dépôt, preuve
(commande et sortie copiée), correction proposée. Un bilan d'en-tête cohérent avec le corps. ⛔ **La liste des axes
exercés ET non exercés** (1 à 3) — sans elle, « 0 finding » ne compte pas. Rapport écrit **d'un seul jet**, un seul
fichier : `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p5.md`. Dernier message : chemin, bilan par sévérité, une
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
