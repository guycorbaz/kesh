# Prompt — revue de code P2 ciblée, Story 15-7a2

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P1 — qui ne touche que des tests et de la
documentation.*

Worktree `/home/gcorbaz/devel/kesh-15-7a2`, branche `story/15-7a2-trace-installation-production`. **Objet** :
`git show 23b3e46e` (diff unique ; ouvre `crates/kesh-api/tests/onboarding_audit_e2e.rs` à `HEAD` autour des
hunks). Rapports remédiés : `target/gate-logs/15-7a2-review-p1-{B,E,A}.md`. Choix C-15-7a2-4.

## Lentille unique — chasseur de régressions de la remédiation

1. **Test 14** `demo_installation_is_refused_by_every_production_step` : chaque route est-elle appelée à son
   étape EXACTE (sinon le 400 viendrait de la garde d'étape et non de `require_not_demo`, et la mutation ne
   serait pas distinguée) ? Le contrôle « même requête, `is_demo` levé → 200 » est-il fait pour chacune ? La
   liste de six routes est-elle exacte contre `crates/kesh-api/src/routes/onboarding.rs`
   (`grep -n "require_not_demo\|lock_state_at_step" `) ?
2. **Tests 9 (c) et 9 (d)** : le déclencheur fait-il échouer la BONNE écriture (`account.chart_loaded` pour (c),
   l'entrée d'étape pour (d)) ? Les assertions (« 0 compte », « état inchangé ») pourraient-elles être vraies
   par construction, comme le test 9 d'origine ?
3. **Test 13 étendu** (pool d'une connexion, neuf routes) : chaque route atteint-elle vraiment son code
   transactionnel, ou s'arrête-t-elle avant (refus d'étape) — auquel cas le test ne prouve rien pour elle ?
4. **Test 1** : les `details` comparés « en entier aux lignes en base » le sont-ils vraiment (toutes les clés),
   ou le test relit-il ce qu'il a écrit ?
5. **Aucune ligne de production touchée** : `git diff -U0 a298d19f 23b3e46e -- 'crates/*/src'` doit être vide.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-7a2-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`.
