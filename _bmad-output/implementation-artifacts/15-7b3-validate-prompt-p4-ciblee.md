# Prompt — validation P4 ciblée de la spec, Story 15-7b3

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P3.*

Worktree `/home/gcorbaz/devel/kesh-15-7`, branche `story/15-7-trace-onboarding`. **Objet** : `git show 2bc8dc32`
(lis ce diff ; ouvre la fiche `_bmad-output/implementation-artifacts/15-7b3-reparation-des-installations-atteintes.md`
autour des hunks). Rapports remédiés : `target/gate-logs/15-7b3-p3-R.md` et `15-7b3-p3-F.md`. Choix C-15-7-53.

## Lentille unique — chasseur de régressions de la remédiation

1. **Mutation 9, variante `bank_profiles`** : fait-elle vraiment rougir le test 6 (b) ? Vérifie au squash
   (`crates/kesh-db/test-schema/`) que `bank_profiles.company_id` est en `ON DELETE CASCADE` et relis le montage
   du test 6 (b). La note « garde / SAVEPOINT » des Dev Notes est-elle exacte (25 RESTRICT, 4 CASCADE — recompte
   au squash avec `grep -n "REFERENCES .companies." -A1`) ?
2. **Erreurs sous `SAVEPOINT`** : la règle « erreur d'origine rendue, jamais le 1305 » est-elle cohérente avec
   l'AC 2 (non bloquant au démarrage) et l'AC 3 (restauration : l'erreur annule l'import) ? La forme d'émission
   citée (`reconciliation.rs:1026-1072`) existe-t-elle (`sed -n`) ?
3. **Manuel** : la cellule Dépannage, la ligne Rollback, la limite au § Reprises. Le grep de contrôle du nom de
   service prescrit par l'AC 6 rendrait-il zéro ligne après correction ? Relève aujourd'hui, toi-même, toutes les
   commandes `docker compose … kesh` et `container kesh` de `docs/manual/fr/admin-manual.tex` qui ne visent pas
   `kesh-api` ou `kesh-mariadb` (formes avec options : `logs --tail=100 kesh`), et compare à la liste de l'AC 6.
4. **Propagation et cohérence** : « Cas laissés » bornés, angle mort reformulé, dépendance à la 15-11, montage
   `log_bin` aligné sur la 15-7b2, mutation 14, recompte (6 AC, 7 tâches, 6 tests, 14 mutations, 5 modules).

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés.** Rapport complet dans
`target/gate-logs/15-7b3-p4-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture.
