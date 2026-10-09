# Prompt — revue de code P2 ciblée, Story 15-13b

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P1.*

Worktree `/home/gcorbaz/devel/kesh-15-13b`, branche `story/15-13b-sauvegarde-persistante`. **Objet** :
`git show 7b1db902` (diff unique ; ouvre `crates/kesh-api/src/routes/admin.rs` à `HEAD` autour des hunks).
Rapports remédiés : `target/gate-logs/15-13b-review-p1-{B,E,A}.md`.

## Lentille unique — chasseur de régressions de la remédiation

1. **La seule ligne de production touchée** : la chaîne du `warn!` de `write_backup_file` (continuation `\`).
   Le message rendu est-il exactement celui voulu (un espace entre « sauvegarde » et « pré-import », aucun
   autre) ? Le format `{e}` est-il intact ?
2. **`aucun_blanc_parasite_dans_le_code`** : sur quel périmètre lit-il le code (un seul fichier ? tout le
   crate ?) ; peut-il rougir à tort sur un alignement légitime, une chaîne littérale, un commentaire ou un
   tableau ? Peut-il passer à vide (assertion de montage) ? Un test de style aussi large est-il justifié pour
   un seul message — ou faut-il le restreindre aux littéraux de journal ?
3. **`echecs_avant_sauvegarde_tous_convertis`** : la lecture lexicale (« avant `write_pre_import_backup` »)
   est-elle robuste à rustfmt et aux commentaires ? Compte-t-elle bien trois constructions directes et une
   conversion ? Peut-elle passer à vide ?
4. **`write_pre_import_backup_dossiers_crees_et_existants`** : l'umask du processus de test peut-il fausser les
   modes attendus (`0700`, `0755`) ? Le dossier temporaire est-il nettoyé ?
5. **Doc-comments** neufs (angles morts de `write_backup_file`, conversion nommée sur `AdminFullImportFailed`,
   `check_schema_compat`, `build_keshbackup`) : exacts contre le code ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-13b-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `docker`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`reset`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`,
SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`grep`.
