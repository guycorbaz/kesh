# Prompt — revue de code P2 ciblée, Story 15-11b

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P1.*

Worktree `/home/gcorbaz/devel/kesh-15-11b`, branche `story/15-11b-lecture-unique-des-variables`. **Objet** :
`git show 387a6aa0` (diff unique ; ouvre les fichiers à `HEAD` autour des hunks). Rapports remédiés :
`target/gate-logs/15-11b-review-p1-{B,E,A}.md`. Choix C-15-11b-3. Journal des mutations :
`target/gate-logs/15-11b-review-p1-mutations.txt`.

## Lentille unique — chasseur de régressions de la remédiation

1. **Tests neufs** (`from_env_jwt_secret_blank_edges_do_not_count`, `from_env_test_mode_trimmed_true_is_accepted`
   dans `crates/kesh-api/src/config.rs`, `rust_log_vide_vaut_info` dans
   `crates/kesh-api/tests/configuration_transmise.rs`) : prouvent-ils ce qu'ils disent ? Le test qui lance le binaire
   peut-il passer à vide (binaire introuvable, `.env` lu malgré tout, sortie non capturée, avertissement attendu émis
   pour une autre raison) ? Le témoin `RUST_LOG=error` discrimine-t-il vraiment ?
2. **Normalisation `r#`** et **dédoublonnage des exclusions** dans le test lexical : un jeton légitime peut-il être
   mal normalisé (faux rouge) ou une lecture échapper (faux vert) ? Le garde `exclusions >= 1` reste-t-il juste ?
3. **Doc-comment de `LogConfig::from_raw`** et **angles morts** écrits : exacts contre le code ?
4. **`.env.example`, manuel (`.tex` ET PDF aplati : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   vers `target/gate-logs/`), CHANGELOG** : la formulation « sous Docker » de `KESH_LOG_FILE_PATH` est-elle juste
   contre les trois compose (`${…-…}` vs `${…:-…}`) ?
5. **Aucune ligne de production exécutable touchée** : vérifie par `git diff -U0 6f82b763 387a6aa0 -- crates/*/src`.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-11b-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `pdftotext`.
