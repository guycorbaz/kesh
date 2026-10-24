# Prompt — validation P4, Story 31-4 (la répétition à blanc d'une clôture)

*Versionné le 2026-10-24. **Passe complète, deux lentilles, R et F**, contexte frais, modèle **Sonnet** (rotation après la
P3 en Opus, `CLAUDE.md` § *Review Iteration Rule*). Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-4`, branche `story/31-4`. Fiche : `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` ;
registre : `epic-31-choix-autonomes-314.md` (C-31-4-1 à 16 ; **C-31-4-15 et C-31-4-16 font loi** : le recyclage de la P3 a
conduit à retirer le script de fixture, son test shell, l'enveloppe Rust, la garde G22 et leurs mutations). Règles :
`CLAUDE.md`, `docs/agents-orchestration.md` § 6. Cahier des charges d'origine : commentaire de la clé
`29-7-repetition-a-blanc` de `_bmad-output/implementation-artifacts/sprint-status.yaml`
(`grep -nF "29-7-repetition-a-blanc" … | cut -c1-6000`). Rapports de la P3 :
`/home/gcorbaz/devel/kesh-gate-logs/314-validate-p3-R.md` et `…-p3-F.md`.

Rapports, **un seul fichier chacun, écrit d'un seul jet** : lentille R
`/home/gcorbaz/devel/kesh-gate-logs/314-validate-p4-R.md` ; lentille F `/home/gcorbaz/devel/kesh-gate-logs/314-validate-p4-F.md`.

## ⛔ Attaque d'abord la réécriture de la P3

Sur ce projet, **la sévérité se déplace vers ce qu'on vient d'écrire** (`CLAUDE.md` § *La passe ciblée*). Avant tout autre
axe, lis `git log -3 --stat` puis le diff de la remédiation P3 (le commit `docs(31-4): validation P3 remédiée…`, repérable
par `git log --oneline -5`) et cherche ce qu'elle **a introduit** :

- **Le retrait est-il complet et propre ?** `grep -nE "fixture|G22|JETONS|enveloppe|_test|accessToken|test shell"` sur la
  fiche : tout reste doit être une entrée **datée** (Change Log), une mention « retiré P3 », ou le registre. Un renvoi
  vivant à une garde, un script ou une mutation retirés (AC, tâche, contrôle, table des mutations, *Actions demandées*,
  *Ce qui se vérifie à la station*) est un finding. Les numéros retirés sont-ils tous marqués ? Les recomptes du Change Log
  P3 (13 AC dont 2 retirés ; 9 tâches ; 9 contrôles ; 16 mutations dont 1 active ; constats (a) à (t), (e) retiré ; choix
  C-31-4-1 à 16) sont-ils exacts ? **Recompte toi-même et donne la commande.**
- **La garde du `rm -rf` (AC7)** : le bloc `if … && case … in kesh-repetition-*) true ;; *) false ;; esac && … ; then …
  else … fi` est-il **syntaxiquement écrivable** dans un listing LaTeX et valide en `bash`/`sh` (un `case` dans une chaîne
  `&&`) ? Refuse-t-il bien : variable vide, relatif inexistant, lien, nom faux, barre finale, `/var/tmp` lui-même ? Accepte-t-il
  `COPIE=/var/tmp/kesh-repetition-<date>` ? L'essai (iii) à quatre cas est-il **sans danger** (aucun `rm` hors de ses
  dossiers, restes retirés) ?
- **Contrôle 8** borné au bloc `ports:` de `kesh-api` : est-il décidable sur une sortie réelle de `docker compose config`
  (contrôle sur `host_ip`, `published`) sans retomber dans le piège de `KESH_HOST: 0.0.0.0` ?
- **Données réelles** : l'exemplaire est posé **après le contrôle 9** ; les exports de souveraineté (production et copie)
  vont dans `$COPIE/telechargements/` ; l'AC7 les nomme et le contrôle final les cherche. Un chemin par lequel un fichier
  sensible resterait dans `~/Téléchargements` ou dans un dossier synchronisé ?
- **Arrêt** défini à l'AC7 (d) : chaque « arrêt » de l'AC3 y renvoie-t-il ? La reprise (contrôle 1, nouvelle date) est-elle
  praticable sans `mkdir -p` ni `rm` improvisé ?
- **La source de la sauvegarde** (État des lieux « Instance de démonstration », T1) : `seed-demo` existe-t-il tel que cité ?
  Le critère de l'AC4 (« 0 manquante », *N* possiblement nul) est-il cohérent avec l'AC5 (les quatre gestes exigent une
  écriture de régularisation 1300/2300 — la démonstration en porte-t-elle ? la fiche dit-elle quoi faire sinon ?) ?
- **C-31-4-13** : l'AC13 borne le contrôle aux manuels, nomme les dix sites de l'application (`messages.ftl` ×4 ×2, `notify.ts`,
  `opening-balances/+page.svelte`) comme hors périmètre ; l'entrée `Corrigé` du CHANGELOG dit « les manuels ». Les dix sites
  sont-ils exacts (`grep -rnE`) ?

⛔ **Ne propose AUCUN outillage nouveau** : ni script, ni fixture, ni test shell, ni garde de texte, ni réextension d'une
garde existante. **Le livrable de cette story est la procédure exécutée**, non un outillage (C-31-4-15). Un trou se signale
comme **angle mort à écrire dans la fiche** ou comme **geste à journaliser en T3/T8** ; une proposition d'outillage ne
compte pas comme finding.

## Lentille R — la raison, les risques, la complétude

Cherche ce qui, dans la procédure que la fiche prescrit, **peut encore détruire des données de la production ou du
lecteur**, ou les faire sortir de la machine, et ce que le cahier des charges exigeait que la fiche omet. Axes, tous à
exercer :

1. **R1 jusqu'au bout.** Pour chaque chemin par lequel la copie pourrait viser la base, les dossiers ou les conteneurs
   d'une autre pile — compose Synology renommé, `.env` recopié, variables du shell, contexte Docker distant, compose
   cherché dans un parent, override préexistant, `down -v` ou `rm -rf` mal ciblés —, le contrôle qui l'arrête est-il
   **avant** le geste dangereux et sa sortie attendue est-elle décidable ? Les essais (i) à (iii) de l'AC9 sont-ils sans
   danger (aucun `up`, aucun nom de projet qui collide, aucun `rm` hors des dossiers de l'essai) ?
2. **Le dossier sous `/var/tmp`** : partagé, nettoyé par `systemd-tmpfiles` (30 jours sur la station), accessible au
   conteneur jetable de la destruction ; l'oubli d'une copie (qui reste) est-il dit ?
3. **Cahier des charges de la 29-7**, point par point (R1, R2, F5, F L9, G19, `.env` depuis `.env.example`, aucune donnée
   réelle, la marche qui répète ses gestes). Un point affaibli, omis ou contredit par le retrait ?
4. **Les gestes de la clôture (AC5)** contre `docs/manual/fr/user-manual.tex` et les routes de `crates/kesh-api`.
5. **Données réelles** (`docs/agents-orchestration.md` § 8) : fiche, journal, captures, dossier de la copie,
   téléchargements, sauvegarde de sécurité, exports de souveraineté.
6. **Périmètre** (`CLAUDE.md` § *Règle de splitting préventif*) : combien de modules la fiche touche-t-elle maintenant ?
   Un élargissement non mécanique ?
7. **Ce qui porte la justesse** une fois l'outillage retiré : journal d'exécution (AC9), constats de la T3, rejeu depuis le
   PDF (T8), relecture en revue. Chaque AC vérifiable a-t-il son journal ? Les constats (a) à (t) couvrent-ils tout ce que la
   fiche affirme de Compose, de Docker et de la station sans l'avoir exécuté ?
8. **Manuel** : la fiche **nomme-t-elle** le manuel d'administration et le manuel utilisateur (`.tex` **et PDF**),
   l'emplacement, les labels, les renvois (prose vers le manuel utilisateur, `\ref` seulement internes) ? Les gardes
   G14, G15, G16, G19 rougiraient-elles ? Les PDF se contrôlent aplatis :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` (de même `user-manual.pdf`).

