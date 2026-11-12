# Revue de code P3 ciblée — Story 34-9 (#969, #974) : la remédiation de la passe ciblée P2

**Une seule lentille**, **Sonnet**, en contexte frais, braquée sur **les seuls commits de code de la remédiation P2**
(`CLAUDE.md`, § *La passe ciblée*). Question unique : **la remédiation P2 a-t-elle introduit un défaut, ou laissé une
contradiction ?**

## Cadre

- Worktree : `/home/gcorbaz/devel/kesh-e349`, branche `story/34-9-rapport-import-dossier`.
- **Périmètre** : `git diff 7740ad67..e071f051 -- crates frontend docs`, soit les deux commits de code de la remédiation
  P2 lus aplatis, d'un seul tenant : `f1b8e65c` (Svelte, catalogues, manuel, Vitest, gardes i18n) et `e071f051`
  (`listReloadFailed` remis à faux par tout rechargement réussi — défaut relevé en écrivant ce prompt, corrigé avant la
  passe ; sa correction est à relire). Les passes précédentes : P1 (`/home/gcorbaz/devel/kesh-gate-logs/e349-review-p1-{B,E,A}.md`, 0 au-dessus
  de LOW), P2 ciblée (`/home/gcorbaz/devel/kesh-gate-logs/e349-review-p2-ciblee.md`, 1 MEDIUM, 3 LOW).
- Fiche : `_bmad-output/implementation-artifacts/34-9-rapport-import-dossier.md` (Change Log, entrée « Passe ciblée
  P2 ») ; choix `e34-choix-autonomes-e349.md`, en particulier C-e349-11 et C-e349-12.
- **Rapport** : `/home/gcorbaz/devel/kesh-gate-logs/e349-review-p3-ciblee.md`. Chaque finding : sévérité, fichier et
  ligne, preuve copiée, correction proposée. Le rapport se termine par les **axes exercés** et **non exercés** : une
  passe qui ne les déclare pas ne compte pas.

## ⛔ Lecture seule

Aucune écriture hors du rapport, aucune redirection, aucun fichier temporaire. Interdits : `cargo`, `npm`, `npx`, `make`,
`latexmk`, tout `scripts/*.sh` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`),
`git checkout`, `git switch`, `git commit`, `git stash`, `git reset`, `gh` qui écrit, `docker`. Permis : `git show`,
`git diff`, `git log`, `git grep`, `sed -n`, `cat`, `ls`, `/usr/bin/grep` (dans ce shell `grep` est une fonction),
`pdftotext … -` vers la sortie standard.

## Axes

1. **P2-1, le texte d'un doublon en attente hors de la liste** (`+page.svelte`, branche `existingStatus ===
   'to_complete' && !importing && !toComplete.some(…)`) : `discardedHere` (posé au retrait réussi, vidé à la fin de
   chaque import), `listReloadFailed` (posé dans le `finally` de `runImport`, **remis à faux par `reloadList`** à tout
   rechargement réussi, commit `e071f051`). Les trois textes sont-ils choisis dans le bon ordre ? `discardedHere`
   survit-il là où il ne devrait pas (retrait avant tout import, second import) ? La remise à faux dans `reloadList`
   couvre-t-elle tous les sites de rechargement (`onMount`, `safeReloadList`, `runImport` ; les recenser par
   `git grep -n "reloadList\|listImported(" -- 'frontend/src/routes/(app)/supplier-invoices/import/'`), et
   l'affectation du `finally` de `runImport` (`listReloadFailed = reloadFailed`) reste-t-elle cohérente avec elle ? La
   consigne « actualisez » ne paraît-elle plus que dans le cas d'un rechargement en échec ? Greper le symptôme
   (`actualisez`, `aktualisieren`, `aggiorna`, `refresh`) sur les catalogues, le Svelte et le manuel.
2. **P2-2, l'ancien rapport retiré au lancement** (`report = null` en tête de `runImport`) : effet sur le premier
   rendu, sur un import refusé en `409` (`INBOX_IMPORT_ALREADY_RUNNING`), sur la notification ; `fresh` reste-t-il
   défini sur tous les chemins ?
3. **P2-3, les tests** (`import-page.test.ts`) : le cas « pendant le rechargement, le lien reste » tient-il la
   promesse en suspens au bon moment, et rougit-il sans la garde `!importing` (mutation **M10**,
   `/home/gcorbaz/devel/kesh-gate-logs/e349/mut/M10-vitest.log`) ? Le cas du second import rougit-il sans
   `report = null` (mutation **M11**, `mut/M11-vitest.log`) ? Le cas « un rechargement réussi après coup ôte
   l'invitation à actualiser » rougit-il sans la remise à faux (mutation **M12**, `mut/M12-vitest.log`) ? Les cas
   « retiré » et « liste non rechargée » disent-ils ce que fait le code ?
4. **Textes** : clé neuve `-report-duplicate-not-in-list-reload-failed` et valeur changée de
   `-report-duplicate-not-in-list`, aux quatre catalogues ; replis Svelte égaux au catalogue fr ; aucun `->` ;
   compteurs des gardes recomptés (`i18n-keys.test.ts` 2944 → 2946 sites, `i18n-un-repli-par-cle.test.ts` 428 → 429
   clés) — non crus.
5. **Manuel** (`docs/manual/fr/user-manual.tex`, paragraphe du doublon « déjà importé ») contre le Svelte. **PDF** : non
   régénéré dans la story (C-e342-20) ; le constater, ne pas le compter comme défaut.
6. **Captures** (`/home/gcorbaz/devel/kesh-gate-logs/e349/captures/`) : `p2-limite-seule.png` (cadre bordé, toast
   « la limite par import est atteinte »), `p2-toast-combine.png` (toast « … non importé(s), et la limite … »),
   `p2-doublon-retire.png` (ligne « retiré de la liste », sans lien) ; journal `e2e-r7/capture-p2.log`. L'écran
   montré est-il celui que décrivent le code et le manuel ?
7. **Gates déclarés** au Change Log (entrée « Passe ciblée P2 ») contre leurs journaux
   (`/home/gcorbaz/devel/kesh-gate-logs/e349/gates4.log`, `p2-*.log`, `e2e-r6/playwright.log` ; au dernier commit de
   code `e071f051` : `gates5.log`, `p3pre-*.log`, `e2e-r8/playwright.log`) : ne déclarer que ce qui a tourné ; les échecs E2E nommés contre `docs/testing.md` (sept de la KF-029, #97).
8. **Messages de commit** : `git log 665de151..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
   doit être vide.
