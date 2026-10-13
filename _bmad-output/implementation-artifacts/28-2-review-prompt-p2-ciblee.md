# Prompt — revue de code P2 **ciblée**, Story 28-2 (une erreur d'API ne s'affiche plus « [object Object] »)

*Versionné le 2026-10-13. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **la seule
remédiation de la revue de code P1** (`CLAUDE.md`, § *La passe ciblée* ; D6, rétrospective de l'Epic 25 : Haiku
réservé aux passes ciblées). Trend : P1 (Sonnet, lentilles B, E, A) **0 CRITICAL, 0 HIGH, 1 MEDIUM, 14 LOW**.*

Dépôt `/home/gcorbaz/devel/kesh-28-2`, branche `story/28-2`. **Périmètre : la remédiation P1, lue en DIFF UNIQUE
aplati** — ne lis pas la séquence des commits (règle Haiku du `CLAUDE.md` : les numéros de ligne d'un hunk de
commit intermédiaire ne sont pas ceux du fichier final) :

```sh
git diff 12ec02c8 HEAD -- frontend                       # code et tests (un seul commit de code : d514132e)
git diff 12ec02c8 HEAD -- _bmad-output/implementation-artifacts/28-2-erreurs-object-object.md \
                          _bmad-output/implementation-artifacts/epic-28-choix-autonomes.md
```

Rapports P1 que ce diff remédie (à lire) : `/home/gcorbaz/devel/kesh-gate-logs/28-2-review-p1-B.md`,
`28-2-review-p1-E.md`, `28-2-review-p1-A.md`. Journal des mutations de la remédiation :
`/home/gcorbaz/devel/kesh-gate-logs/28-2-mutations-p1.log` (P1-M1 à P1-M14).

Fichier de production touché : **un seul**, `frontend/src/routes/(app)/reports/+page.svelte` (`formatError`). Tout le
reste du diff `frontend` est du test (`*.test.ts`, la garde `erreurs-en-texte.test.ts`, la spec E2E).

## La lentille — chasseur de régressions de la remédiation P1

Pour **chaque hunk**, vérifie contre le fichier final :

1. **`formatError` (B-1 = A-4, C-28-2-33 révisé)** : la suppression de `return err.message` change-t-elle un rendu
   autre que celui du message vide ? Une `ApiError` `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` **sans** `details` complets,
   une `ApiError` ordinaire, une valeur non `ApiError` : que rend la fonction, et chaque cas est-il tenu par un test
   de `reports-page.test.ts` ? Les autres appelants de `formatError` dans le fichier (exports `:403`, `:437`, `:473`,
   `:505` environ — relis-les) supposaient-ils `err.message` brut ?
2. **G5 élargie (E-M1, C-28-2-36)** : l'expression régulière de `reglesDeLiaison` relève-t-elle bien la liaison
   **opérande entier** d'un `+` / `+=` des deux côtés, et **pas** un champ (`err.message`, `err?.code`, `err[0]`,
   `err(…)`), ni `err2`, `myerr`, ni `x++` / `++x` ? Cherche une **forme réelle du dépôt** que l'élargissement ferait
   rougir à tort (faux positif) ou une forme de concaténation qu'il manque encore (faux négatif) : `String.prototype.concat`,
   `[…, err].join(…)`, `'a' + (err)`, `'a' + err as string`, `err\n+ 'a'` (opérateur en début de ligne). Une forme
   manquante **et** absente du tableau des angles morts du préambule est un finding ; présente au tableau, elle ne
   l'est pas.
3. **Exemption `console` (E-L3)** : les deux fictives neuves (même ligne après l'appel ; premier argument)
   tiennent-elles ce que leur titre annonce ? Le code de `detecter` est-il inchangé (seuls des tests ajoutés) ?
4. **Préambule (E-L4, B-3)** : chaque ligne neuve du tableau « ce qu'elle ne voit pas » est-elle **vraie** au code
   de la garde (rejoue mentalement la règle concernée) ? « Aucune de ces formes n'existe dans `src/` » : vérifie par
   `grep` (`grep -rnE "\?\.toString\(|String\([^)]*\?\?|String\([^)]*\bas\b|\.then\([^)]*,\s*\(?\w+\)?\s*=>" frontend/src --include=*.ts --include=*.svelte | grep -v '\.test\.'`).
5. **Tests neufs (E-L1, E-L2, E-L5)** : chacun prouve-t-il ce qu'il nomme ? L'appel API est-il constaté avant
   l'assertion du texte ? Le remplacement de `archiveBankAccount: vi.fn()` / `createApiKey: vi.fn()` /
   `revokeApiKey: vi.fn()` par des mocks pilotés change-t-il le sens d'un test **existant** du même fichier (valeur
   de retour par défaut `undefined` conservée ?) ? Les sorties rouges de P1-M1 à P1-M14 (journal) nomment-elles le
   test attendu ?