## Lentille F — les faits que la fiche affirme

Pour **chaque fait** que la remédiation P3 a ajouté ou modifié — ligne « Instance de démonstration », ligne des gardes
(l. 2785, 2762 de `configuration_transmise.rs`), AC3 (contrôles 3, 7, 8), AC4, AC6, AC7 (bloc de la garde, (c), (d)), AC9
(essais), AC10 (moteur), AC12 (place de `Corrigé`), AC13 (dix sites), *La marche*, T0 à T8, constats (p) à (t), registre
(C-31-4-15, 16) et Change Log P3 — vérifie-le par `grep -nF`, `sed -n` ou `git show` et **copie la sortie**. Axes, tous à
exercer :

1. **Les numéros de ligne** restants : `user-manual.tex` l. 59, 794, 845 ; `admin-manual.tex` l. 1409-1413, 1445, 1451,
   1598, 1613, 1847, 2085 ; `routes/(app)/+layout.svelte` l. 118-130 ; `settings/+page.svelte` l. 582-590 ;
   `routes/onboarding.rs` l. 204-229 ; `lib.rs` l. 936 ; `configuration_transmise.rs` l. 2661, 2762, 2785 ; `CHANGELOG.md`
   l. 13, 40, 60, 106 ; `docs/manual/Makefile` l. 35.
