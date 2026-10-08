# Prompt — revue de code P2 ciblée, Story 15-8a

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation de la revue P1.*

Worktree `/home/gcorbaz/devel/kesh-15-8`, branche `story/15-8-modifier-une-ecriture`. **Objet** :
`git diff 93658b38 683caaa4` (un seul diff aplati ; 14 fichiers). Rapports remédiés :
`target/gate-logs/15-8a-review-p1-{B,E,A}.md`. Choix C-15-8a-9 à 11 au registre `epic-15-choix-autonomes.md`.

## Lentille unique — chasseur de régressions de la remédiation

1. **Le test `the_put_replays_a_deadlock_it_lost`** (`crates/kesh-api/tests/journal_entry_reversal_e2e.rs`) :
   prouve-t-il un rejeu ? La preuve invoquée (B obtient en exclusif le projet que le `PUT` tenait, ce qui n'est
   possible que si la transaction du `PUT` a été annulée) est-elle juste ? Le test peut-il passer sans 1213
   (ordre d'arrivée différent, `innodb_lock_wait_timeout`) ? Est-il instable (sleep, délais) ?
2. **Les tests A-1 / A-2** : mordent-ils ? (`updated_at` antidaté, I2 sur deux modifications.)
3. **Libellé E3** (`opening-balances-complete-date-today`, 4 locales + repli Svelte) : le texte correspond-il à
   `decide_date` (relis-la) ? Mêmes variables dans les 4 locales ?
4. **Documentation** : manuel utilisateur (`.tex` + PDF aplati : `pdftotext -nopgbrk docs/manual/fr/user-manual.pdf
   - | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`), `docs/api-external.md`, CHANGELOG : la garde
   `LATER_FISCAL_YEAR_CLOSED` est-elle décrite comme limitée à la modification, avec renvoi à #543 ? Les résolutions
   de conflit du rebase (C-15-8a-10 : encadré *postable* de la 15-5b) sont-elles cohérentes ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés.** Rapport complet dans
`target/gate-logs/15-8a-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers
`target/gate-logs/`.
