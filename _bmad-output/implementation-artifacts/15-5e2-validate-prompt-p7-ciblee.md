# Prompt — validation P7 ciblée de la spec, Story 15-5e2

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P6.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Objet** : `git show b5ac76b4`
(lis ce diff ; ouvre la fiche `_bmad-output/implementation-artifacts/15-5e2-rejeu-des-autres-flux.md` autour des
hunks). Rapports remédiés : `target/gate-logs/15-5e2-p6-R.md` et `15-5e2-p6-F.md`. Choix C86.

## Lentille unique — chasseur de régressions de la remédiation

1. **Test 8** (`manual_match_is_replayed_when_it_is_the_deadlock_victim`) : relis `post_manual` au code
   (`crates/kesh-api/src/routes/reconciliation.rs`) et vérifie l'ordre réel des verrous (exercice, puis sentinelle
   `companies` via `projects.rs`), le rôle du verrou nommé (`GET_LOCK`) — le cycle décrit peut-il se former, ou le
   verrou nommé sérialise-t-il avant ? —, et que le critère « angle mort si le cycle ne se forme pas en T0 » est
   écrit. Le harnais cité existe-t-il (`grep -nF "fn victime" crates/kesh-api/tests/rejeu_interblocage_e2e.rs`,
   `exiger_un_rejeu`) ?
2. **`ENVELOPPES` / `RETRY_WITH_AUTORISE` / volet (c bis)** : cohérence entre eux et avec le registre livré
   (`crates/kesh-api/tests/audit_route_registry.rs`) ; la clause « échoue si `post_accept` n'appelle plus
   `retry_with` » ; la mutation sur `finalize`.
3. **Recompte de l'inventaire** (385 lignes / 62 fichiers) : rejoue la commande de la fiche et compare.
4. **Cohérence** AC2 / Dev Notes / T0 / T1 / T4, et la phrase de l'AC4 sur les routes ouvertes aux clés API.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-5e2-p7-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture.
