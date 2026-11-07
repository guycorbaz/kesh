# Revue de code P3 — 33-10, la remédiation de la P2 (#925), une lentille (lecture seule)

Contexte frais, français soigné. Passe ciblée : relis la remédiation de la P2 entière, non la story. Elle comprend deux auteurs : **`7810e15f`**, remédiation de l'orchestrateur (L5, `gate-ci.sh`), et **`9b39b322`**, celle de l'agent de développement (M1, L1 à L4, L6 à L8, enveloppes Rust). L'orchestrateur n'a aucun privilège de justesse : juge son commit comme l'autre.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e337 diff 93cb2317 9b39b322 -- scripts crates docs
git -C /home/gcorbaz/devel/kesh-e337 log --oneline 93cb2317..story/33-10-outillage-release
```

Bornes en SHA : `93cb2317` (fin de la remédiation de la P1), `9b39b322` (dernier commit de code). Lis les fichiers entiers par `git show 9b39b322:<fichier>`. Findings de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/e3310-review-p2.md`. Fiche, dernière entrée du Change Log (« remédiation de la revue P2 ») : `git show story/33-10-outillage-release:_bmad-output/implementation-artifacts/33-10-outillage-release.md`. Choix C-e334-36 à 38 : `_bmad-output/implementation-artifacts/e33-choix-autonomes-e334.md`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e3310/p3/` (`rouges-recette.log`, `verts-recette.log`, `mut/`, et les gates).

⚠️ **Contexte** : la station est tombée en OOM le 2026-11-07 à cause d'un cas de test de cette story (`/dev/full` passé comme journal à `gate-ci.sh`, relu sans fin par un `sed`). Toute commande du code relu qui pourrait lire un fichier spécial, un flux ou un périphérique sans fin est un axe à part entière.

## Axes obligatoires

1. **`7810e15f` (L5)** : la garde `[ -f "$JOURNAL" ]` après `touch` — que reste-t-il d'atteignable (lien, FIFO, fichier remplacé entre contrôle et `tee`, `/proc`) ? `PIPESTATUS` relu, code 5, ce qu'il advient du code de `cargo` ; le cas de test « journal verrouillé » (le `chmod` du faux `docker`) prouve-t-il ce qu'il nomme ? Le cas `/dev/full` passe-t-il par la garde qu'il croit éprouver ?
2. **M1, `arreter_image` et `attendre_fin`** : la séquence `docker stop` → 20 s → TERM → 2 s → KILL → 2 s → second `docker stop` → `wait`. Le `wait` peut-il encore attendre sans borne ? `kill -0` sur un zombie ; le conteneur créé après la mort de son client ; `nettoyer` rappelé sous un signal pendant `arreter_image`. Le faux client (`trap '' TERM`) modélise-t-il le vrai ? Le test prouve-t-il que le client ne survit pas ?
3. **L4, `--relance-faite`** : peut-il faire sauter la fermeture de l'environnement ? Un appelant qui le passe ; son retrait par `shift` ; le test (la condition remplacée par `sed` dans une copie) modélise-t-il « un environnement que la relance ne ferme pas » ?
4. **L7, `curl` borné** : toute invocation passe-t-elle par `C` (`git grep -n 'CURL' 9b39b322 -- scripts/recette-montee.sh`) ? L'ordre des options (`--max-time` répété, la dernière l'emporte) ; les bornes choisies (5, 60, 600 s) contre la taille possible d'un `.keshbackup` ; l'assertion « tous à `--max-time` » est-elle un inventaire clos ou une énumération ?
5. **L1, L2, L3, L6** : `3>&-` sur les deux lancements de fond, et les autres processus de fond éventuels (crochet) ; `audit_log` compté par `LIMIT` au lieu d'une fenêtre de numéros, contre `crates/kesh-db/src/backup.rs` (ordre de réinsertion) ; `"$DOCKER"` aux outils ; le motif `base2` d'`awk`, vide sans `--import`.
6. **Les enveloppes Rust** (`ulimit -v 4000000 && exec timeout -k 10 600 bash "$0"`) : bornes suffisantes pour la durée mesurée du test, comportement en CI, ce qu'un dépassement rend comme message.
7. **Les textes** — en-tête des scripts, `docs/testing.md`, `docs/ci.md` (durées des tests, codes), la fiche (conception, Change Log) ; **le manuel**, `docs/manual/fr/admin-manual.tex` et son PDF : toujours muet sur ces outils (`git grep -n 'gate-ci\|recette-montee' 9b39b322 -- docs/manual`, `git show 9b39b322:docs/manual/fr/admin-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`).

## Contraintes

⛔ **La lentille écrit son rapport à ce chemin, et à lui seul : `/home/gcorbaz/devel/kesh-gate-logs/e3310-review-p3.md`.** Un rapport rendu par message ne compte pas. Lecture seule hors de ce fichier : ni redirection ni fichier temporaire (les tubes sont permis), aucune compilation.

⛔ **Aucune exécution d'aucun test, ni d'aucun script** : ni `scripts/tests/*.test.sh`, ni `cargo test`/`nextest`, ni `scripts/gate-ci.sh`, ni `scripts/recette-montee.sh`, ni `bash -c` d'un extrait. Le comportement d'un outil (`docker`, `bash`, `curl`, `kill`) se tient de la lecture et se déclare non exécuté. `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `docker` et tout `gh` qui écrit sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport porte : **les axes réellement exercés et ceux qui ne l'ont pas été** ; chaque finding avec sévérité, fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
