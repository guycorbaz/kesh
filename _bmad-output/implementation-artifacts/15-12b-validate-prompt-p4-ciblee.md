# Prompt — validation P4 ciblée de la spec, Story 15-12b

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation P3 de la fiche 15-12b. Aucune règle du filet n'a changé en P3.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Objet** :
`git show c7a5dbcc -- _bmad-output/implementation-artifacts/15-12b-filet-sous-un-bilan-clos.md` (lis ce diff ;
ouvre la fiche autour des hunks). Rapports remédiés : `target/gate-logs/15-12-p3-R.md` et `15-12-p3-F.md`.
Choix C117, C120 à C123. Code de référence : `origin/main` (`git show origin/main:<chemin>`), `5e4bec50`.

## Lentille unique — chasseur de régressions de la remédiation

1. **Inventaire de l'AC 21 part B** refait par le symptôme (72 lignes, 44 lignes / 42 sites qui posent un exercice
   clos, un seul qui change de sens : `journal_entries.rs:3452-3453`) : rejoue la commande de la fiche et recompte ;
   prends trois sites au hasard dans la table et vérifie leur classement au code.
2. **C120 (angle mort de `settlement_entry_cancel_blocker`)** : le doc-comment, `api-external.md` (`:319`, `:353`,
   `:361`) et le test sur le patron C-15-8-29 sont-ils prescrits de façon implémentable et cohérente ? L'issue #568
   est-elle citée ?
3. **AC 11 (trois voies du lot, `projectId` omis pour la règle, C123)**, **AC 12**, **AC 18** : cohérence entre eux,
   avec les tâches et avec le code (`grep -nF` sur `accept_one_rule`, `project_error_to_failed_proposal`).
4. **C117** (précédence 2-bis avant 3-ter-bis, la story mergée en second l'écrit) et la **dérogation** (10 modules,
   C121) : recompte des modules ; la section est-elle conforme à l'exception du CLAUDE.md ?
5. **CHANGELOG** : `Corrigé` vs `Modifié` (C123) bien prescrits.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-12b-p4-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture.
