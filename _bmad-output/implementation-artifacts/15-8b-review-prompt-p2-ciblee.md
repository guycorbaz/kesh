# Prompt — revue de code P2 ciblée, Story 15-8b

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation de la revue P1.*

Worktree `/home/gcorbaz/devel/kesh-15-8`, branche `story/15-8b-supprimer-une-ecriture`. **Objet** :
`git show 8cfb3759` (un seul commit, rebasé sur `ef39dd54`). Rapports remédiés :
`target/gate-logs/15-8b-review-p1-{B,E,A}.md`. Choix C-15-8b-7 et C-15-8b-8.

## Lentille unique — chasseur de régressions de la remédiation

1. **Clé `error-fiscal-year-closed-generic`** (« ajoutée, modifiée ou supprimée ») : même texte et mêmes variables
   dans les 4 locales (`grep -n "error-fiscal-year-closed-generic" crates/kesh-i18n/locales/*/messages.ftl`) et dans
   son repli Rust (`crates/kesh-api/src/errors.rs`) ? Le message reste-t-il juste pour TOUS les chemins qui le rendent
   (création, modification, suppression, et d'autres éventuels : `grep -rn "FiscalYearClosed\b" crates/*/src`) ?
2. **Doc-comment de `delete_by_id`** (`crates/kesh-db/src/repositories/journal_entries.rs`) : rattaché et exact ?
3. **Test Playwright Consultation** (« Modifiée » visible) : `modifierLibelle` modifie-t-elle bien l'écriture par
   l'API avec le bon contexte authentifié (rôle autorisé à modifier), et l'assertion porte-t-elle sur l'écran
   Consultation ? Le test pourrait-il passer à vide ?
4. **Test API A3** (`period_lock_closed_year_and_later_year_refuse_the_delete`, borne la veille → 204) : mord-il ?
5. **Commentaires** (`journal-entries/+page.svelte`, `i18n-keys.test.ts`) : exacts.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-8b-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par
MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture.
