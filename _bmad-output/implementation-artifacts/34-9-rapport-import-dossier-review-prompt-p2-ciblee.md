# Revue de code P2 ciblée — Story 34-9 (#969, #974) : la remédiation de la revue P1

**Une seule lentille**, en contexte frais, braquée sur **le seul commit de la remédiation** (`CLAUDE.md`, § *La passe
ciblée*). Question unique : **la remédiation P1 a-t-elle introduit un défaut, ou laissé une contradiction ?** Modèle :
Haiku, Sonnet ou Opus (une passe ciblée peut aller à Haiku ; toute affirmation de présence ou d'absence s'appuie alors
sur une sortie `git grep -nF` copiée).

## Cadre

- Worktree : `/home/gcorbaz/devel/kesh-e349`, branche `story/34-9-rapport-import-dossier`.
- **Périmètre** : `git diff e74f55f6..HEAD -- crates frontend docs` (la remédiation : le commit `8ddeee5c` et le
  correctif du clignotement qui le suit, lus aplatis, d'un seul tenant). Le reste de la story a été revu
  en P1 (rapports `/home/gcorbaz/devel/kesh-gate-logs/e349-review-p1-{B,E,A}.md`, aucun finding au-dessus de LOW).
- Fiche : `_bmad-output/implementation-artifacts/34-9-rapport-import-dossier.md` (AC-1 à AC-8 ; Change Log, entrée
  « Revue de code P1 ») ; choix `e34-choix-autonomes-e349.md`.
- **Rapport** : `/home/gcorbaz/devel/kesh-gate-logs/e349-review-p2-ciblee.md`. Chaque finding : sévérité, fichier et
  ligne, preuve copiée, correction proposée. Le rapport se termine par les **axes exercés** et **non exercés** : une
  passe qui ne les déclare pas ne compte pas.

## ⛔ Lecture seule

Aucune écriture hors du rapport, aucune redirection, aucun fichier temporaire. Interdits : `cargo`, `npm`, `npx`, `make`,
`latexmk`, tout `scripts/*.sh` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`),
`git checkout`, `git switch`, `git commit`, `git stash`, `git reset`, `gh` qui écrit, `docker`. Permis : `git show`,
`git diff`, `git log`, `git grep`, `sed -n`, `cat`, `ls`, `/usr/bin/grep` (dans ce shell `grep` est une fonction).

## Ce que le commit change, et les axes à exercer

1. **`with_moved_to`** (`crates/kesh-api/src/inbox_import.rs`) : `None | Some(Null)` → map vide, placé **avant** le bras
   `Some(autre)`. L'ordre des bras est-il juste ? Un `details` objet est-il toujours gardé ? Test unitaire étendu.
2. **Test `duplicate_details_names_the_document_or_nothing`** : la fixture `ImportedSupplierInvoice` compile-t-elle avec
   tous les champs de l'entité (`crates/kesh-db/src/entities/imported_supplier_invoice.rs`) ? Les assertions
   prouvent-elles les trois champs, `null` compris ?
3. **Test `unstable_file_is_skipped_with_its_reason`** (`crates/kesh-api/tests/inbox_import_e2e.rs`) : garde `Ecrivain`
   (arrêt et jointure à la destruction, panique comprise), cinq essais à fichiers distincts (`noise_png(64, 100 + essai)`).
   Peut-il **passer à vide** ? Un essai importé peut-il faire d'un essai suivant un doublon ? L'`return` après les
   assertions, le `panic!` final : un faux vert est-il possible ?
4. **`+page.svelte`** :
   - `reportHasProblem` lit `truncatedAt` (cadre bordé à la troncature seule) ;
   - `noticeOf` : branche neuve « non importés **et** limite » avant « non importés » ; l'ordre des quatre branches,
     et la valeur de `n` ;
   - lien d'un doublon `to_complete` : seulement si `toComplete` contient le document, sinon le texte
     `-report-duplicate-not-in-list` — mais pas pendant `importing` (jusqu'au rechargement de la liste), pour ne pas
     faire clignoter le texte (corrigé par le second commit, voir ci-dessous). **Réactivité** : `toComplete` change au
     rechargement, au retrait (`discard`), et la ligne du rapport suit-elle ? Rechargement en échec : `importing`
     repasse à `false` dans le `finally`, le texte paraît-il alors ? Effet sur l'E2E (`inbox-import.spec.ts`, le lien
     est attendu après le rechargement).
5. **Textes** : deux clés neuves aux quatre catalogues (`-notice-not-imported-truncated`,
   `-report-duplicate-not-in-list`), repli Svelte égal au catalogue fr, aucun `->` ; compteurs des gardes
   (`i18n-keys.test.ts` 2942 → 2944, `i18n-un-repli-par-cle.test.ts` 426 → 428) **recomptés**, non crus.
6. **Vitest** (`import-page.test.ts`) : le cas mixte charge désormais le document 7 dans la liste ; nouveau message du
   toast mixte ; cas « limite seule » avec cadre ; cas « ligne absente de la liste » ; cas du rechargement en échec
   avec `truncatedAt: null`. Les assertions disent-elles ce que le code fait, et rougiraient-elles sur la régression
   qu'elles visent ?
7. **Documentation** : `docs/api-external.md` (les deux phrases corrigées : « l'état lu par l'import, relu après
   l'échec quand le doublon naît d'une course », « document disparu entre-temps, ou relecture en erreur ») contre le
   code ; `docs/manual/fr/user-manual.tex` (lien « tant qu'elle y figure », cadre bordé, « si des documents ont été
   importés ») contre le Svelte. **PDF** : non régénéré dans la story (C-e342-20), à l'intégration ; le constater, ne
   pas le compter comme défaut.
8. **Captures** (`/home/gcorbaz/devel/kesh-gate-logs/e349/captures/`, reprises après la remédiation) et journal
   `e2e-r5/capture.log` (nombre de toasts visibles avant et après le second import) : le second cadre de la capture P1
   était-il un empilement de deux imports, et la nouvelle capture n'en montre-t-elle qu'un ?
9. **Gates déclarés** au Change Log (entrée « Revue de code P1 ») contre leurs journaux
   (`/home/gcorbaz/devel/kesh-gate-logs/e349/p1-*.log`, `gates3.log`, `e2e-r4/playwright.log`) : ne déclarer que ce qui
   a tourné ; les sept échecs E2E nommés contre `docs/testing.md`.
10. **Messages de commit** : `git log 665de151..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
    doit être vide.
