# Prompt — revue de code P2 ciblée, Story 15-7b3

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte frais,
en lecture seule, braquée sur le seul commit de code de la remédiation P1.*

Worktree `/home/gcorbaz/devel/kesh-15-7b3`, branche `story/15-7b3-reparation-des-installations-atteintes`. **Objet** :
`git show d38aeb45` (diff unique ; ouvre les fichiers à `HEAD` autour des hunks). Rapports remédiés :
`/home/gcorbaz/devel/kesh-gate-logs/15-7b3-review-p1-{B,E,A}.md` (P1 Sonnet ×3 : 3 MEDIUM). Choix C-15-7b3-4 et 5
(`_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`). Mutations : `/home/gcorbaz/devel/kesh-gate-logs/157b3-mutations.log`.

⚠️ Tu relis du CODE : un défaut est ce que le code fait de faux, ou un test qui ne prouve pas ce qu'il dit. Avant
d'affirmer qu'une chose est absente du code, copie la sortie d'un `grep -nF` qui le montre. Ce qu'une autre story
prescrit n'est pas un manque.

## Lentille unique — chasseur de régressions de la remédiation P1

1. **Garde de liste vide dans `company_referencing_columns`** (`crates/kesh-db/src/repositories/companies.rs`) : rend-elle
   bien une erreur (et non une liste vide traitée comme « rien ne désigne ») ? Le test sur une base vide choisie par
   `USE` revient-il ensuite à la bonne base, sans polluer les tests suivants ?
2. **Aides de test déplacées** vers `crates/kesh-db/tests/support/installations_atteintes.rs` (inclusion par `#[path]`
   dans trois tests d'intégration, et sous `#[cfg(test)]` dans `bootstrap.rs`) : plus aucune référence à l'ancien
   emplacement (`grep -rn "test_fixtures::rendre_principaux_orphelins\|test_fixtures::poser_declencheur_en_echec" crates`) ?
   `test_fixtures.rs` identique à `e892dcfa` (`git diff e892dcfa d38aeb45 -- crates/kesh-db/src/test_fixtures.rs`) ?
3. **Tests neufs** `repair_on_restore_keeps_superfluous_stubs` et `full_import_is_undone_when_the_repair_fails` :
   prouvent-ils ce qu'ils disent (trois sociétés gardées à la restauration ; import annulé en entier — base d'avant
   intacte, aucune entrée `installation.repaired` ni `admin.full_import`) ? Le déclencheur posé est-il retiré à la fin
   (sinon il pollue les tests suivants) ?
4. **Manuel** : `docker compose restart kesh-api` dans le Dépannage, sans casser les recettes `up -d` voisines des
   15-7b2/15-11a (rechargement de `.env`) ; « le journal d'audit excepté ».

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve**, correction proposée. ⛔ **Liste des axes
exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/15-7b3-review-p2-ciblee.md` ; dernier message : chemin, bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`, `sqlx`,
`cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`.
