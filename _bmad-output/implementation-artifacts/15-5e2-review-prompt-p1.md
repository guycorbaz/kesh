# Prompt — revue de code P1, Story 15-5e2

*Versionné le 2026-10-08. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-5e2`, branche `story/15-5e2-rejeu-des-autres-flux`. **Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-15-5e2 diff 688fed25dea8fcc50ae2b2e40761a07777229843..6125d50b` (un seul diff aplati ; le code,
les tests, les manuels, la doc — les fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-5e2-rejeu-des-autres-flux.md` (AC, tâches, Dev Agent Record, Change Log). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #536, #484, #555. Registre : `epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Rollout (C61) : douze routes rejouées, six sites retry_with migrés vers les enveloppes par équivalence exacte, registre sans ARejouer (22/0/4/89), RETRY_WITH_AUTORISE et volet (c bis), commentaires d'ordre, Pattern 5, api-external § 10, CHANGELOG, manuels. Lentille E : revue FICHIER PAR FICHIER des douze routes (règle du rollout) — pour chacune : toute écriture est-elle dans la tentative ? aucun effet de bord hors transaction (e-mail, fichier, audit hors tx) ? entrées clonées par tentative ? lectures préalables sans verrou hors de la tentative (C-15-5e2-3) justes ? Priorité aux trois extractions complete_import_once, post_manual_once, post_split_once (post_split_once reconstruit lignes, audit et libellé à chaque tentative : rien de partagé ni de consommé entre deux tentatives ?) et aux six équivalences (même nombre de tentatives, même prédicat, même nom d'opération).

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

Rapport complet dans `target/gate-logs/15-5e2-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers `target/gate-logs/`.
