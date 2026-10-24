# Prompt — validation P2, Story 31-4 (la répétition à blanc d'une clôture)

*Versionné le 2026-10-24. **Passe complète, deux lentilles, R et F**, contexte frais, modèle **Sonnet** (rotation après
la P1 en Opus, `CLAUDE.md` § *Review Iteration Rule*). Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-4`, branche `story/31-4`. Fiche : `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` ;
registre : `epic-31-choix-autonomes-314.md`. Règles : `CLAUDE.md`, `docs/agents-orchestration.md` § 6. Cahier des charges
d'origine : commentaire de la clé `29-7-repetition-a-blanc` de `_bmad-output/implementation-artifacts/sprint-status.yaml`
(`grep -nF "29-7-repetition-a-blanc" … | cut -c1-6000`). Rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/314-validate-p1-R.md`
et `…-p1-F.md`. La remédiation P1 est le commit `96b8cd5c` : `git show 96b8cd5c`.

Rapports, **un seul fichier chacun, écrit d'un seul jet** : lentille R
`/home/gcorbaz/devel/kesh-gate-logs/314-validate-p2-R.md` ; lentille F `/home/gcorbaz/devel/kesh-gate-logs/314-validate-p2-F.md`.

## ⛔ Attaque d'abord la remédiation

Sur ce projet, **la sévérité se déplace vers ce qu'on vient d'écrire** (`CLAUDE.md` § *La passe ciblée*). Avant tout autre
axe, lis `git show 96b8cd5c` et cherche ce que la remédiation **a introduit** : le *principe directeur* (une option de
Compose mal nommée, un ordre d'options impossible, un `-f` qui manque à une commande prescrite, une commande que la
neutralisation rend impossible — `MARIADB_*` neutralisés alors qu'un contrôle les lit ?), les neuf contrôles (un attendu
indécidable, un contrôle qui arrête une procédure correcte, un renvoi « contrôle N » resté sur l'ancienne numérotation),
la garde G22 (un jeton absent de la procédure prescrite, un critère (b) ou (e) qui rougit sur un manuel juste, (d) qui
passe à vide), la fixture (un refus qui interdit le chemin nominal, un test au `curl` factice qui ne peut pas distinguer
les cas), les mutations M1 à M11 (chacune tuée par ce qu'elle dit ?). Puis les axes ci-dessous.

## Lentille R — la raison, les risques, la complétude

Cherche ce qui, dans la procédure que la fiche prescrit, **peut encore détruire des données de la production ou du
lecteur**, ou les faire sortir de la machine, et ce que le cahier des charges exigeait que la fiche omet. Axes, tous à
exercer :

1. **R1 jusqu'au bout, après la remédiation.** Pour chaque chemin par lequel la copie pourrait viser la base, les dossiers
   ou les conteneurs d'une autre pile — compose Synology renommé, `.env` recopié, variables du shell (`COMPOSE_*`,
   `DOCKER_*`, `KESH_*`), contexte Docker distant, dossier nommé `kesh`, compose cherché dans un parent, override
   préexistant, `down -v` ou `rm -rf` mal ciblés —, le contrôle qui l'arrête est-il placé **avant** le geste dangereux, et
   sa sortie attendue est-elle décidable ? Les **essais volontaires du refus** de l'AC9 (i) à (iii) sont-ils eux-mêmes
   sans danger (aucun `up`, aucun nom de projet qui collide) ?
2. **R2** et les montages : `.env`, shell, `--project-directory`, chemins relatifs résolus hors du dossier, override qui
   ajouterait un montage. La relecture après override (contrôle 8) les couvre-t-elle ?
3. **Cahier des charges de la 29-7** : relis le commentaire de la clé, point par point (R1, R2, F5, F L9, G19, « .env de la
   copie depuis .env.example », « aucune donnée réelle », « la marche répète ses gestes »). Un point affaibli, omis ou
   contredit ? Les ajouts (collision, épinglage, sauvegarde de sécurité, version à `/health`, dossier non synchronisé)
   introduisent-ils un risque ?
4. **Les gestes de la clôture (AC5)** : décrits tels que `main` les fait ? Compare à `docs/manual/fr/user-manual.tex`
   (`sec:preparer-cloture`, `sec:transitoires`, `sec:reouverture-exercice`, `Clôture d'un exercice`) **et** aux routes de
   `crates/kesh-api` (clôture, réouverture, contre-passation datée, `closing-checks`) — la P1 n'a pas lu les routes.
5. **Données réelles** (`docs/agents-orchestration.md` § 8) : fiche, fixture, journal, captures, dossier de la copie,
   téléchargements, sauvegarde de sécurité. Le `.keshbackup` réel est-il traité comme un secret de bout en bout, sans que
   la procédure fasse supprimer l'original ?
6. **Périmètre** (`CLAUDE.md` § *Règle de splitting préventif*) : la remédiation a ajouté une enveloppe Rust et un label
   dans une sous-section voisine. Plus de cinq modules ? Un élargissement non mécanique ?
7. **Mutations et tests** : chaque AC vérifiable a-t-il son test ou son journal ? Un test peut-il passer à vide ? Le test
   shell est-il réellement lancé au gate (enveloppe, dépendance à `jq`) ?
8. **Constats renvoyés à la T3** : la liste (a) à (i) est-elle complète au regard de ce que la fiche affirme de Compose
   et de Docker sans l'avoir exécuté ? Un comportement tenu pour acquis ailleurs dans la fiche n'y figure-t-il pas ?
9. **Manuel** : la fiche **nomme-t-elle** le manuel d'administration (`.tex` et PDF), l'emplacement, les labels, les
   renvois (prose vers le manuel utilisateur, `\ref` seulement internes) ? Les gardes G14, G15, G16 de
   `crates/kesh-api/tests/configuration_transmise.rs` rougiraient-elles sur la sous-section prescrite ? Les PDF se
   contrôlent aplatis : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`.

## Lentille F — les faits que la fiche affirme

Pour **chaque fait** du tableau « État des lieux », des « Prémisses », du *principe directeur*, des AC, du registre
(C-31-4-1 à 12) et du Change Log de la P1, vérifie-le par `grep -nF`, `sed -n` ou `git show` et **copie la sortie**.
Axes, tous à exercer :

1. **Les numéros de ligne ajoutés par la remédiation** : `docker-compose.yml` l. 5, 12, 15, 47-48, 50, 63 ; `.env.example`
   l. 52, 126, 155, 232, 233, 262, 374, 377 ; `admin-manual.tex` l. 269, 573, 886, 1598, 2013, 2018-2022 ; `user-manual.tex`
   l. 15, 685, 699, 792, 794, 823, 827, 843, 845 ; `lib.rs` l. 175-176, 251, 1153-1157, 1167 ; `test_endpoints_e2e.rs`
   l. 97-104 ; `version.rs` l. 270-275 ; `manifest.rs` l. 25 ; `configuration_transmise.rs` l. 2660-2662, 2757-2759,
   2779-2793 ; `tests/common/manuel.rs` l. 125 ; `helpers/inbox.ts` l. 27, 100, 153-160 ; `helpers/api-fixtures.ts`
   l. 105-106 ; `backup-restore-new-install.spec.ts` l. 103-105 ; `recette.sh` l. 33, 43 ; `+layout.svelte` l. 118-124.
2. **Les routes et corps** que la fixture reprend : `_test/seed` (preset `with-company-no-fy`), `inbox-import`
   (`accepted[]`), `full-export` (méthode, réponse), création et validation d'une facture datée, régularisation ; la
   fixture peut-elle réellement faire tout ce que l'AC8 dit avec ces routes ?
3. **G22** : `manuel::normaliser` et `manuel::listings` font-ils ce que l'AC11 suppose (le texte des listings est-il
   conservé tel quel par la normalisation, ou faut-il extraire les listings **avant** de normaliser) ? Le numéro G22 est-il
   libre (`grep -rnF "G22" crates docs scripts`) ?
4. **Le modèle du test shell** : `scripts/tests/prepare-release.test.sh` et `crates/kesh-db/tests/prepare_release_script.rs`
   — la forme prescrite pour `repetition_fixture_script.rs` est-elle transposable (chemin, crate) ?
5. **Cohérence interne** : AC1 à AC12, T0 à T8, neuf contrôles, M1 à M11, C-31-4-1 à 12, issues n° 1 et 2, renvois
   « contrôle N », « AC N », « étapes 2 à 5 », *La marche* contre l'AC3 : une contradiction, un renvoi faux, un nombre non
   recompté ? Recompte toi-même.
6. **Entrées datées** : la remédiation a-t-elle réécrit une entrée passée de la fiche, du registre ou de
   `sprint-status.yaml` (une entrée datée se complète par une annotation, ne se réécrit pas) ? Le registre d'epic
   `epic-31-choix-autonomes.md` est-il intact (union au merge) ?
7. **Ce que la fiche omet** : un fichier versionné qui dit comment restaurer, copier ou répéter (`docs/`, `README.md`,
   `DOCKER_START.md`, `scripts/synology/`) et qu'il faudrait tenir d'accord.

## Ce que tu rends

Findings `R1…` / `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, ligne du dépôt, preuve (commande et
sortie copiée), correction proposée. Distingue ce qui vient **de la remédiation P1** de ce qui vient de la conception
d'origine. ⛔ **La liste des axes exercés ET non exercés** — sans elle, « 0 finding » ne compte pas. Toute affirmation de
présence ou d'absence s'appuie sur une sortie `grep -nF` copiée ; un comportement de Docker ou de Compose que tu n'as pas
pu exécuter se dit *non exécuté* et se propose comme constat de la T3. Dernier message : chemin, bilan par sévérité, une
ligne par finding MEDIUM ou plus.

## Interdits

⛔ **N'écris aucun fichier hors de ton rapport**, et ton rapport par un **chemin absolu** sous
`/home/gcorbaz/devel/kesh-gate-logs/`. Le dépôt principal `/home/gcorbaz/devel/kesh` est en **lecture seule** ; ses
fichiers non versionnés (`.env`, `target/`…) n'ont pas de copie — un `.env` y a déjà été écrasé par un agent de cette
fiche, après un `cd` échoué. Donc : **aucune écriture par chemin relatif** (`-o`, `>`, `cp`, `tee`) ; chaîner par `&&`,
jamais par `;` après un `cd` ; si une commande préalable échoue ou est refusée, **arrête-toi**, ne poursuis pas
« ailleurs ».

Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
**`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, **`docker` (aucune sous-commande,
`compose config` et `context` compris)**, **`curl`**, **`mkdir`**, **`python3`, `python`, `perl`, `ruby`, `node`**,
`mariadb`/`mysql`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture, `sed -i`. `pdftotext` vers
la sortie standard seulement (aucun fichier temporaire). Aucune donnée réelle de Guy (rien de `~/kesh-test/`, jamais
ouvert). Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log|show|diff|blame`, `pdftotext … -`.
