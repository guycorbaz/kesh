# Prompt — revue de code P2 ciblée, Story 15-5e1

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation de la revue P1.*

Worktree `/home/gcorbaz/devel/kesh-15-5e1`, branche `story/15-5e1-socle-rejeu`. **Objet** : `git show 1dc41152`
(un seul commit). Rapports remédiés : `target/gate-logs/15-5e1-review-p1-{B,E,A}.md`. Choix C-15-5e1-4.

## Lentille unique — chasseur de régressions de la remédiation

1. **`retry_with` épuisé** (`crates/kesh-db/src/retry.rs`) : le `tracing::error!` est-il émis une seule fois, au
   bon moment (après la dernière tentative, pas après une erreur métier), avec `operation` et `attempts` exacts ?
   La sémantique de retour de `retry_with` est-elle inchangée (même erreur rendue) ? Les appelants
   (`kesh_api::retry`, les sites directs) sont-ils intacts ?
2. **Témoin** (`crates/kesh-api/tests/common/capture_rejeu.rs`) : la liste `epuisements` compte-t-elle les seuls
   `ERROR` de la cible `kesh_db::retry` ? Les volets (e) et (f) du test 1 mordent-ils (une mutation les ferait-elle
   rougir) ?
3. **Connexion détachée** (`rejeu_interblocage_e2e.rs`) : la connexion qui pose `innodb_lock_wait_timeout = 1`
   est-elle vraiment détachée (`PoolConnection::detach` ou équivalent) puis fermée, sur tous les chemins (y compris
   un `panic!`/`assert!` qui échoue avant) ? Aucune autre connexion de test ne garde un réglage de session ?
4. **Doc-comment de `supplier_invoices::create_in_tx`** : les étiquettes `(1)`, `(2)`, `(2, suite)`, `(3)`, `(4)`…
   correspondent-elles exactement aux commentaires du code (`grep -nF "// (" crates/kesh-db/src/repositories/supplier_invoices.rs`) ?
5. **Pattern 5** (`docs/MULTI-TENANT-SCOPING-PATTERNS.md`) : le paragraphe réécrit et l'exemple sont-ils exacts
   (`retry_with` prend bien le nom d'opération en premier paramètre, signature relue au code) ?
6. **CHANGELOG** et AC6 (« trois documents faux entre les deux merges ») cohérents.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-5e1-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par
MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, lecture des sources
dans `~/.cargo/registry/src/`.
