# Prompt — revue de code P1, Story 15-8b

*Versionné le 2026-10-08. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-8`, branche `story/15-8b-supprimer-une-ecriture`. **Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-15-8 diff 42b13a62..9c890a02` (un seul diff aplati ; le code,
les tests, les manuels, la doc — les fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-8b-supprimer-une-ecriture.md` (AC, tâches, Dev Agent Record, Change Log). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #532, #543, #431. Registre : `epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Base 42b13a62 (main 52a9b19b avec la 15-8a + clôture de spec) ; diff de code 42b13a62..9c890a02. Suppression d'une écriture (DELETE) dans le cadre de la 15-8a. Axes : ordre des contrôles et des verrous du DELETE (exercice postérieur clos, garde de modification, verrou de période, FOR UPDATE avant toute lecture — mutation 1451), retrait complet du gel (EntryIsPosted, ENTRY_IS_POSTED, clé i18n 4 locales, enforce_immutability : inventaire fermé AC10, 6 lignes restantes), audit par acteur et clé d'API, rejeu, écran (Supprimer + confirmation, « Modifiée », « Historique » et droits par rôle), soldes de départ (AC7), manuels et PDF aplatis, api-external, Pattern 5, CHANGELOG, README.

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

Rapport complet dans `target/gate-logs/15-8b-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers `target/gate-logs/`.
