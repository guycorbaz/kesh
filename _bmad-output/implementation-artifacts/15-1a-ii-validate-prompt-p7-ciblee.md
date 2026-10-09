# Prompt — validation P7 ciblée de la spec, Story 15-1a-ii

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation P6. Aucune règle n'a changé en P6.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Objet** :
`git show a8e7d3ff -- _bmad-output/implementation-artifacts/15-1a-ii-gardes-du-lettrage.md` (lis ce diff ; ouvre
la fiche autour des hunks). Rapports remédiés : `target/gate-logs/15-1a-ii-p6-R.md` et `15-1a-ii-p6-F.md`.
Code de référence : `origin/main` (`git show origin/main:<chemin>`).

⚠️ **La story n'est pas développée** : tu relis une SPÉCIFICATION. Un test, un texte ou une constante que la fiche
prescrit et qui n'existe pas encore dans le code n'est PAS un défaut. Un défaut est une contradiction, une
prescription inexécutable, un fait cité faux, ou un site oublié.

## Lentille unique — chasseur de régressions de la remédiation

1. **Grep élargi d'AC8** : rejoue toi-même
   `git grep -nE "sans cycle|cycle connu|par uniformité|No known cycle|by uniformity|known cycle" origin/main -- crates frontend/src docs README.md CHANGELOG.md website`
   et compare au tri écrit dans la fiche (8 lignes : six inscrites, deux étrangères). Une ligne manque-t-elle ?
   Le tri des deux « étrangères » (`audit_route_registry.rs:95`, `opening_complement.rs:26`) est-il juste au code ?
2. **R6-2** : `journal-entries.api.ts:42-45` existe-t-il à ces lignes avec « ne modifie rien » / « demeure » ? Le
   tableau de R6 compte-t-il bien cinq endroits, et T5 les reprend-il ?
3. **F6-1** : le second cas du test (c) (origine sous `books_locked_through`, 201 attendu) est-il cohérent avec la
   règle des périodes de la 15-1a-i (C113 : lettrer exige au moins une ligne en période ouverte) ? Le miroir du
   jour l'est-il, et la fiche le dit-elle ?
4. **R6-4, F6-2, F6-3** : numéros cités (`:3202`, `:3219`, `:4856`, `:4891`, `user-manual.tex:506`,
   `journal-entries.types.ts:96`) exacts sur `origin/main` ?
5. **Recomptes** du Change Log (5 AC, 7 tâches, 13 tests, 9 mutations, 11 modules).

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-1a-ii-p7-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`grep`.
