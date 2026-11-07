# Revue de code P1 — 33-9, le test `sidebar-navigation` et l'auto-dépliage (#424), trois lentilles B, E, A (lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **33-9** de Kesh. Contexte frais. Français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche : ce que le test et le correctif portent en eux-mêmes.
- **E — Edge Case Hunter** : chaque branche : `toggle` au montage, à l'auto-dépliage, clic rapide (deux `toggle` coalisés), repli sur une page du groupe puis navigation dans le groupe, déplier puis recharger sur une page du groupe, plusieurs onglets, stockage indisponible, rôle sans Administration.
- **A — Acceptance Auditor** : la fiche contre le code et les journaux, AC par AC, mutation par mutation, `docs/testing.md`, `CLAUDE.md`, CHANGELOG, manuel.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e337 log --oneline bdc3faa4..story/33-9-e2e-sidebar
git -C /home/gcorbaz/devel/kesh-e337 diff bdc3faa4 story/33-9-e2e-sidebar -- frontend docs CLAUDE.md CHANGELOG.md
```

⚠️ Le worktree porte une **autre** branche au moment de ta lecture : lis par la référence (`git show story/33-9-e2e-sidebar:<fichier>`), jamais le fichier sur le disque.

Fichiers : `frontend/src/routes/(app)/+layout.svelte` (`handleToggle`), `frontend/tests/e2e/sidebar-navigation.spec.ts`, `docs/testing.md`, `CLAUDE.md`, `CHANGELOG.md`. Fiche : `_bmad-output/implementation-artifacts/33-9-e2e-sidebar.md`, même référence (B la lit **après** son examen à l'aveugle). Journaux, tous dans `/home/gcorbaz/devel/kesh-gate-logs/e339/` : `e2e-rspec/`, `e2e-rsonde/`, `e2e-rsonde2/` (les sondes : `sidebar-probe.spec.ts.retiree`, `sidebar-probe2.spec.ts.retiree`), `suite-e337-r2-extrait.log`, `e2e-rrouge/`, `mut/resume.txt`, `mut/M*.log`, `gates-head.log`, `e2e-rvingt/`, `e2e-rsuite1/`, `e2e-rsuite2/`. Captures : `captures/1-accounts-groupe-deplie-pour-la-page.png`, `captures/2-accueil-groupe-replie-comme-choisi.png`.

## Axes obligatoires

1. **Le verdict (a)** : le journal de la sonde le soutient-il ? Ce que la fiche déclare ne pas expliquer l'est-il honnêtement ?
2. **Pas d'affaiblissement** : le poll peut-il passer à vide ; aucun `test.skip`, `retry`, `.fixme`, `waitForTimeout` dans la spec commitée (`git grep` à la référence) ; M1 à M5 tuées pour la raison dite.
3. **`if (open === expandedState[key]) return;`** : un choix de l'utilisateur peut-il être perdu ? `expandedState` et le DOM peuvent-ils diverger (Svelte réécrit-il `open` ?) ; l'interaction avec `autoExpandedPaths` ; un groupe replié par l'utilisateur sur une page qui en fait partie, puis rechargé.
4. **Le compte des échecs attendus** : « 7 à 9 », `docs/testing.md`, `CLAUDE.md` ; les relevés datés non réécrits ; deux tirages de suite ne prouvent pas l'absence de la course : la fiche le dit-elle ?
5. **Le manuel** — `docs/manual/fr/user-manual.tex` et `.pdf` : la fiche dit « relu, rien à changer » : contrôle-le, le PDF aplati (`git show story/33-9-e2e-sidebar:docs/manual/fr/user-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`), sur le menu, les groupes et leur mémorisation.
6. **Le CHANGELOG** contre le comportement réel, phrase par phrase.

## Contraintes

⛔ Lecture seule : aucune écriture hors de ton rapport, ni redirection ni fichier temporaire, aucune compilation ni exécution ; `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `docker` et tout `gh` qui écrit sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport : `/home/gcorbaz/devel/kesh-gate-logs/e339-review-p1-{B,E,A}.md`, selon ta lentille : **les axes réellement exercés et ceux qui ne l'ont pas été** ; chaque finding avec sévérité, fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
