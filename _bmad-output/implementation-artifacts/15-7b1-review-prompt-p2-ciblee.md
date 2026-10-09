# Prompt — revue de code P2 ciblée, Story 15-7b1

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P1 — tests et commentaires seulement.*

Worktree `/home/gcorbaz/devel/kesh-15-7b1`, branche `story/15-7b1-trace-demonstration`. **Objet** :
`git show 4303ac01` (diff unique ; ouvre `crates/kesh-api/tests/onboarding_audit_e2e.rs` à `HEAD` autour des
hunks). Rapports remédiés : `target/gate-logs/15-7b1-review-p1-{B,E,A}.md`. Choix C-15-7b1-2, C-15-7b1-3.

## Lentille unique — chasseur de régressions de la remédiation

1. **Tests de l'étape sous verrou** (`seed_demo_refuses_step_3_under_lock`, `…_step_4_…`) : la garde exercée
   est-elle bien celle de la dernière transaction (`kesh-seed/src/lib.rs`, revérification sous verrou) et non la
   pré-vérification du handler ? Les assertions « rien d'écrit » sont-elles toutes vraies seulement si la garde
   refuse ?
2. **Test de course HTTP** (`seed_demo_race_with_start_production_is_a_400`) : le déclencheur
   `AFTER INSERT ON fiscal_years` pose-t-il l'étape 3 au bon moment, et uniquement dans la base de ce test
   (base éphémère `#[sqlx::test]`, pas une base partagée) ? Le déclencheur est-il supprimé ou confiné ?
3. **Tests du rejeu** (`is_seed_retryable_accepts_1213_and_only_it`,
   `seed_demo_last_transaction_is_replayed_on_deadlock`) : la 1213 levée par `SIGNAL` est-elle reconnue par le
   même chemin qu'une vraie (code d'erreur, type `DbError::Sqlx`) ? Le compteur MyISAM garantit-il UNE seule
   levée (MyISAM n'est pas transactionnel : c'est voulu, vérifie que c'est écrit) ? Le test passerait-il à vide
   si le rejeu n'avait pas lieu ?
4. **Garde de source 11 (b)** normalisée (blancs, casse) : un faux rouge possible sur un autre texte ?
5. **Aucune ligne de production exécutable touchée** : `git diff -U0 99335b53 4303ac01 -- 'crates/*/src'`, hors
   lignes `//` et `///`, doit être vide.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-7b1-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `docker`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`reset`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`,
SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`grep`.
