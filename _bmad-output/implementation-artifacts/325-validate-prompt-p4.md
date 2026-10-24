# Prompt — validation P4, Story 32-5 (les clés MCP et les réglages du serveur, en base) — passe ciblée

*Versionné le 2026-10-25. **Passe ciblée** (`CLAUDE.md`, *La passe ciblée*) : **une seule lentille, regression hunter**,
contexte frais, **Opus**, braquée sur **le seul commit de la remédiation P3**. Lecture seule stricte. Trend des passes
complètes : P1 (Sonnet) R 0/2/3/7, F 0/0/5/4 ; P2 (Opus) R 0/0/3/7, F 0/1/4/10 ; P3 (Sonnet) R 0/0/0/7, F 0/0/1/9.*

Dépôt `/home/gcorbaz/devel/kesh-32-5`, branche `story/32-5`. **Le commit à relire** : le dernier de la branche dont le
message contient « validation P3 » — `git log -1 --format=%H --grep "validation P3"` ; son diff :
`git show <ce commit>` (parent : `76975957`). Il ne touche que des artefacts BMAD :
`_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md` (la fiche), `epic-32-choix-autonomes-325.md` (le
registre), `sprint-status.yaml`, et ce prompt. Rapports de la P3 qu'il remédie :
`/home/gcorbaz/devel/kesh-gate-logs/325-validate-p3-R.md` et `325-validate-p3-F.md` ; l'entrée *Validation P3* du
Change Log de la fiche dit ce qui a été fait.

## La lentille — regression hunter

La sévérité se déplace vers ce qu'on vient d'écrire : sur cette story, la P2 a déclaré cinq findings nés de la
remédiation P1, la P3 deux nés de la remédiation P2. Ta question n'est pas « la story est-elle bonne ? » mais **« ce
commit a-t-il introduit un défaut, ou laissé un finding de la P3 à moitié corrigé ? »**. Axes, tous à exercer :

1. **Chaque finding de la P3 est-il réellement remédié ?** Pour F1, R1 à R7, F2 à F10 : le texte nouveau dit-il ce que
   le rapport demandait, et le dit-il **juste au code** ? Ouvre chaque site cité par le diff (`sed -n`, `grep -nF`) et
   copie la sortie — en particulier `middleware/rbac.rs:28-33`, `lib.rs:147-148`, `:158-159`, `:390-400`,
   `services/mod.rs:93-95`, `services/acces.rs` (doc de `ContexteAcces`), `admin_pat_denied_e2e.rs:769`, `:792-815`,
   `:830-836`, `:881-888`, `migrations_upgrade_path.rs` (`grep -nE "\b(83|49)\b"`), `admin_full_import_e2e.rs:953`,
   `:1093`…`:1502`, `:3692`, `:3859`, `onboarding_audit_e2e.rs:2013-2028`, `post_restore.rs:211-224`,
   `docs/migrations-idempotence-audit.md:89-100`, `middleware/auth.rs:83`.
2. **Ce que le commit a pu casser** : une phrase neuve qui contredit une autre partie de la fiche ou du registre
   (C-32-5-7 contre C-32-5-17 et l'AC-c2 ; les comptes de modules 4 / 18 / 11 contre le tableau du découpage et le
   point 4 à Guy ; T-a1/T-a2 contre AC-a12 et P8 ; les comptes « relatifs » de l'AC-b2 contre T-b0) ; un décompte faux
   (recompte les AC, tâches, mutations, choix — `grep -cE '^[0-9]+\. \*\*AC-'`, `'^- \[ \] \*\*T-'`, `'^\| M[abcd][0-9]'`,
   `'^### C-32-5-'`) ; une affirmation de l'entrée *Validation P3* du Change Log que le diff ne tient pas (« symptômes
   grepés », « restent seulement les mentions datées ») — **refais les greps qu'elle déclare** ; une annotation du
   registre de sprint qui ne dit pas l'état de la fiche, ou un YAML cassé (lis-le, sans l'exécuter).
3. **Le manuel** : le commit n'en change pas le texte, mais il change ce que la fiche **prescrit** d'y écrire (AC-a13
   (1), versions sans l'assistant). Vérifie la prescription contre `docs/manual/fr/admin-manual.tex` (§ *Restaurer sur
   une nouvelle machine*, l. 2371-2392, et le menu *Administration → Restaurer / Importer*) **et contre le PDF aplati**
   (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}Restaurer / Importer.{80}"`).
4. **Le reste n'est pas ton périmètre** : ne refais pas la validation complète ; un défaut hors du diff ne se rapporte
   que s'il rend faux une phrase du diff.

## Ce que tu rends

Findings `H1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche (ou du registre), site du dépôt, preuve
(commande et sortie copiée), correction proposée. Un bilan d'en-tête cohérent avec le corps. ⛔ **La liste des axes
exercés ET non exercés** (1 à 3) — sans elle, « 0 finding » ne compte pas. Rapport écrit **d'un seul jet**, un seul
fichier : `/home/gcorbaz/devel/kesh-gate-logs/325-validate-p4.md`. Dernier message : chemin, bilan par sévérité, une
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
