# Prompt — revue de code P2 **ciblée**, Story 28-1 (les boutons d'action destructive se voient, #640)

*Versionné le 2026-10-13. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **la seule
remédiation de la revue de code P1** (`CLAUDE.md`, § *La passe ciblée* ; D6, rétrospective de l'Epic 25 : Haiku
réservé aux passes ciblées). Trend : P1 (Sonnet, lentilles B, E, A) **0 CRITICAL, 0 HIGH, 2 MEDIUM, 10 LOW**
(E1, E2 ; B1-B3, E3-E6, A-L1 à A-L3, dont B1 = A-L1, B3 = E5, E3 = A-L3).*

Dépôt `/home/gcorbaz/devel/kesh-28-1`, branche `story/28-1`. **Périmètre : la remédiation P1, lue en DIFF UNIQUE
aplati** — ne lis pas la séquence des commits (règle Haiku du `CLAUDE.md` : les numéros de ligne d'un hunk de
commit intermédiaire ne sont pas ceux du fichier final) :

```sh
git diff 5ded5663 HEAD -- frontend CHANGELOG.md          # tests, lockfile, CHANGELOG (un seul commit de code : 99682720)
git diff 5ded5663 HEAD -- _bmad-output/implementation-artifacts/28-1-boutons-destructifs.md \
                          _bmad-output/implementation-artifacts/epic-28-choix-autonomes.md
```

Rapports P1 que ce diff remédie (à lire) : `/home/gcorbaz/devel/kesh-gate-logs/28-1-review-p1-B.md`,
`28-1-review-p1-E.md`, `28-1-review-p1-A.md`. Journal des mutations de la remédiation :
`/home/gcorbaz/devel/kesh-gate-logs/28-1-mutations-p1.log` (P1-M1 à P1-M11).

Fichier de production touché : **aucun**. Vérifie-le d'abord :
`git diff --stat 5ded5663 HEAD -- frontend/src ':!*.test.ts' ':!*.spec.ts'` doit ne rendre que rien.
`frontend/package-lock.json` change d'une ligne (`"license": "MIT"` rétablie, C-28-1-42).

## La lentille — chasseur de régressions de la remédiation P1

Pour **chaque hunk**, vérifie contre le fichier final :

1. **E2E (b), E1 (C-28-1-44)** — `frontend/tests/e2e/destructive-buttons.spec.ts` : l'attente
   `document.getAnimations()` peut-elle **ne jamais finir** (animation infinie mal exclue, `effect` nul, transition
   CSS `transition-all` du bouton comptée comme animation) ou **finir trop tôt** (animations pas encore créées au
   moment du premier sondage) ? `evalueDans` résout `String(n.target[0])` par `document.querySelector` : que se
   passe-t-il si `target[0]` est un tableau (shadow DOM), ou si le sélecteur d'axe désigne un autre élément de même
   forme ? L'assertion sur `incomplete` peut-elle être vraie par construction (liste vide parce que la règle n'a
   rien évalué) — et l'assertion sur `passes` l'empêche-t-elle ? `await` présent devant chaque `evalueDans` ?
2. **AC7 (1) et (1-bis), E2** — `button-destructive.test.ts` : le cinquième mot entier
   `focus-visible:ring-destructive/40` est-il échappé correctement par `motEntier` (le `/`) ? L'absence de
   `focus-visible:ring-ring/50` dans la fusion est-elle exacte au regard de la base de `button.svelte` (relis la
   ligne `base:`) ? P1-M1 rougit-elle dans le journal, et sur quels tests ?
3. **`@theme` sans commentaires, E6** : la suppression `/\/\*[\s\S]*?\*\//g` est-elle faite **avant** la recherche
   de `@theme` et de `\n}` — un commentaire contenant `@theme`, `{` ou `\n}` déplacerait-il encore les bornes ? Un
   `/*` dans une chaîne CSS (`url("/*")`) est-il un risque réel dans `app.css` (`grep -nF '/*' frontend/src/app.css`) ?
   L'auto-test synthétique prouve-t-il ce qu'il nomme ? `declare()` remplace-t-il fidèlement l'ancienne
   expression de « tout jeton employé est défini » ?
