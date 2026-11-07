# Revue de code P2 — 33-10, la remédiation de la P1 (#925), une lentille (lecture seule)

Contexte frais, français soigné. La remédiation touche les deux scripts, leurs tests et `EnvGuard` : relis-la entière, non la story.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e337 diff fb518be6 de54053b -- scripts crates docs
git -C /home/gcorbaz/devel/kesh-e337 log --oneline fb518be6..story/33-10-outillage-release
```

Bornes en SHA : `fb518be6` (fin de la P1), `de54053b` (dernier commit de code). Le worktree porte une autre branche : lis par `git show de54053b:<fichier>`. Findings de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e3310-review-p1-{B,E,A}.md`. Fiche, dernière entrée du Change Log : `git show story/33-10-outillage-release:_bmad-output/implementation-artifacts/33-10-outillage-release.md`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e3310/p2/` (`rouges-*.log`, `mut/`, `gates-head.log`, `recette-a4/`, `recette-b4/`).

## Axes obligatoires

1. **Ce qui peut encore atteindre autre chose que le conteneur et le dossier du script** : la relance (`env -0`, `HOME=/nonexistent`), `docker context inspect` et ce qu'il ne voit pas, `arreter_conteneur`, le crochet (et ce que le texte en dit), `KESH_DOCKER`/`KESH_CURL`.
2. **Les contrôles qui pourraient se taire** : tout tube restant sous `pipefail` (`git grep -n '|' de54053b -- scripts/recette-montee.sh scripts/gate-ci.sh`), `journal_sans_erreur`, `peupler` et ses `|| exit $?`, `echec` sous une redirection, `exec 3>&1` et les processus de fond qui héritent de ce descripteur.
3. **L'empreinte** : `ORDER BY 1`, `users`, `audit_log` et `AUDIT_DECALAGE` (l'hypothèse que l'import ajoute dans l'ordre, après les entrées de l'installation), ce que `docs/testing.md` dit couvrir contre le code.
4. **Le nettoyage** : `arreter_image` bornée (100 × 0,2 s), `trap` TERM et INT, ce qui reste si le client reste sourd.
5. **Les tests shell** : les cas neufs prouvent-ils ce qu'ils nomment ? Le témoin de la relance a-t-il sa sonde positive ? Le cas « client sourd » sous `timeout 90` ; le cas « journal long » à 600 Ko ; la durée.
6. **`gate-ci.sh`** : `find` des `.env` (élagage, worktree, liens), journal résolu avant le `cd`, `touch` avant la ré-exécution sous `mem-guard`.
7. **Les textes** — `docs/testing.md`, `docs/ci.md`, en-têtes des scripts ; **le manuel**, `docs/manual/fr/admin-manual.tex` et son PDF : toujours muet sur ces outils (`git grep -n 'gate-ci\|recette-montee' de54053b -- docs/manual`, `git show de54053b:docs/manual/fr/admin-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`).

## Contraintes

⛔ **La lentille écrit son rapport à ce chemin, et à lui seul : `/home/gcorbaz/devel/kesh-gate-logs/e3310-review-p2.md`.** Un rapport rendu par message ne compte pas. Lecture seule hors de ce fichier : ni redirection ni fichier temporaire (les tubes sont permis), aucune compilation ni exécution, **ni des scripts ni de leurs tests** ; `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `docker` et tout `gh` qui écrit sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport porte : **les axes réellement exercés et ceux qui ne l'ont pas été** ; chaque finding avec sévérité, fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
