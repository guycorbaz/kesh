# Prompt — revue de code P2 ciblée, Story 15-5e2

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation de la revue P1.*

Worktree `/home/gcorbaz/devel/kesh-15-5e2`, branche `story/15-5e2-rejeu-des-autres-flux`. **Objet** :
`git show ee77f450` (un seul commit). Rapports remédiés : `target/gate-logs/15-5e2-review-p1-{B,E,A}.md`. Choix
C-15-5e2-7.

## Lentille unique — chasseur de régressions de la remédiation

1. **`conclude_locked_attempt(tx_outer, lock_result, flow)`** (`crates/kesh-api/src/routes/reconciliation.rs`) : le
   `match` factorisé est-il strictement équivalent aux deux anciens (compare au diff ligne à ligne : chaque bras,
   chaque code d'erreur, chaque message, le `RELEASE_LOCK` et le `rollback` sur TOUS les chemins, l'ordre
   commit / release) ? Un 1213 atteint-il toujours le prédicat `is_app_deadlock` de l'enveloppe ?
2. **Lecture dans la tentative** de `was_previously_rejected` et du montant : la transaction bancaire est-elle relue
   SOUS VERROU avant d'en tirer ces valeurs ? Le test 8 étendu (écriture concurrente de `auto_match_rejected_at`
   pendant l'attente) prouve-t-il ce qu'il dit, et la mutation le fait-elle rougir ?
3. **Clones du pool retirés** dans cinq fermetures (`post_manual`, `post_split`, `complete_import`,
   `onboarding::finalize`, `reconciliation::cancel`) : la fermeture capture-t-elle encore ce dont elle a besoin à
   chaque tentative (rien de consommé par la première) ?
4. **Doc-comments** de `post_manual` / `post_split`, point (vii) du registre, paragraphe « Required » du Pattern 5 :
   exacts ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-5e2-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par
MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`switch`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`,
SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture.