6. **E2E (E-L6)** : seul le titre du scénario 2 change-t-il ? Le titre neuf est-il exact au regard de l'en-tête du
   fichier ?
7. **Fiche et registre (A-1, A-2, A-3, E-L7)** : recompte, ne relis pas — `isApiError(` hors tests à `4b31fcac` et
   au `HEAD` (commande d'A-1 dans `28-2-review-p1-A.md`) ; fichiers qui le portent (`git grep -l`, mêmes filtres) ;
   `git diff --name-only 4b31fcac HEAD | grep -E "credit-notes|invoices|payment-batches|supplier-invoices"` ; nombre
   de choix (`grep -c "^### C-28-2-" epic-28-choix-autonomes.md`) contre « C-28-2-1 à C-28-2-38 » du `## Status` ;
   ligne M19 contre le journal `28-2-mutations.log`.
8. **Manuel contre code (obligatoire), `.tex` ET PDF aplati** : la remédiation change-t-elle un texte que voit
   l'utilisateur (alerte des rapports sur une `ApiError` au message vide) au point que le manuel doive le dire ?
   `grep -n` sur `docs/manual/fr/*.tex`, et `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   pour « rapport », « message d'erreur », « Erreur inattendue ».
9. **Propagation** : un symptôme corrigé à un site l'est-il partout ? Greper la **valeur** : `return err.message`
   dans `frontend/src` hors tests — attendus : `errorMsg = e.message` des deux modales (B-2 laissé à #644,
   C-28-2-37) et trois `return err.message` **hors du périmètre de la story**, chacun dans une branche `ApiError`
   suivie d'un repli traduit (`export/+page.svelte:28`, `supplier-invoices/import/+page.svelte:301`,
   `AdminRestorePanel.svelte:84`) : même cas du message vide, classe D, #644 — vérifie-le, ne le compte pas comme
   résidu s'il se confirme ;
   `chaîne vide` dans la garde et la fiche ; `\b164\b`, `\b55\b` dans le Dev Agent Record ; `message réseau du
   catalogue` sur le dépôt.

⚠️ **Entrées passées** : les entrées antérieures du Change Log et du registre gardent leurs chiffres d'alors
(consigne 15) — ce n'est pas un résidu.
⚠️ **Grep ground-truth obligatoire** : toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée, sur
le **fichier final** (`git show HEAD:<chemin>` ou le fichier du worktree). Un CRITICAL ou HIGH qui affirme qu'un patch
« n'a pas été appliqué » se vérifie ainsi avant d'être écrit.
⚠️ **Erreur de catégorie** : un livrable que la fiche attribue à une story **future** ou à #644 (classe D) n'est pas un
défaut de cette remédiation.

## Ce que tu rends

Findings numérotés `P2-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), origine (remédiation P1 / rédaction antérieure), correction proposée. ⛔ **La liste des axes exercés
ET non exercés** (les neuf ci-dessus) — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-2-review-p2-ciblee.md` ; dernier message : le chemin du rapport, le bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`,
`cargo`, `npm`, `npx`, `node`, `vitest`, `playwright`, `docker`, `bash` ou `sh` sur un script, **tout interpréteur**
(`python`, `python3`, `perl`, `ruby`, `deno`, `awk` avec un programme qui écrit), `gh` en écriture, toute requête SQL.
Rien de `~/kesh-test/` (relevé réel de Guy). Autorisés : lecture, `grep`, `sed -n`, `wc`, `find`, `git log`/`show`/
`diff`/`blame`/`grep`, `gh api` en lecture, `pdftotext` vers la sortie standard.
