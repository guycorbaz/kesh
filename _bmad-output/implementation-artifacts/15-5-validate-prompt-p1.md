# Prompt — validation P1 de la spec, Story 15-5 (gardes de postabilité côté serveur)

*Versionné le 2026-10-08. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (décision D6
de la rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Fiche** :
`_bmad-output/implementation-artifacts/15-5-gardes-postabilite-serveur.md` (commit `428985c8`). Issues :
`gh issue view 427 --comments`, `gh issue view 429 --comments`, et pour la frontière `gh issue view 474`. Choix
autonomes C3 à C6 : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`.
Aucun code n'est encore écrit.

## Lentilles

- **A — Auditeur d'acceptation** : applique `.claude/skills/bmad-create-story/checklist.md` à la fiche. Chaque
  phrase des deux issues (et du commentaire de #429) est-elle couverte par un AC testable ? Chaque AC par une tâche,
  chaque tâche par un test ? La fiche est-elle implémentable sans deviner (noms de fonctions, fichiers, codes
  d'erreur, clés i18n dans les 4 locales, replis Svelte) ? Les choix C3–C6 sont-ils cohérents avec les AC ?
- **B — Chasseur de chemins non gardés** : refais **toi-même**, depuis le code, l'inventaire des sites non résolus —
  tout appel `journal_entries::create_in_tx` / `create_in_tx_inner` hors tests avec `enforce_postable = false`, et
  toute route (handler Axum) qui reçoit un identifiant de compte du client (`grep -rnE "account_id|accountId"`
  dans `crates/kesh-api/src/routes/`). Compare à l'inventaire de la fiche : un site **absent** de la fiche, mal
  classé, ou un numéro de ligne faux (`grep -nF` à l'appui) est un finding. Vérifie aussi les clés d'API (les routes
  atteintes par PAT) et les chemins par lot.
- **C — Régressions et bords** : quels tests existants changent de sens (fixtures `postable: true`, test qui fige
  `INACTIVE_OR_INVALID_ACCOUNTS`, frontend qui lit ce code — `grep -rn` dans `frontend/src`) ; l'exemption « si la
  valeur change » (C4) a-t-elle un trou (PATCH partiel, valeur `null`, même compte renvoyé, compte devenu non
  imputable après coup) ; interaction avec les **règles de rapprochement existantes** déjà en base dont le compte
  n'est pas imputable (que voit l'utilisateur ? une migration ou une donnée est-elle nécessaire ?) ; le **manuel**
  (`docs/manual/fr/user-manual.tex` et `admin-manual.tex`, et le **PDF aplati** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`) — tous les
  passages devenus faux sont-ils nommés ? CHANGELOG ; règle de découpage (§ « Règle de splitting préventif »).

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (de la fiche et du code), **preuve** (commande et
sortie, ou code cité relu), correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la
sortie d'un `grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`,
`cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, et aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`target/gate-logs/`.
