# Prompt — validation P3, Story 31-4 (la répétition à blanc d'une clôture)

*Versionné le 2026-10-24. **Passe complète, deux lentilles, R et F**, contexte frais, modèle **Opus** (rotation après
la P2 en Sonnet, `CLAUDE.md` § *Review Iteration Rule*). Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-4`, branche `story/31-4`. Fiche : `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` ;
registre : `epic-31-choix-autonomes-314.md` (C-31-4-1 à 14). Règles : `CLAUDE.md`, `docs/agents-orchestration.md` § 6.
Cahier des charges d'origine : commentaire de la clé `29-7-repetition-a-blanc` de
`_bmad-output/implementation-artifacts/sprint-status.yaml` (`grep -nF "29-7-repetition-a-blanc" … | cut -c1-6000`).
Rapports de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/314-validate-p2-R.md` et `…-p2-F.md`. Choix de l'orchestrateur
postérieurs à la P2 : `git show 1984631e 07f9fdd8` (C-31-4-13 et C-31-4-14, au registre). **La remédiation P2 est le
commit `6a37e3c4`** : `git show 6a37e3c4`.

Rapports, **un seul fichier chacun, écrit d'un seul jet** : lentille R
`/home/gcorbaz/devel/kesh-gate-logs/314-validate-p3-R.md` ; lentille F `/home/gcorbaz/devel/kesh-gate-logs/314-validate-p3-F.md`.

## ⛔ Attaque d'abord la remédiation P2

Sur ce projet, **la sévérité se déplace vers ce qu'on vient d'écrire** (`CLAUDE.md` § *La passe ciblée*). Avant tout autre
axe, lis `git show 6a37e3c4` et cherche ce que la remédiation **a introduit** :

- **Le contrôle 3 et la garde du `rm -rf`** (AC3, AC7) : `mkdir -m 0700` sans `-p`, `test ! -L`, `realpath -e`,
  `dirname` égal à `/var/tmp`, nom au préfixe `kesh-repetition-`, vacuité. Un test qui refuse une procédure correcte ? Un
  cas qui passe encore (barre finale, espace dans le chemin, `/var/tmp` sur un autre système de fichiers que le Docker de
  la machine, conteneur jetable qui ne voit pas `/var/tmp`) ? L'ordre « vacuité avant le premier fichier » est-il tenu
  par *La marche* ? La forme `if … then … else … fi` sans `exit` ni `return` est-elle écrivable telle quelle ?
- **G22 réduite** (AC11) : `manuel::sans_commentaires` puis `manuel::section` sur ce texte — les décalages restent-ils
  justes ? Les jetons de `JETONS_G22` figurent-ils tous dans une procédure conforme à la fiche (un jeton qui ne pourrait
  pas apparaître, comme l'était `docker compose config`) ? Le motif d'absence `(?m)^[^#\n]*\bKESH_[A-Z_]*HOST_DIR\s*=`
  rougit-il sur un manuel juste (une phrase de prose ? une commande `grep` du contrôle 6, qui *cite* ce motif dans un
  listing ?) ou laisse-t-il passer une affectation ? Le test de retrait en mémoire peut-il passer à vide ?
- **AC13 et C-31-4-13** : les quatre lignes, le grep de contrôle (un faux zéro ? une ligne légitime touchée ?), les deux
  PDF ; la précision de fait sur la carte *Exercices comptables* de la page *Paramètres* est-elle exacte, et change-t-elle
  la justesse de la correction ?
- **La fixture** (AC8) : login du preset (`admin`/`admin123`), `accessToken`, `Authorization: Bearer`, contact, compte
  bancaire principal, `GET /api/v1/invoices/<id>/pdf`, `X-Kesh-Backup-Files: 2`. Chaque route et chaque corps existent-ils
  tels qu'écrits ? Un refus ajouté interdit-il le chemin nominal ? Le cas « login en 401 » est-il réalisable au `curl`
  factice ?
- **Contrôles 1, 8 et 9** : `docker compose version --short` (sortie comparable à « 2.24.0 » ?), une seule publication,
  `restart: "no"`, filtres non vides.
- **Mutations** M1, M3 à M7, M10 à M16 : chacune tuée par ce qu'elle dit ? M2, M8, M9 sont-elles retirées partout (T5,
  renvois) ?
- **F4** : le renvoi de `admin-manual.tex` l. 1847 réécrit en `\S\ref{sec:test-restauration}` ; G16 de
  `configuration_transmise.rs` rougirait-elle ?

