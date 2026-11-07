# Revue de code P2 — 33-11, la remédiation de la P1 (#928), une lentille ciblée (lecture seule)

Contexte frais, français soigné. Passe ciblée : relis **la remédiation de la P1**, non la story entière. Elle touche le script du filtre (code exécuté par la CI), son test, et des textes.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e337 diff d72bc24d story/33-11-filtre-ci -- scripts .github docs CLAUDE.md _bmad-output
git -C /home/gcorbaz/devel/kesh-e337 log --oneline d72bc24d..story/33-11-filtre-ci
```

Bornes en SHA : `d72bc24d` (fin de la P1) ; `7f4b25d7`, dernier commit de code (script et test) ; la tête de `story/33-11-filtre-ci`, commit de textes qui le suit. Findings de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e3311-review-p1-{A,B,E}.md`. Fiche, dernière entrée du Change Log : `_bmad-output/implementation-artifacts/33-11-filtre-ci.md`. Mode opératoire : `33-11-filtre-ci-preuves-ac6.md`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e3311/p2/` (`rouges.log`, `mut/M10-deux-points.log`, `mut/M11-nul.log`, `test-shell.log`, `textes.log`).

## Axes obligatoires

1. **Le script peut-il encore rendre `code=false` à tort ?** `exec 3<`, la boucle `read -d ''` et son `|| [ -n "$nom" ]`, le compte `lus` contre `tr -cd '\0' | wc -c`, `set -uo pipefail` sans `-e`, ce que vaut `termines` si `tr` ou `wc` échoue, un fichier de liste qui change entre les deux lectures.
2. **Les cas neufs prouvent-ils ce qu'ils nomment ?** « main avancée après la branche : trois points » (M10 réellement tuée par ce cas, et non par un autre ?), fichier supprimé seul, nom à retour à la ligne, `git` factice sans NUL final, dernier nom sans NUL. Chaque cas a-t-il sa sonde positive ?
3. **La garde de `ci.yml`** : `contient_ligne` (`grep -qxF` sur here-string), l'ouverture lue sur la ligne du job `backend`, `id: decision` et `steps.decision.outputs.code`, le refus de `continue-on-error`. Ce qu'elle laisse encore passer.
4. **Le mode opératoire de l'AC-6** : les commandes `gh` sont-elles justes (champs `--json`, noms des jobs tels que `ci.yml` les écrit, `--log --job`) ? (b), (d), (e) calés sur l'état ; (c) sous `strict: true`, `UNKNOWN`, `BEHIND`, `BLOCKED` ; la règle « ne pas fusionner tant que le revert n'est pas poussé et vert ». Une conduite qui pourrait conclure à tort, dans un sens ou dans l'autre ?
5. **Les textes** : `docs/ci.md` (§ du filtre, tableau des timeouts, « sur chaque PR »), `CLAUDE.md:293` (passage historique rendu à sa lettre) et `:297` (conditionnel) ; tout ce qui affirme encore le comportement de GitHub comme relevé (`git grep -nE 'est relevé|sont sautés|compte pour réussi' -- docs CLAUDE.md`). **Le manuel** (`docs/manual/`) : doit-il dire quelque chose de ce filtre ? (`git grep -n 'ci-seuls-artefacts\|_bmad-output' -- docs/manual`).

## Contraintes

⛔ **La lentille écrit son rapport à ce chemin, et à lui seul : `/home/gcorbaz/devel/kesh-gate-logs/e3311-review-p2.md`.** Un rapport rendu par message ne compte pas. Lecture seule hors de ce fichier : ni redirection ni fichier temporaire (les tubes sont permis), aucune compilation ni exécution, **ni du script ni de son test** ; `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `docker` et tout `gh` qui écrit sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport porte : **les axes réellement exercés et ceux qui ne l'ont pas été** ; chaque finding avec sévérité, fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
