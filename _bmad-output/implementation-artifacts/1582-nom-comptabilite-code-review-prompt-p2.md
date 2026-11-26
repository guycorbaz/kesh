# Prompt — revue de code P2 ciblée de la story `1582-nom-comptabilite` (#1582), une lentille, sur le seul commit de remédiation de la P1

Tu relis, **en lecture seule**, la **remédiation de la revue de code P1** de la story `1582-nom-comptabilite` de Kesh (frontend seul, hors epic, visée v0.20.0).
**Passe ciblée : une seule lentille. Tu ne rouvres pas la P1.**

- **Modèle : Haiku. Contexte frais** (la P1 était Opus). Langue du rapport : français.
- **Entrée** : le diff **aplati** de ce seul commit, `/home/gcorbaz/devel/kesh-gate-logs/s1582/p2-diff.patch` (lecture : `cat`), produit par
  `git diff 12b02d2dd HEAD` sur la branche `story/1582-nom-comptabilite` (`12b02d2dd` = tête relue par la P1). Rapport de la P1 :
  `/home/gcorbaz/devel/kesh-gate-logs/s1582/code-review-p1.md` (R1-1 à R1-8). Fiche : `/home/gcorbaz/devel/kesh-1582/_bmad-output/implementation-artifacts/1582-nom-comptabilite.md`.
  Journaux : `…/dev/mutations.md`, `…/e2e/e2e-mut-r11.log`, `…/e2e/e2e-full2.log`, `…/dev/gate-front2.log`, `…/captures/mesures.txt`, `…/captures/4b-groupes-replies-*.png`.
- **Lentille** : « chaque remédiation corrige-t-elle son finding, sans défaut neuf ? » — pour R1-1, R1-2, R1-3, R1-5, R1-6, R1-7, R1-8 : le diff fait ce que la fiche dit, le test
  neuf prouve ce qu'il dit (R1-1 : le journal de la mutation montre le rouge ; R1-3 : M11a rejouée tuée), les sélecteurs désignent un seul élément, la capture 4b montre bien trois
  groupes repliés (ouvre-la), le texte du manuel (`docs/manual/fr/user-manual.tex`, lignes 150-160) ne redit plus « le nom de la comptabilité » deux fois et reste conforme au code ;
  vérifie aussi le `.pdf` aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) : il garde l'ancien texte (attendu, régénéré à la release). R1-4 : l'issue P4 écrite en fin de fiche décrit le défaut sans l'avoir corrigé.

## Garde-fou `grep -nF`

Pour tout finding CRITICAL ou HIGH qui affirme l'absence d'un code attendu ou la présence d'un anti-pattern, donne `grep -nF "<chaîne>" <fichier>` (`-F` obligatoire) et sa sortie
copiée ; sans cette preuve, le finding est écarté. Cite toujours `chemin:ligne`. Tu travailles sur le diff aplati : relis le fichier réel avant d'affirmer qu'un code manque.

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. Tu ne lances **ni** `cargo`, **ni** `npm` / `npx` (dont `vitest`, `playwright`), **ni** `make`, **ni** aucun `scripts/*`
(dont `scripts/prepare-release.sh`, `scripts/mem-guard.sh`), **ni** `docker`, `mariadb`, `sqlx`, `curl`, `wget`, **ni** `git commit`, `checkout`, `stash`, `restore`, `reset`, `merge`,
`rebase`, `fetch`, `push`, `worktree`, **ni** `gh` en écriture, **ni** `sed -i`, `perl -pi`, `tee`, `>`, `>>` ou toute redirection vers un fichier, **ni** `pkill`, `killall`, `kill` par motif,
**ni** `python3` ou autre interpréteur. Rien sous `~/kesh/`. Permis : `cat`, `grep`, `sed -n`, `ls`, `wc`, `find` (sans `-exec` ni `-delete`), `git log`, `git show`, `git diff`, `git grep`,
`pdftotext` (lecture), la lecture d'images.

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/s1582/code-review-p2.md`** (le seul fichier que tu écris).

## Rapport

Pour chaque finding : `R2-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), constat avec la ligne lue citée, conséquence, remède. En fin de rapport : décompte **CRITICAL / HIGH / MEDIUM / LOW**
recompté depuis tes findings ; **axes exercés** et **non exercés** (avec la raison) ; les commandes lancées. Un « 0 finding » sans cette liste ne compte pas comme passe.
