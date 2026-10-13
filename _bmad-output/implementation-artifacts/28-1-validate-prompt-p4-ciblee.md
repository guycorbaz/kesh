# Prompt — validation P4 **ciblée** de la spec, Story 28-1 (les boutons d'action destructive se voient)

*Versionné le 2026-10-13. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **le seul
commit de la remédiation P3** (`CLAUDE.md`, § *La passe ciblée* : quand une boucle converge, ce qu'il reste à relire
n'est plus la story, c'est la dernière remédiation). Trend des MEDIUM distincts : P1 3 HIGH + 12 → P2 11 → P3 4,
0 HIGH depuis la P2.*

Dépôt `/home/gcorbaz/devel/kesh-28-1`, branche `story/28-1`.
**Périmètre : `git show 2d794013`** (remédiation P3), soit un **diff unique, aplati** — ne lis pas la séquence des
commits antérieurs, et ne cherche pas une ligne du diff par son numéro de hunk : relis-la dans le fichier **final**.
Fichiers touchés : la fiche `_bmad-output/implementation-artifacts/28-1-boutons-destructifs.md` et le registre
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` (entrées C-28-1-32 à C-28-1-36 et quatre
rectifications en italique). Rapports P3 que ce commit remédie : `/home/gcorbaz/devel/kesh-gate-logs/28-1-validate-p3-{R,F}.md`.

La fiche est une **spec** : le code qu'elle prescrit (spec E2E, gardes Vitest, helpers) **n'existe pas encore**.

## La lentille — chasseur de régressions de la remédiation P3

Pour **chaque hunk** du commit, vérifie contre le code et les paquets au sol (`frontend/`, `frontend/node_modules/`) :

1. **Délais de l'AC8 (C-28-1-32)** : `test.describe.configure({ timeout })` et `expect.configure({ soft, timeout })`
   existent-ils dans Playwright 1.59 (`frontend/node_modules/playwright/types/test.d.ts`, `grep -nF "configure("`) ?
   Le budget écrit (cinq assertions rouges en (a), trois en (b), 2 s chacune) tient-il dans 120 s ? Les assertions
   de **montage** non `soft` (`toHaveCount(1)`, `toBeVisible`) peuvent-elles rougir en phase rouge (T0) ?
2. **Cadrage des captures (C-28-1-33)** : `page.screenshot({ clip })` prend-il des coordonnées dans le même repère
   que `locator.boundingBox()` (`playwright-core/lib/server/screenshotter.js`, `page.js`) ? 8 px couvrent-ils
   l'anneau `ring-3` et le contour de `frontend/src/app.css:62-65` ? `page.mouse.move(0, 0)` quitte-t-il bien le
   bouton `ds-destructive` sur `/design-system` (`frontend/src/routes/design-system/+page.svelte`) — n'y a-t-il
   pas un élément survolable en (0, 0) qui changerait la capture ? L'assertion « fond au repos » ajoutée au step de
   focus est-elle vraie au repos **et** compatible avec `:focus-visible` (aucune règle de fond au focus) ?
3. **Relevé de l'AC7 (4) (C-28-1-34)** : le `=` ajouté aux séparateurs produit-il un jeton parasite **neuf** ?
   Refais : `grep -rhoE '[^ "'"'"'`{}()]*=(bg|text|border|ring|outline|fill|stroke|divide|placeholder|decoration|caret|accent)-[a-z][a-z0-9-]*' frontend/src --include=*.svelte --include=*.ts | grep -v '^class:'`.
   Recompte les directives : `grep -rnE 'class:[a-z-]*(bg|text|border|ring|outline)-' frontend/src --include=*.svelte | grep -v '\.test\.'`
   (la fiche écrit 16, dont 15 de couleur). Les trois auto-tests écrits rendent-ils bien ce que la fiche annonce,
   par la règle de relevé **telle qu'écrite** (préfixes jusqu'au dernier `:`, `!` initial, opacité `/<n>`) ?
   M22 est-elle tuée par l'auto-test qu'on lui attribue ?
4. **Assertion de version (C-28-1-35)** : `@tailwindcss/node` n'exporte-t-il vraiment pas son `package.json`
   (`frontend/node_modules/@tailwindcss/node/package.json`, champ `exports`) ? Sa dépendance `tailwindcss` est-elle
   exacte ? Le chemin `node_modules/@tailwindcss/node/node_modules/tailwindcss` est-il celui où npm placerait une
   copie divergente ? `frontend/package.json` : quels spécificateurs portent `tailwindcss` et `@tailwindcss/vite` ?
5. **Décomptes et références (C-28-1-36)** — recompte depuis le code, ne relis pas les nombres :
   - classe A : 17 confirmations parmi les 24 `variant="destructive"`, 13 à côté d'un *Annuler* `outline`, 4
     `ghost`, 13 dans un `Dialog.Root` et 4 en ligne (`dunning:300`, `vat-rates:366`, `api-keys:404`,
     `bank-accounts:586`) ; `accounts/+page.svelte:712` et `:717` ;
   - AC4 : six `<button>` bruts en `text-destructive` (les `<button` sont souvent multilignes : lire l'élément
     englobant, non la ligne) ; libellés *Annuler le lot*, *Annuler le règlement*, *Annuler la facture* ;
     bordure `border` sans couleur → `currentcolor` (`frontend/node_modules/tailwindcss/preflight.css`) ;
   - le septième `ghost` à icône, `settings/fiscal-years/+page.svelte:417` (*Clôturer*), et l'absence d'un
     huitième ; `BankImportUpload.svelte:344` ; `bank-import/[id]/+page.svelte` sans aucun `variant=` ;
   - angle mort 11 : « un seul des cinq sites dépend du jeton » contre `fiscal-years:603`, `email-templates:439`,
     `ContactPersonsManager:114`, `BankProfileList:79`, `bank-import/[id]:143` ;
   - les recomptes du Change Log P3 : 14 sujets, 4 MEDIUM, 10 LOW (contre les deux rapports P3), mutations M1-M22
     (21 actives), angles morts 1-12, choix C-28-1-32 à 36.
6. **Propagation** : le symptôme corrigé survit-il ailleurs, hors des Change Logs et des rectifications
   historiques (consigne : les passes passées ne se réécrivent pas) ? Grepe la **valeur** :
   `grep -nE "M1-M21|C-28-1-31\b|\b345\b|[Ss]eize|douze à côté|quatre \`<button>\`|version exacte|\{\}\(\)\]|issue à ouvrir|issue demandée" _bmad-output/implementation-artifacts/28-1-boutons-destructifs.md _bmad-output/implementation-artifacts/epic-28-choix-autonomes.md`,
   puis trie chaque occurrence : historique légitime ou résidu.

⚠️ **Erreur de catégorie** (consigne 5 des agents) : un livrable **prescrit par la fiche mais absent du code** n'est
**pas** un défaut — « `destructive-buttons.spec.ts` n'existe pas », « `softExpect` n'est défini nulle part », « la
devDependency `@tailwindcss/node` manque à `package.json` », « M22 n'est pas jouée » sont **attendus**. Un finding ne
porte que sur ce que **le texte du commit affirme du code existant**, ou sur ce qu'il **prescrit de faux ou
d'infaisable**.
⚠️ **Grep ground-truth obligatoire** : toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée
(`-F` : le code est plein de `(`, `[`, `.`, `*`). Un CRITICAL ou HIGH qui affirme qu'un patch « n'a pas été
appliqué » ou qu'une ligne « contient encore » quelque chose se vérifie au `grep -nF` sur le **fichier final**
avant d'être écrit — les numéros de ligne d'un hunk ne sont pas ceux du fichier final.

## Ce que tu rends

Findings numérotés `P4-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), correction proposée, et **origine** (né de la remédiation P3, ou antérieur). ⛔ **La liste des axes
exercés ET non exercés** (les six ci-dessus, avec pour chacun ce qui a été réellement lu ou grepé) — un « 0 finding »
sans elle ne compte pas comme passe. Bilan recompté sur tes propres titres. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-1-validate-p4-ciblee.md` ; dernier message : le chemin du rapport, le bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. **Aucune commande qui écrit, compile ou
exécute**, et en particulier **aucun interpréteur** : `node`, `npx`, `npm`, `python`, `python3`, `perl`, `ruby`,
`deno`, `bun`, `bash`/`sh` sur un script, `vitest`, `playwright` ; ni `scripts/*` (dont **`scripts/prepare-release.sh`**,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`, `cargo`, `docker`, toute requête SQL,
`git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `gh` en écriture,
`sed -i`. Rien de `~/kesh-test/` (relevé réel de Guy) : aucune donnée réelle dans le rapport. Autorisés : lecture,
`grep`, `sed -n`, `awk` en lecture, `wc`, `git log`/`show`/`diff`/`blame`, `gh issue view` / `gh api` en lecture.