⛔ **Ne propose pas de réétendre G22** (ordre, seuils, lecture de chaque commande, nouveaux critères) : C-31-4-14 l'a
réduite à des présences, sur la convergence de la P2 ; ce qu'elle ne prouve pas est porté par le journal d'exécution et
la T8, et la fiche le dit. Un trou de G22 se signale **comme angle mort à écrire dans la fiche**, non comme garde à
ajouter. Une proposition d'extension ne compte pas comme finding.

## Lentille R — la raison, les risques, la complétude

Cherche ce qui, dans la procédure que la fiche prescrit, **peut encore détruire des données de la production ou du
lecteur**, ou les faire sortir de la machine, et ce que le cahier des charges exigeait que la fiche omet. Axes, tous à
exercer :

1. **R1 jusqu'au bout.** Pour chaque chemin par lequel la copie pourrait viser la base, les dossiers ou les conteneurs
   d'une autre pile — compose Synology renommé, `.env` recopié, variables du shell, contexte Docker distant, compose
   cherché dans un parent, override préexistant, `down -v` ou `rm -rf` mal ciblés —, le contrôle qui l'arrête est-il
   **avant** le geste dangereux, et sa sortie attendue est-elle décidable ? Les essais volontaires du refus de l'AC9 (i)
   à (iii) sont-ils sans danger (aucun `up`, aucun nom de projet qui collide, aucun `rm` hors des dossiers de l'essai) ?
2. **Le dossier sous `/var/tmp`** : partagé entre utilisateurs, nettoyé par `systemd-tmpfiles` (âge ?) — une copie
   peut-elle disparaître en cours de répétition, ou être lue par un autre utilisateur ? Les dossiers créés par Docker en
   root dans un dossier `0700` : accessibles au conteneur jetable de la destruction ?
3. **Cahier des charges de la 29-7** : relis le commentaire de la clé, point par point (R1, R2, F5, F L9, G19, « .env de la
   copie depuis .env.example », « aucune donnée réelle », « la marche répète ses gestes »). Un point affaibli, omis ou
   contredit par la remédiation P2 ?
4. **Les gestes de la clôture (AC5)** contre `docs/manual/fr/user-manual.tex` et les routes de `crates/kesh-api`.
5. **Données réelles** (`docs/agents-orchestration.md` § 8) : fiche, fixture, journal, captures, dossier de la copie,
   téléchargements, sauvegarde de sécurité, export de souveraineté de l'étape 0.
6. **Périmètre** (`CLAUDE.md` § *Règle de splitting préventif*) : la P2 fait entrer le manuel utilisateur, un renvoi
   d'une section Synology, une rubrique `Corrigé` au CHANGELOG. Plus de cinq modules ? Un élargissement non mécanique ?
7. **Mutations et tests** : chaque AC vérifiable a-t-il son test ou son journal ? Un test peut-il passer à vide ? Le test
   shell est-il réellement lancé au gate, et sa garde contre le test muet est-elle écrivable sur le modèle ?
8. **Constats renvoyés à la T3**, (a) à (o) : complets au regard de ce que la fiche affirme de Compose, de Docker et de
   la station sans l'avoir exécuté ?
9. **Manuel** : la fiche **nomme-t-elle** le manuel d'administration et le manuel utilisateur (`.tex` et PDF),
   l'emplacement, les labels, les renvois (prose vers le manuel utilisateur, `\ref` seulement internes) ? Les gardes
   G14, G15, G16 rougiraient-elles ? Les PDF se contrôlent aplatis :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` (de même `user-manual.pdf`).

## Lentille F — les faits que la fiche affirme

Pour **chaque fait** que la remédiation P2 a ajouté ou modifié — État des lieux (lignes « Authentification de l'API »,
« Gel du PDF », renvoi relatif, chemin des exercices), *principe directeur*, AC1, AC3, AC7, AC8, AC9, AC11, AC12, AC13,
*La marche*, mutations, T1, T3 à T5, registre (annotations de C-31-4-6, 7, 8, 11, 13, 14) et Change Log P2 —,
vérifie-le par `grep -nF`, `sed -n` ou `git show` et **copie la sortie**. Axes, tous à exercer :

1. **Les numéros de ligne** : `user-manual.tex` l. 59, 794, 845 ; `admin-manual.tex` l. 1445, 1451, 1598, 1613, 1847,
   2085 ; `routes/(app)/+layout.svelte` l. 118-130 ; `routes/(app)/settings/+page.svelte` l. 582-590 ; `lib.rs`
   l. 249-251, 863, 1118, 1153-1157 ; `routes/auth.rs` l. 107-110, 133-142 ; `routes/admin.rs` l. 39-40, 97 ;
   `routes/issued_invoice_pdf.rs` l. 1-14 ; `routes/invoice_pdf_service.rs` (`find_primary`) ;
   `fiscal-year-closing-journey.spec.ts` l. 181 ; `helpers/api-fixtures.ts` l. 39 ; `tests/common/manuel.rs` l. 34-61,
   84, 125, 145 ; `CHANGELOG.md` l. 13, 40, 60, 106 ; `kesh-db/src/test_fixtures.rs` (doc du preset `with-company-no-fy`).
2. **Les routes et corps** de la fixture : login (corps, réponse camelCase), contacts, compte bancaire `isPrimary`,
   facture datée, validation, `GET …/pdf` (code, corps), `full-export` (en-têtes), `inbox-import` (`accepted[]`),
   `_test/seed` sans jeton. La fixture peut-elle réellement faire tout ce que l'AC8 dit ?
3. **G22** : `sans_commentaires` conserve-t-il les sauts de ligne et le contenu des `lstlisting` ? `section` sur ce texte
   trouve-t-il le label ? `listings` rend-il des lignes ? Le numéro G22 est-il toujours libre (`grep -rnF "G22" crates
   docs scripts`) ?
4. **Le modèle du test shell** : `scripts/tests/prepare-release.test.sh` et `crates/kesh-db/tests/prepare_release_script.rs`
   — la garde contre le test muet que la fiche dit reprendre existe-t-elle sous cette forme ?
5. **Cohérence interne et recomptes** : AC1 à AC13, T0 à T8, neuf contrôles, M1 à M16 (dont trois retirées), constats
   (a) à (o), C-31-4-1 à 14, une issue à demander, renvois « contrôle N », « AC N », « M N », « G22 (x) » : une
   contradiction, un renvoi faux (une lettre de G22 de l'ancienne version restée hors des entrées datées), un nombre non
   recompté ? Recompte toi-même et donne la commande.
6. **Entrées datées** : la remédiation a-t-elle réécrit une entrée passée de la fiche, du registre ou de
   `sprint-status.yaml` (une entrée datée se complète par une annotation, ne se réécrit pas) ? Le registre d'epic
   `epic-31-choix-autonomes.md` est-il intact (union au merge) ?
7. **Ce que la fiche omet** : un autre renvoi relatif que l'insertion déplacerait, un autre chemin
   « Paramètres → <écran du groupe Administration> » qui ne serait pas une carte de la page *Paramètres*, un fichier
   versionné qui dit comment restaurer ou répéter (`docs/`, `README.md`, `DOCKER_START.md`, `scripts/synology/`).

## Ce que tu rends

Findings `R1…` / `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, ligne du dépôt, preuve (commande et
sortie copiée), correction proposée. Distingue ce qui vient **de la remédiation P2** (ou des choix C-31-4-13/14) de ce
qui vient de la conception antérieure. ⛔ **La liste des axes exercés ET non exercés** — sans elle, « 0 finding » ne
compte pas. Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée ; un comportement de
Docker, de Compose ou de la station que tu n'as pas pu exécuter se dit *non exécuté* et se propose comme constat de la
T3. Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ **N'écris aucun fichier hors de ton rapport**, et ton rapport par un **chemin absolu** sous
`/home/gcorbaz/devel/kesh-gate-logs/`. Le dépôt principal `/home/gcorbaz/devel/kesh` est en **lecture seule** ; ses
fichiers non versionnés (`.env`, `target/`…) n'ont pas de copie — un `.env` y a déjà été écrasé par un agent, après un
`cd` échoué. Donc : **aucune écriture par chemin relatif** (`-o`, `>`, `cp`, `tee`) ; chaîner par `&&`, jamais par `;`
après un `cd` ; si une commande préalable échoue ou est refusée, **arrête-toi**, ne poursuis pas « ailleurs ».

Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
**`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, **`docker` (aucune sous-commande,
`compose config`, `compose version` et `context` compris)**, **`curl`**, **`mkdir`**, `realpath` et `findmnt` sur un
chemin à créer, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture, `sed -i`. `pdftotext` vers la sortie
standard seulement (aucun fichier temporaire). Aucune donnée réelle de Guy (rien de `~/kesh-test/`, jamais ouvert).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log|show|diff|blame`, `pdftotext … -`.
