# Prompt — revue de code P1, Story 15-13a

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-13a`, branche `story/15-13a-mariadb-non-publiee`. **Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-15-13a diff 200f5e79..b9892834` (un seul diff aplati ; le code,
les tests, les manuels, la doc — les fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-13a-mariadb-non-publiee-mots-de-passe-obligatoires.md` (AC, tâches, Dev Agent Record, Change Log). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #551 (#577, #578 pour le contexte). Registre : `epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Choix C-15-13-1 à 29, C-15-13a-1, C-15-13a-2. Code et fichiers touchés : docker-compose.yml et docker-compose.prod.yml (port non publié, `${MARIADB_*_PASSWORD:?…}`), .env.example, config.rs/main.rs (avertissement kesh_dev et gabarit `<…>`, indice 1045, decode_utf8_lossy), tests (demarrage_mariadb.rs, configuration_transmise.rs, tests/common/binaire.rs), ci.yml (refus vérifié dans les deux sens) et docs/ci.md, manuel admin, brochure et PDF, DOCKER_START.md, init-demo.sh retiré, CHANGELOG. Axes : aucun mot de passe ni URL dans un journal (avertissements, indice 1045) ; l'étape CI peut-elle passer à vide ; la procédure de mise à jour du manuel et le tableau des cas (aucun cas qui pousse à écrire un mot de passe neuf dans .env) ; la recette ALTER USER (historique, ps) ; le texte sur les sous-commandes de Compose (mesuré sur 2.40 seulement : est-il écrit comme tel ?) ; manuel et brochure, PDF aplatis.

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

Rapport complet dans `target/gate-logs/15-13a-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers `target/gate-logs/`.
