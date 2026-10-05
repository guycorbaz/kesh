# Prompt — revue de code P1, Story 25-4-d1 (les comptes de solde)

*Versionné le 2026-10-01. Deux lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d1-comptes-de-solde`. **Diff à revoir : `git diff 835130d9 1d95c91b`** — un seul commit d'implémentation. Fiche :
`_bmad-output/implementation-artifacts/25-4-d1-comptes-de-solde.md` (AC 1–7). CR : `gh issue view 384`. Règles :
`CLAUDE.md` (§ *Migration breaking policy*, § *Review Iteration Rule*).

## Lentilles

- **A — Blind + edge-case hunter** : le diff et le code environnant.
  - Plan : `validate_designated_account` factorisé — les messages du marqueur d'arrondi sont-ils **mot pour mot**
    ceux d'avant (`git show 835130d9:crates/kesh-core/src/chart_of_accounts/mod.rs`) ? Le double marqueur est-il
    contrôlé **avant** ou **après** les validations par nature, et le message est-il juste dans tous les cas ? Une
    graphie JSON inconnue (`"bankfees"`) est-elle refusée ou ignorée ?
  - 3805 « Pertes sur créances » en **Revenue** sous le parent `30` : cohérent avec le reste du code (rôles,
    `is_postable`, rapports, balance, compte de résultat, seed de démonstration `kesh-seed`, tests qui comptent les
    comptes d'une société) ? Un compte ajouté au plan casse-t-il un test, un E2E, un rapport ?
  - Désignation : `chart_designated_accounts` lit le plan une fois et verrouille quatre comptes `FOR UPDATE` dans un
    ordre fixe — risque d'interblocage avec un autre chemin qui verrouille les mêmes comptes dans un autre ordre ?
    Plan introuvable → tout `None` ; un compte archivé → `None` sans erreur.
  - Route : `resolve_designated_account` (absent préservé, `null` effacé, validé **seulement s'il change**) — le
    refactor du compte d'arrondi préserve-t-il exactement son comportement d'avant ? Les quatre champs peuvent-ils
    désigner **le même** compte, et est-ce un problème ?
  - Frontend : `writeOff` en `$state` d'objet, `bind:value={writeOff[field.key]}` dans un `{#each}` sur un
    `$derived` — réactivité correcte, valeur `null` envoyée pour « — Sélectionner — », compte archivé déjà choisi
    gardé visible ?
- **C — Acceptance auditor** : chaque AC tenu ? Garde-fous P5/P6/P7/P8 **recomptés depuis la source** (`ls
  crates/kesh-db/migrations/*.sql | wc -l`, les compteurs de l'audit, `migrations_upgrade_path.rs`, le squash, le
  `migrations.sha384` recalculé). **Pars du symptôme** : `grep -rn "default_rounding_account_id\|defaultRoundingAccountId"
  crates/ frontend/src` — tout site qui énumère les champs des réglages porte-t-il aussi les trois nouveaux ?
  `sitesTotal` recompté ? Les tests prouvent-ils ce qu'ils disent (le test de comptage fige-t-il vraiment 86/86/83 ?
  les tests d'API généralisés couvrent-ils chaque champ indépendamment ?) Manuel admin
  (`docs/manual/fr/admin-manual.tex` et **PDF aplati** : `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
  `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`) et **manuel
  utilisateur** : décrit-il le contenu des plans ou les paramètres de facturation d'une façon que la story rend
  fausse ? CHANGELOG. Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu), scénario d'échec,
correction. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF`. ⛔ La liste
des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `cargo check`, `pdftotext` et expériences dans le scratchpad.
