# Prompt — revue de code P1, Story 15-1a-i

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-1a-i`, branche `story/15-1a-i-marque-du-lettrage`. **Le diff à revoir** :
`git -C /home/gcorbaz/devel/kesh-15-1a-i diff dc4bc58b..09a9d16b -- . ':!_bmad-output'` (un seul diff aplati ; le
code, la migration, les tests, les manuels, la doc — les fiches `_bmad-output/` sont le contexte, pas l'objet).
**Fiche** : `_bmad-output/implementation-artifacts/15-1a-i-marque-du-lettrage.md` (AC, tâches, R1–R7, Dev Agent
Record, Change Log) ; sœur à venir `15-1a-ii-gardes-du-lettrage.md` (ce qu'elle prescrit n'est PAS un manque de la
15-1a-i). Issue (par `gh api repos/guycorbaz/kesh/issues/518`) : #518. Registre : `epic-15-choix-autonomes.md`
(C124–C128, C-15-1a-i-1 à 5). Règles : `CLAUDE.md`, en particulier § « Migration breaking policy » (P1–P8 : la
migration `20261009000001` relève `kesh_version_min_required` à 0.13.0 et les dix crates passent à 0.13.0 dans le
même commit ; `EXEMPT_MIGRATIONS`, squash `test-schema`, `docs/migrations-idempotence-audit.md` et ses compteurs).

Code touché : schéma du lettrage (clé de lettrage sur `journal_entry_lines`), `kesh-core::lettering` (codes,
fonctions pures par cause), `kesh-db/src/repositories/letterings.rs` (création et dissolution d'un groupe, modes
`Manual`/`System`, verrous des exercices triés par `start_date`, `check_rows_affected`), routes
`kesh-api/src/routes/letterings.rs` (POST, GET, DELETE ; enveloppe de rejeu ; registres d'audit et de routes),
exposition et export, i18n, manuels et PDF, api-external, CHANGELOG.

Axes : l'ensemble clos des écrivains de `journal_entry_lines` qui doivent respecter la marque (un chemin qui
modifie ou supprime une ligne lettrée sans passer par la garde, en tenant compte de ce que la 15-1a-ii prend en
charge) ; l'ordre des verrous et les cycles avec la clôture (15-12a/b), la modification et la suppression
d'écriture (15-8a/b), les règlements ; la création concurrente de deux groupes sur les mêmes lignes ; le plafond
`i64::MAX` et l'arithmétique des codes ; l'audit avant/après ; l'import d'une sauvegarde antérieure (rejeu,
`post_restore`) ; le bump de version et ses tests runtime ; les tests qui passeraient à vide ; le **manuel**.

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Défauts de correction, régressions, erreurs de concurrence
  (verrous, REPEATABLE READ, ordre d'acquisition), erreurs rendues au client, tests qui passeraient à vide, code mort,
  duplication (DRY), doc-comments devenus faux.
- **E — Edge Case Hunter** : chaque branche et chaque borne du code modifié — lignes vides, en doublon, d'une autre
  société, de comptes non lettrables, d'exercices clos ou de période verrouillée, groupe déséquilibré, clé maximale ;
  chemins par clé d'API ; locales ; et les **chemins NON modifiés qui devraient l'être** (inventorier par `grep` les
  sites non résolus de la même famille).
- **A — Acceptance Auditor** : chaque AC de la fiche contre le code ET les tests (un AC sans test qui le prouve est un
  finding) ; le Dev Agent Record ne déclare-t-il que ce qui a tourné (chiffres recomptés :
  `grep -c '#\[sqlx::test\]\|#\[test\]\|#\[tokio::test\]'` aux deux bornes) ; la Migration breaking policy point par
  point ; le **manuel** (`docs/manual/fr/*.tex` **et PDF aplatis** : `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' '`,
  ligatures ﬀ/ﬁ/ﬂ normalisées), `docs/api-external.md`, CHANGELOG, i18n 4 locales.

## Ce que tu rends

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a-i-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans
une base, ni aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`),
`make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
