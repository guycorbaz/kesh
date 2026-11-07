# Revue de code P1 — 33-11, le filtre de CI (#928), trois lentilles B, E, A (lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **33-11** de Kesh. Contexte frais. Français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche.
- **E — Edge Case Hunter** : chaque nom de fichier, événement, état de `changes` ; `pull_request` d'une branche externe, base déplacée, fusion de `main`, sous-module, fichier supprimé, nom à retour à la ligne, `_bmad-output` vidé, `workflow_dispatch` si on l'ajoutait.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC, mutation par mutation, textes.

⚠️ **Une erreur de ce filtre saute des tests à tort, et elle est muette** : la protection de `main` compte un job sauté pour réussi, alors que Kesh tient les livres réels de Guy. C'est l'axe qui prime.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e337 log --oneline bdc3faa4..story/33-11-filtre-ci
git -C /home/gcorbaz/devel/kesh-e337 diff bdc3faa4 story/33-11-filtre-ci -- .github scripts docs CLAUDE.md CHANGELOG.md
```

Lis par la référence (`git show story/33-11-filtre-ci:<fichier>`). Fichiers : `.github/workflows/ci.yml`, `scripts/ci-seuls-artefacts.sh`, `scripts/tests/ci-seuls-artefacts.test.sh`, `docs/ci.md`, `CLAUDE.md`, `CHANGELOG.md`. Fiche : `_bmad-output/implementation-artifacts/33-11-filtre-ci.md`, même référence (B la lit **après** son examen à l'aveugle) ; le mode opératoire : `33-11-filtre-ci-preuves-ac6.md`. Registre : `e33-choix-autonomes-e3311.md`. Journaux, dans `/home/gcorbaz/devel/kesh-gate-logs/e3311/` : `rouges.log`, `mut/resume.txt`, `mut/M*.log`, `textes.log`.

## Axes obligatoires

1. **Peut-il sauter des tests à tort ?** Tout chemin vers `code=false` et vers un job sauté : la condition, `needs`, l'ordre des étapes, `$GITHUB_OUTPUT`, une sortie écrite puis un échec, une PR qui modifie le script, ses tests ou `ci.yml`. **Les preuves GitHub de l'AC-6 n'ont pas tourné : ne les déclare pas acquises.** Comportement de `needs` avec `if` : la documentation GitHub, si tu peux la lire, et dis si tu ne l'as pas lue.
2. **Le script** : `read -d ''`, le dernier nom sans NUL, `LC_ALL=C`, `set -uo pipefail` sans `-e`, le `trap` et `conclure`, les trois points, `base`/`tête` commençant par `-`.
3. **Les tests** : hermétiques en CI (`git` du runner, identité, `init.defaultBranch`, `mktemp`), et le temps qu'ils ajoutent ; la garde de `ci.yml`, ce qu'elle manquerait (`bloc`, indentation, un job renommé, un cinquième job) ; un test qui passerait à vide.
4. **Les textes** : `docs/ci.md`, `CLAUDE.md`, le passage historique annoté et non réécrit ; `git grep` du symptôme (« aucun filtre », « Aucun filtre », `paths:`, « quatre jobs », « 4 jobs »), chaque site.
5. **Le manuel** — `docs/manual/fr/` `.tex` et PDF : la fiche n'y touche pas : confirme qu'il ne décrit pas la CI autrement (`git grep -n -E 'CI|intégration continue' story/33-11-filtre-ci -- docs/manual/fr/admin-manual.tex`, le PDF aplati par `git show story/33-11-filtre-ci:docs/manual/fr/admin-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`).
6. **Le mode opératoire de l'AC-6** : les commandes disent-elles ce que la fiche exige, chaque attendu est-il observable ?

## Contraintes

⛔ Lecture seule : aucune écriture hors de ton rapport, ni redirection ni fichier temporaire, aucune exécution ; `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `docker`, tout `gh` qui écrit, sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport : `/home/gcorbaz/devel/kesh-gate-logs/e3311-review-p1-{B,E,A}.md`, selon ta lentille : **les axes réellement exercés et ceux qui ne l'ont pas été** ; chaque finding avec sévérité, fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
