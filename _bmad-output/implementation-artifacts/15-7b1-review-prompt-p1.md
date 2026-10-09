# Prompt — revue de code P1, Story 15-7b1

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-7b1`, branche `story/15-7b1-trace-demonstration`. **Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-15-7b1 diff 200f5e79..87ababd4` (un seul diff aplati ; le code,
les tests, les manuels, la doc — les fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-7b1-trace-demonstration.md` (AC, tâches, Dev Agent Record, Change Log). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #434, #544 (#538 pour le contexte). Registre : `epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Choix C-15-7-*, C-15-7b1-1. Code touché : kesh-seed (SeedAttemptError, is_seed_retryable, acteur (user_id, api_key_id), dernière transaction sous retry_with : verrou d'état, clear_stub_in_tx, réglages puis taux, étape, installation.demo_seeded), handler seed-demo (400 sur StepAlreadyCompleted, UPDATE is_stub et lecture de ui_mode retirés), registre 105/5/2, une action et ses 4 libellés, helper create_key_via_http remonté dans tests/common, manuels FR et PDF, CHANGELOG. Axes : la dernière transaction est-elle vraiment atomique et rejouable (rien lu ou écrit hors transaction, booléens relus à chaque tentative) ; ui_mode lu hors verrou (mutation déclarée non couverte : est-ce acceptable ?) ; l'audit for_actor pour une clé d'API ; le refactor du helper de test ne change-t-il le sens d'aucun test d'api_keys_e2e ; la partition du registre recomptée ; la sortie de démonstration au manuel (KESH_PRODUCTION_RESET) ; PDF aplati.

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

Rapport complet dans `target/gate-logs/15-7b1-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers `target/gate-logs/`.
