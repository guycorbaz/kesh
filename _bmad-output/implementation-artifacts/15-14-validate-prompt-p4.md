# Prompt — validation P4 des specs, Stories 15-14a et 15-14b

*Versionné le 2026-10-09 (P4). Deux lentilles (Opus), contexte frais chacune, en lecture seule, chacune sur les DEUX
sous-fiches. Rotation (décision D6 de la rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe
ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-14`, branche `story/15-14-lot-documentation-libelles` (rebasée sur `bcded0c8`, 15-13a mergée).
**Fiches** : `_bmad-output/implementation-artifacts/15-14-lot-documentation-libelles.md` (index et tri),
`15-14a-manuels-et-libelles.md`, `15-14b-exploitation-et-multi-societe.md`. Chaque sous-fiche a une section
« Ce que la validation P1 doit regarder » : couvre-la. Quatrième passe. Trend : P1 8 MEDIUM (Sonnet) → P2 9 (Opus) → P3 4 distincts (Sonnet), remédiée par `b02e9af7`
(branche rebasée sur `245b91ee` : 15-13a et 15-7b1 mergées, 15-13b pas encore). Rapports précédents :
`/home/gcorbaz/devel/kesh-gate-logs/15-14-validate-p{1,2,3}-{R,F}.md`. Le défaut récurrent est l'**inventaire incomplet** ;
`b02e9af7` l'attaque à la racine (C-15-14-25) : chaque inventaire est un `git grep -I` sur tout le dépôt suivi moins
un bloc d'exclusions `E` justifié, `LC_ALL=C.UTF-8`. **Rejoue chaque commande en copiant le bloc `E` depuis la fiche**
et recompte (144, 132, 153, 477…) ; juge les EXCLUSIONS elles-mêmes (un dossier exclu à tort cache-t-il un site ?) ;
vérifie le test neuf G13 (C-15-14-26 : rougirait-il sur les trois mutations annoncées ?), la renumérotation G13–G18,
la normalisation des macros LaTeX (F-4), le pré-script (`.tmp` puis `mv`, privilèges réduits, C-15-14-28/29).
Tu peux EXÉCUTER les commandes d'inventaire en lecture (git grep, grep, pdftotext, perl sans -i), rien d'autre.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : celles du
tableau de l'index (15-14a : #539 #547 #488 #291 #458 #449 #432 #569 #321 #323, refs #459 ; 15-14b : #575 #554
#127). Choix C-15-14-1 à 31 : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles :
`CLAUDE.md`. La 15-7b1 est mergée ; la 15-13b (PR #584) pas encore.

## Lentilles

- **R — Auditeur d'acceptation** : applique `.claude/skills/bmad-create-story/checklist.md`. Chaque phrase de chaque
  issue fermée couverte par un AC testable, chaque AC par une tâche, chaque tâche par une vérification ;
  implémentable sans deviner (fichiers, lignes, clés i18n dans les 4 locales, replis Svelte, sections LaTeX) ;
  **faits cités revérifiés au code** (`grep -nF`, numéros de ligne) ; recompte des AC, tâches et issues
  (l'index annonce 11 `bug`/`known-failure` + 2 `documentation`) ; les issues écartées le sont-elles à bon droit ?
- **F — Full-scope adversary** : pour chaque texte que la story corrige, **où ailleurs la même affirmation est-elle
  écrite** (greper la valeur, pas la formulation : manuels FR `.tex` **et PDF aplatis**
  `pdftotext docs/manual/fr/<x>.pdf - | tr '\n' ' ' | tr -s ' '`, 4 locales `messages.ftl`, replis Svelte,
  `docs/`, README, `website/`, CHANGELOG) — inventorier les sites NON couverts ; la correction proposée est-elle
  vraie **contre le code** ? Un libellé changé casse-t-il un sélecteur E2E/Vitest (`grep -rn` dans
  `frontend/src` et `frontend/tests`) ou le garde `lint-i18n-ownership` ? La coupe 15-14a / 15-14b et l'ordre
  de dépendance tiennent-ils (§ « Règle de splitting préventif ») ?

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), sous-fiche visée, `fichier:ligne` (fiche et code),
**preuve** (commande et sortie, ou code cité relu), correction proposée. Pour tout finding affirmant qu'un texte
ou un code est absent ou présent : la sortie d'un `grep -nF`. ⛔ **La liste des axes exercés ET non exercés** —
un « 0 finding » sans elle ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-14-validate-p4-<R|F>.md` ;
dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans
une base, ni qui compile ou exécute : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers
la sortie standard.
