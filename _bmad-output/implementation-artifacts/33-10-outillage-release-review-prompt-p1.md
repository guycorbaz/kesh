# Revue de code P1 — 33-10, le gate en mode CI et la recette de montée (#925), trois lentilles B, E, A (lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **33-10** de Kesh. Contexte frais. Français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche : quoting, code de sortie perdu, `trap`, tâches de fond, conteneur oublié, écriture hors du dossier jetable.
- **E — Edge Case Hunter** : chaque frontière : option sans valeur, chemin à espace, ports, base lente à démarrer, image absente ou à tirer, run tué, phase qui échoue et ce qu'elle laisse, `N` faux, crochet qui échoue, `.env` à chaque étage, environnement déjà fermé, `mem-guard` absent.
- **A — Acceptance Auditor** : la fiche contre le code et les journaux, AC par AC, mutation par mutation.

⚠️ **Ces scripts ne doivent avoir aucun accès réel.** Kesh 0.17.4 tient les livres réels de Guy sur son NAS ; le script ne connaît ni le NAS, ni ses chemins, et n'emploie pas le MariaDB de dev. Cherche tout ce qui pourrait atteindre autre chose que son conteneur et son dossier : c'est l'axe le plus cher.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e337 log --oneline bdc3faa4..story/33-10-outillage-release
git -C /home/gcorbaz/devel/kesh-e337 diff bdc3faa4 story/33-10-outillage-release -- scripts crates docs CLAUDE.md
```

⚠️ Le worktree peut porter une **autre** branche : lis par la référence (`git show story/33-10-outillage-release:<fichier>`), jamais le fichier sur le disque.

Fichiers : `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/tests/{gate-ci,recette-montee}.test.sh`, `scripts/fixtures/recette/`, `crates/kesh-db/tests/{gate_ci_script,recette_montee_script}.rs`, `crates/kesh-api/src/config.rs` (`#[cfg(test)]`, `EnvGuard`), `docs/ci.md`, `docs/testing.md`, `docs/agents-orchestration.md`, `CLAUDE.md`. Fiche : `_bmad-output/implementation-artifacts/33-10-outillage-release.md`, même référence (B la lit **après** son examen à l'aveugle). Registre : `e33-choix-autonomes-e334.md`, C-e334-21 à 25, 30, 31, 33 à 35. Journaux, dans `/home/gcorbaz/devel/kesh-gate-logs/e3310/` : `ac2.res`, `ac2b.res`, `ac2-gate-ci.log`, `rouges.log`, `mut/resume.txt`, `mut/M*.log`, `recette-a.log`, `recette-a-journal/`, `recette-b.log`, `recette-b-journal/`, `gates-head.log`, `gate-ci.log`, `gate-test-fast.log`.

## Axes obligatoires

1. **Le périmètre fermé de `recette-montee.sh`** : la relance par `env -i` et sa comparaison, le crochet `--apres` (exécuté, sourcé : ce qu'il peut atteindre), `KESH_DOCKER` et `KESH_CURL`, qui traversent la relance et sont exécutés (un appelant peut-il s'en servir, et cela compte-t-il ?), la liste des `-e`, `--network host` de l'image et ce qu'il ouvre.
2. **`gate-ci.sh`** : il écrit sur `kesh-mariadb-dev`, en `root` : la liste blanche, l'ordre des refus, mot de passe dans l'argument de `docker exec -e`, ce qu'un `DROP` peut viser. La fidélité à `ci.yml`, et les écarts déclarés.
3. **Le nettoyage** : `trap`, `exit` dans un sous-shell, conteneurs retirés sur tous les chemins, port rendu, processus en tâche de fond attendus ; un test qui passerait à vide.
4. **L'empreinte** : ce qu'elle regarde et ce qu'elle ne regarde pas, et si `journal_sans_erreur` écarte trop (`Recovery effectué`) ; `join`, tri et locale ; la phase 3 et son délai de 60 s.
5. **Les tests shell** : hermétiques ? Dépendances au poste et aux runners (`python3`, `ss`, `jq`, ports, `/tmp`) ; ce qu'ils ne prouvent pas sur des factices ; la durée qu'ils ajoutent en CI.
6. **`EnvGuard`** : le `Drop`, l'ordre, ce que le doc-comment dit de la limite.
7. **Le manuel** — `docs/manual/fr/admin-manual.tex` et son PDF : la fiche dit qu'il ne cite pas ces outils de développement : contrôle-le (`git grep -n 'gate-ci\|recette-montee' story/33-10-outillage-release -- docs/manual`, et le PDF aplati, `git show story/33-10-outillage-release:docs/manual/fr/admin-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`). Aucun comportement visible ne change.
8. **`CLAUDE.md`, `docs/ci.md`, `docs/testing.md`** : phrases factuelles, aucune règle neuve (Q3 sans réponse) ; ce qu'ils affirment contre les journaux, et la durée mesurée.

## Contraintes

⛔ Lecture seule : aucune écriture hors de ton rapport, ni redirection ni fichier temporaire, aucune compilation ni exécution, **ni des scripts ni de leurs tests** ; `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `docker` et tout `gh` qui écrit sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport : `/home/gcorbaz/devel/kesh-gate-logs/e3310-review-p1-{B,E,A}.md`, selon ta lentille : **les axes réellement exercés et ceux qui ne l'ont pas été** ; chaque finding avec sévérité, fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