2. **La démonstration** : `seed_demo` pose-t-il ce que l'AC5 exige (exercice suivant, compte 1300/2300, écriture) ? La fiche
   le suppose-t-elle à tort ?
3. **La garde du `rm -rf`** : une exécution **mentale** exacte de chaque cas de l'essai (iii) ; le code de sortie d'un `case`
   dans une chaîne `&&` (norme POSIX, § 2.9.4.3) ; `dirname`/`basename`/`realpath -e` sur les cas ; **aucune exécution** (voir
   interdits).
4. **Gardes existantes** qui relisent la sous-section : G14 (angle mort `-p`), G15 (préfixe `kesh-repetition_`), G16 (e)
   (liste fermée Hyper Backup), G19 (URL au tag `v0.14.0`) — rougiraient-elles sur la sous-section telle que la fiche la
   prescrit ? Le numéro « G22 » est-il redevenu libre et n'est-il cité nulle part comme vivant ?
5. **Cohérence interne et recomptes** : AC1 à AC13, T0 à T8, neuf contrôles, M1 à M16, constats (a) à (t), C-31-4-1 à 16,
   une issue à demander, renvois « contrôle N », « AC N », « M N », « constat (x) » : une contradiction, un renvoi faux, un
   nombre non recompté ? Recompte toi-même, commande à l'appui.
6. **Entrées datées** : la remédiation P3 a-t-elle réécrit une entrée passée de la fiche, du registre ou de
   `sprint-status.yaml` (une entrée datée se complète par une annotation) ? `git diff <avant-P3>..HEAD` sur les lignes `-`.
   Le registre d'epic `epic-31-choix-autonomes.md` est-il intact (union au merge) ?
7. **Ce que la fiche omet** : un renvoi relatif que l'insertion déplacerait, un autre « Paramètres → <écran du groupe
   Administration> » dans les `.tex`, un fichier versionné qui dit comment restaurer ou répéter (`docs/`, `README.md`,
   `DOCKER_START.md`, `scripts/synology/`).

## Ce que tu rends

Findings `R1…` / `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, ligne du dépôt, preuve (commande et
sortie copiée), correction proposée. Distingue ce qui vient **de la remédiation P3** (ou de C-31-4-15/16) de ce qui vient de
la conception antérieure. ⛔ **La liste des axes exercés ET non exercés** — sans elle, « 0 finding » ne compte pas. Toute
affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée ; un comportement de Docker, de Compose ou
de la station que tu n'as pas pu exécuter se dit *non exécuté* et se propose comme constat de la T3. Dernier message :
chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

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