4. **Garde (f), B2 et E3 (C-28-1-41)** — `boutons-rouges-inventory.test.ts` : `boutonsDestructifs` relève-t-il
   les mêmes éléments que l'ancienne boucle de `destructifsAvecClasse` (`el.type === 'Component'`, valeur exacte
   `destructive`) ? La directive `ClassDirective` est-elle refusée **quel que soit son nom** ? Le montage
   `parAnalyseur === parTexte` peut-il rougir à tort (un `variant="destructive"` écrit sur un autre composant que
   `Button` — `Alert`, `Badge` —, ou dans un commentaire) : cherche-le par
   `grep -rn 'variant="destructive"' frontend/src --include=*.svelte | grep -v '\.test\.'`. Le motif du choix
   `>= 25` plutôt que l'égalité est-il tenable ?
5. **Relevé AC7 (4), E4 (C-28-1-43)** — `jetons-non-definis.test.ts` : l'ordre des alternatives de `MOTIF`
   (composés avant racines) donne-t-il le bon nom pour `ring-offset-x`, `inset-ring-x`, `inset-shadow-x`, et
   **`to-`** ne capture-t-il pas un mot de prose ou un identifiant du dépôt (`to-do`, `to-be`…) qui ferait rougir
   à tort au premier ajout ? `nomDe` reste-t-il juste pour les anciens préfixes (`bg-error-soft` → `error-soft`) ?
   Le `!` final retiré avant l'opacité : `bg-foo/40!` et `bg-foo!/40` — lequel est la forme v4, et lequel est
   relevé ?
6. **CHANGELOG, B1 = A-L1** : la phrase neuve est-elle **vraie** au code (`grep -rnF 'text-destructive'` sur les
   trois sites de la classe E : `supplier-invoices/+page.svelte`, `supplier-invoices/import/+page.svelte`,
   `JournalEntryForm.svelte`) ? Promet-elle encore au-delà ?
7. **Fiche et registre (A-L2)** : recompte, ne relis pas — `grep -c '^### C-28-1-'
   _bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` contre « C-28-1-1 à C-28-1-44 » du `## Status` ;
   lignes du journal `28-1-mutations-p1.log` contre le tableau P1-M1…P1-M11 du Dev Agent Record ; décomptes de
   tests Vitest déclarés contre `28-1-p1rem-gate-front-test:unit.log` et ceux de l'E2E contre
   `28-1-p1rem-gate-e2e.clean.log` (ou le journal que le Dev Agent Record nomme).
8. **Manuel contre code (obligatoire), `.tex` ET PDF aplati** : la remédiation ne change aucun texte visible de
   l'utilisateur — vérifie-le (`git diff 5ded5663 HEAD --stat -- docs`), et qu'aucune phrase des manuels FR ne
   décrive la croix de retrait d'une ligne de saisie (`grep -n 'croix' docs/manual/fr/*.tex` ;
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` pour « croix », « rouge »).
9. **Propagation** : greper la **valeur** — `\b24\b` dans les trois fichiers de test (seuil de montage) ; `gris
   foncé au rouge` sur le dépôt (attendus : `CHANGELOG.md` et la citation d'origine de l'AC12 dans la fiche,
   suivie de son amendement) ; `C-28-1-36` au `## Status` ; `asserté par rien` (angle mort 10, annoté).

⚠️ **Entrées passées** : les entrées antérieures du Change Log et du registre gardent leurs chiffres d'alors
(consigne 15) — « 24 usages », « au moins 24 » de C-28-1-38, les compteurs de la validation : ce n'est pas un résidu.
⚠️ **Grep ground-truth obligatoire** : toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée, sur
le **fichier final** (`git show HEAD:<chemin>` ou le fichier du worktree). Un CRITICAL ou HIGH qui affirme qu'un patch
« n'a pas été appliqué » se vérifie ainsi avant d'être écrit.
⚠️ **Erreur de catégorie** : un livrable que la fiche renvoie à #645, #646 ou #648 (angles morts 1, 2, 11) n'est pas
un défaut de cette remédiation.

## Ce que tu rends

Findings numérotés `P2-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), origine (remédiation P1 / rédaction antérieure), correction proposée. ⛔ **La liste des axes exercés
ET non exercés** (les neuf ci-dessus) — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-1-review-p2-ciblee.md` ; dernier message : le chemin du rapport, le bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`,
`cargo`, `npm`, `npx`, `node`, `vitest`, `playwright`, `docker`, `bash` ou `sh` sur un script, **tout interpréteur**
(`python`, `python3`, `perl`, `ruby`, `deno`, `awk` avec un programme qui écrit), `gh` en écriture, toute requête SQL.
Rien de `~/kesh-test/` (relevé réel de Guy). Autorisés : lecture, `grep`, `sed -n`, `wc`, `find`, `git log`/`show`/
`diff`/`blame`/`grep`, `gh api` en lecture, `pdftotext` vers la sortie standard.
