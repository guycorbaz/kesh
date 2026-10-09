# Prompt — revue de code P1, Story 15-12a

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-12a`, branche `story/15-12a-cloture-dans-l-ordre`. **Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-15-12a diff 5e4bec50..1f477526` (un seul diff aplati ; le code,
les tests, les manuels, la doc — les fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-12a-cloture-dans-l-ordre.md` (AC, tâches, Dev Agent Record, Change Log). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #543. Registre : `epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Choix C107 à C123, C-15-12a-1. Code touché : crates/kesh-db/src/repositories/fiscal_years.rs (protocole de verrouillage de close (a)-(e), reopen LIFO, garde de create, LOCK_IN_COMPANY_SQL partagé avec update_name), routes fiscal_years (rejeu de close et create), AppError::FiscalYearBeforeClosedYear, écran des exercices, tests de concurrence 13 a-d et HTTP 9-10, manuels et PDF. Axes : le protocole C119 contre son code réel (fenêtres entre (b) et (b'), relecture (d), ordre réel des verrous, cycles avec create/reopen/update_name/contre-passation) ; les tests de concurrence prouvent-ils ce qu'ils disent (point 1 : la création bute dans find_overlapping et non dans sa garde — l'AC 13 b est-il encore prouvé ?) ; mutations (ix)/(x) jouées par une seule tentative ; le message neuf dans les 4 locales et les replis ; le manuel et le PDF aplati ; #568/#569 cités.

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Défauts de correction, régressions, erreurs de concurrence
  (verrous, REPEATABLE READ, ordre d'acquisition), erreurs rendues au client, tests qui passeraient à vide (un test qui
  ne mord pas sur la mutation qu'il prétend couvrir), code mort, duplication (DRY), doc-comments devenus faux.
- **E — Edge Case Hunter** : chaque branche et chaque borne du code modifié — entrées vides, nulles, multiples, en
  doublon, d'une autre société, archivées ; chemins par clé d'API ; lots partiellement en échec ; locales ; et les
  **chemins NON modifiés qui devraient l'être** (inventorier les sites non résolus de la même famille par `grep`).
- **A — Acceptance Auditor** : chaque AC de la fiche contre le code ET les tests (un AC sans test qui le prouve est
  un finding) ; le Dev Agent Record ne déclare-t-il que ce qui a tourné (chiffres recomptés : `grep -c '#\[sqlx::test\]\|#\[test\]\|#\[tokio::test\]'` aux deux bornes) ;
  le **manuel** (`docs/manual/fr/*.tex` **et PDF aplatis** : `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' '` vers
  `target/gate-logs/`, ligatures ﬀ/ﬁ/ﬂ normalisées), `docs/api-external.md`, CHANGELOG, i18n 4 locales.

## Ce que tu rends

Rapport complet dans `target/gate-logs/15-12a-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers `target/gate-logs/`.
