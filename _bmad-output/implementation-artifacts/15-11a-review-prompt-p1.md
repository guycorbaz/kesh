# Prompt — revue de code P1, Story 15-11a

*Versionné le 2026-10-08. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-11a`, branche `story/15-11a-compose-transmet-la-configuration`. **Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-15-11a diff 1bd554910127d7ba4e4462946c20ffaae9842e19..edd45a6c` (un seul diff aplati ; le code,
les tests, les manuels, la doc — les fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-11a-compose-transmet-la-configuration.md` (AC, tâches, Dev Agent Record, Change Log). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #550, #557, #558, #534. Registre : `epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Diff de code 1bd554910127d7ba4e4462946c20ffaae9842e19..edd45a6c (main ef39dd54 + planification). SÉCURITÉ d'abord : la garde de l'AC16 dans config.rs (is_template_placeholder, GENERATE_ME, forme <…>, ordre avant la longueur, KESH_JWT_SECRET et KESH_ADMIN_PASSWORD) — une valeur réelle peut-elle être refusée à tort (faux positif bloquant le démarrage d'une installation saine) ? une valeur de gabarit passer ? Puis : les 38 clés des deux compose (forme ${X:-}, KESH_LOG_FILE_PATH ${X-…}, KESH_ADMIN_PASSWORD sans défaut), image: dans docker-compose.yml, montages de prod inchangés, le test configuration_transmise.rs (mord-il ? liste LUES exacte ? tri par octets), yaml-rust2, l'étape config -q de la CI, le manuel admin (.tex + PDF aplati, ligatures normalisées : tableaux de sec:env-vars lisibles, sous-section « Passer à la 0.13.0 »), DOCKER_START.md, READMEs, CHANGELOG (Corrigé #550, Sécurité #557, action pour les installations existantes), brochure.

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

Rapport complet dans `target/gate-logs/15-11a-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers `target/gate-logs/`.
