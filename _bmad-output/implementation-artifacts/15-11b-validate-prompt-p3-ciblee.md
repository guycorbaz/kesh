# Prompt — validation P3 ciblée de la spec, Story 15-11b

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P2.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Objet** : `git show 390ccfd1`
(lis ce diff ; ouvre la fiche `_bmad-output/implementation-artifacts/15-11b-lecture-unique-des-variables.md` autour
des hunks). Rapports remédiés : `target/gate-logs/15-11b-p2-R.md` et `15-11b-p2-F.md`. Choix C80.

## Lentille unique — chasseur de régressions de la remédiation

1. **Jetons surveillés ajoutés** (`EnvFilter`, `Builder`, `with_env_var`, `from_env_lossy`, `try_from_env`, `init`,
   `try_init`) : les 5 entrées neuves de la table pour `crates/kesh-api/src/logging.rs` sont-elles exactes ? Recompte
   toi-même chaque jeton surveillé dans `logging.rs` et dans tout `crates/kesh-api/src` (`grep -nwE
   "EnvFilter|Builder|init|try_init|with_env_var|from_env_lossy|try_from_env" crates/kesh-api/src -r`) et compare au
   total « 22 entrées ». Un `init` ou un `Builder` d'un AUTRE usage (autre crate, autre type) existe-t-il déjà dans le
   code de production et ferait-il rougir le test sans entrée prévue ?
2. **Règle de fenêtre amendée** (un groupe `{…}`/`[…]` termine la fenêtre) : les fenêtres de la table restent-elles
   justes ? Le cas `-> EnvFilter { … }` est-il couvert par un cas (S) ?
3. **Chemins qualifiés** : `kesh_api::config::env_nonempty` dans `main.rs` (vérifie que `main.rs` est bien le crate
   binaire et que `config` est `pub` dans `lib.rs`), `crate::config::env_nonempty` dans `logging.rs` et
   `routes/onboarding.rs`.
4. **Mutations M6, M8, M12, M16, M17** : chacune compile-t-elle telle qu'écrite, et fait-elle rougir la règle
   annoncée ? Comptes 17 / 16 / 1.
5. **Table avant/après** (`KESH_LOG_FILE_*` déjà trimés en aval : vérifie `from_raw`, `LogRotation::parse`,
   `LogFormat::parse`, `parse_max_files`) et § Angles morts (`NO_COLOR`, `TOKIO_WORKER_THREADS`, `TZ`).

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-11b-p3-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, lecture des sources
dans `~/.cargo/registry/src/`.
