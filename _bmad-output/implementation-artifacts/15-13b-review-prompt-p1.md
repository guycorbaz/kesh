# Prompt — revue de code P1, Story 15-13b

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-13b`, branche `story/15-13b-sauvegarde-persistante`. **Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-15-13b diff 200f5e79..7ff477ab` (un seul diff aplati ; le code,
les tests, les manuels, la doc — les fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-13b-sauvegarde-avant-import-persistante.md` (AC, tâches, Dev Agent Record, Change Log). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #552, #576 (#558 pour le contexte). Registre : `epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Choix C-15-13-1 à 29, C-15-13b-1 à 4. Code touché : config.rs (KESH_ADMIN_BACKUP_DIR via env_nonempty, DEFAULT_ADMIN_BACKUP_DIR /data/backup), routes/admin.rs (write_backup_file : .partial 0600, sync_all, vérification du nom final, rename ; dossier 0700 ; AdminPreImportBackupFailed et avant_sauvegarde aux sites antérieurs), errors.rs, catalogues 4 locales (le message nomme les deux pistes), compose (montage ./backup), .gitignore/.dockerignore ancrés, tests (configuration_transmise.rs, tests 11, 12, 15, 16, 19, 20), manuel et PDF, DOCKER_START, recettes (CLAUDE.md : une ligne), README, CHANGELOG. Axes : write_backup_file — le .partial est-il nettoyé dans tous les chemins d'échec et JAMAIS celui d'un autre écrivain ? la sauvegarde n'est-elle jamais visible sous son nom final incomplète ? ; le tri pré/post-sauvegarde des sites ; la garde lexicale du test 20 ; le montage et la constante liés par test ; le manuel (rapatriement sudo cp) et le PDF aplati ; la ligne modifiée de CLAUDE.md (une seule).

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

Rapport complet dans `target/gate-logs/15-13b-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers `target/gate-logs/`.
