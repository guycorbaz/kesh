# Prompt — validation P2 de la spec, Story 15-5b

*Versionné le 2026-10-08. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : P1 Sonnet ×3 → P2 Opus ×2 ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Fiche** :
`_bmad-output/implementation-artifacts/15-5b-gardes-surfaces-neuves.md`. Elle est née du **découpage** de la 15-5 en passe P1 :
fiche mère `15-5-gardes-postabilite-serveur.md` (Change Log de P1), fiche sœur `15-5a-refus-non-imputable.md`. La remédiation de
P1 est le commit `8746e26b` (`git show 8746e26b`). Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et
`/comments`) : 427, 429, 492, 519. Choix autonomes C3–C14 : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`.
Règles : `CLAUDE.md`. Aucun code n'est encore écrit.

## Lentilles

- **R — Regression hunter** : la remédiation de P1 a réécrit et découpé la fiche. A-t-elle introduit des défauts ?
  Contradictions entre AC, tâches, Dev Notes et choix C* ; frontière entre 15-5a et 15-5b (un site traité deux
  fois, ou par aucune des deux ; une dépendance non dite) ; faits recopiés sans vérification (numéros de ligne,
  noms de fonctions, codes d'erreur, clés i18n) — revérifie-les au code (`grep -nF`). Recompte les AC et les
  tâches. La fiche est-elle implémentable sans deviner ?
- **F — Full-scope adversary** : la conception elle-même, et ce que P1 a affirmé sans l'exécuter. Les chemins de
  code qui écrivent avec un compte choisi par le client ; les transactions et l'ordre des contrôles (exemption
  « inchangé », verrous, REPEATABLE READ) ; les tests qui changeront de sens ; le frontend qui lit les codes
  d'erreur (`grep -rn` dans `frontend/src`) ; i18n dans les 4 locales et replis Svelte ; le **manuel**
  (`docs/manual/fr/*.tex` **et le PDF aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`
  vers `target/gate-logs/`) ; CHANGELOG ; la règle de découpage (§ « Règle de splitting préventif »).

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (fiche et code), **preuve** (commande et sortie,
ou code cité relu), correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la sortie
d'un `grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext`
vers `target/gate-logs/`.
