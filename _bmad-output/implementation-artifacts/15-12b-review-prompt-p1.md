# Prompt — revue de code P1, Story 15-12b

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-12b`, branche `story/15-12b-filet-sous-un-bilan-clos`. **Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-15-12b diff 012fc430..adde3bc4` (un seul diff aplati ; le code,
les tests, les manuels, la doc — les fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-12b-filet-sous-un-bilan-clos.md` (AC, tâches, Dev Agent Record, Change Log). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #543 (#568, #569 pour le contexte). Registre : `epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Choix C100 à C123, C-15-12b-1 à 3. Code touché : filet LATER_FISCAL_YEAR_CLOSED aux deux points de passage du journal (create_in_tx_inner, delete_in_tx — condition enforce_ownership levée), lot de rapprochement à trois voies (project_error_to_failed_proposal et la branche propre d'accept_one_rule), message élargi à la saisie (4 locales, repli Rust), bandeau et réparation à l'écran des exercices, prédicteurs (angles morts C120 #568 et C-15-12b-2), manuels et PDF, api-external, CHANGELOG. Axes : la lecture NON verrouillante de find_later_closed dans create_in_tx_inner et sa preuve (invariant I sous la 15-12a) — une course la met-elle en défaut ? ; l'ensemble clos des écrivains d'écritures : en manque-t-il un hors des deux points de passage ? ; l'ordre des refus dans chaque flux (dévalidation, annulations, contre-passation, avoir) contre le code et api-external ; le patron FailedProposal dans le lot ; les tests prouvent-ils ce qu'ils disent ; les recomptes (11 lignes / 6 / 5, neuf refus) ; le manuel et le PDF aplati.

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

Rapport complet dans `target/gate-logs/15-12b-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers `target/gate-logs/`.
