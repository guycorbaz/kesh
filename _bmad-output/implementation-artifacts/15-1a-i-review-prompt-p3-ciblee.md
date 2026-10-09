# Prompt — revue de code P3 ciblée, Story 15-1a-i

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de code de la remédiation P2.*

Worktree `/home/gcorbaz/devel/kesh-15-1a-i`, branche `story/15-1a-i-marque-du-lettrage`. **Objet** :
`git show 8cda7041` (diff unique ; ouvre les fichiers à `HEAD` autour des hunks). Rapports remédiés :
`/home/gcorbaz/devel/kesh-gate-logs/15-1a-i-review-p2-{B,E,A}.md`. Choix C-15-1a-i-11 à 13
(`_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`). Trend : P1 3 MEDIUM → P2 2 MEDIUM, tous deux
nés de la P1.

⚠️ Tu relis du CODE : un défaut est ce que le code fait de faux, ou un test qui ne prouve pas ce qu'il dit. Ne
signale pas comme manquant ce que la fiche prescrit à une AUTRE story (15-1a-ii, 15-1a2). Avant d'affirmer qu'une
chose est absente du code, copie la sortie d'un `grep -nF` qui le montre.

## Lentille unique — chasseur de régressions de la remédiation P2

1. **`build_group` rend `Result`** (`crates/kesh-db/src/repositories/letterings.rs`) : les trois appelants
   propagent-ils l'erreur sans la transformer (un `?` qui change le type, un `map_err` qui perd l'`Invariant`) ? Une
   transaction ouverte est-elle laissée dans un état incohérent quand l'erreur remonte ? L'ordre des verrous a-t-il
   changé ?
2. **`neutraliser_echappements`** (`crates/kesh-db/tests/letterings_lexical.rs`) : remplacer « `\` + caractère
   suivant » par deux espaces peut-il casser un mot légitime et rendre le détecteur aveugle à une forme qu'il voyait
   avant (ex. `\\UPDATE`, une chaîne brute `r"…\…"`, un échappement `\u{…}` de plusieurs caractères) ? Les cinq
   littéraux neufs `N` à `R` prouvent-ils ce qu'ils disent ?
3. **Contrôle négatif (2) de `lettering_invariants`** (A2-3) : atteint-il vraiment les deux `Invariant` (compte puis
   exercice), ou passerait-il si `find_group` rendait une autre erreur ? L'état est-il bien rétabli ensuite (un test
   qui laisse la base modifiée pollue les suivants) ?
4. **Doc-comments** qui renvoient à `MAX_LINES_PER_GROUP` : exacts ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve**, correction proposée. ⛔ **Liste des axes
exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/15-1a-i-review-p3-ciblee.md` ; dernier message : chemin, bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `git commit`/`add`/`checkout`/`stash`/`fetch`, `sqlx`, `cargo`,
`npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`.
