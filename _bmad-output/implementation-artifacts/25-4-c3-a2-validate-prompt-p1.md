# Prompt — validation P1, Story 25-4-c3-a2 (le compte d'arrondi dans les plans livrés)

*Versionné le 2026-09-30. **Une lentille** (Sonnet), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c3-a2-compte-arrondi-plans` (empilée sur la c3-a1,
PR #487). Fiche : `_bmad-output/implementation-artifacts/25-4-c3-a2-compte-arrondi-plans.md`. Sœur :
`25-4-c3-a1-reglage-compte-arrondi.md`. Issue : `gh issue view 476` (commentaires : arbitrages). Règles :
`CLAUDE.md`. Checklist : `.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages de Guy (réglage et non rôle ; 6940, charge, dans les trois plans ; désigné d'office à la
création ; rien chez les sociétés existantes) sont **retenus** : en contester la mise en œuvre, pas le
principe. ⚠️ La fiche décrit du travail **à faire** : ne reproche pas au code de ne pas encore le porter.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **Tous les chemins qui créent une société et ses comptes** — pars du symptôme :
   `grep -rn "bulk_create_from_chart\|load_chart\|insert_with_defaults" crates/`. Onboarding (chemin
   démo, chemin production, reprise), seed de démonstration, endpoint de test `/_test/seed`, import d'une
   sauvegarde, `reset` démo : lequel doit désigner le compte, lequel en est exclu, et la fiche le dit-elle ?
3. **La désignation (AC 3)** : `insert_with_defaults*` peuvent-ils lire `companies.org_type` et appeler
   `load_chart` (dépendance `kesh-db` → `kesh-core`) ? Le plan d'une société peut-il différer de celui que
   donne son `org_type` (changement de forme juridique après l'étape 4, plan importé) — et que se passe-t-il
   alors ? `INSERT IGNORE` : la fiche dit « une société qui a déjà ses réglages n'est pas touchée » — est-ce
   vrai du code actuel (branche `rows == 0`) ? Le verrou `FOR UPDATE` des lookups de rôle s'étend-il au
   compte d'arrondi ?
4. **Le marqueur (AC 2)** : patron de `postable` (24-5) — `#[serde(default)]`, `validate_chart`,
   `is_postable` réutilisable ? Le nom JSON (`roundingDifference`) respecte-t-il la casse des autres champs
   (`parentNumber`) ?
5. **Tests existants touchés (AC 5)** : `grep -rn "load_chart\|bulk_create_from_chart" crates/*/tests crates/*/src`
   — quels tests comparent le plan semé à un état attendu, comptent les entrées, ou vérifient les réglages
   après onboarding (`accounts_role_backfill.rs`, `closing_accounts_backfill.rs`, tests d'onboarding de
   `kesh-api`) ? Lequel casserait, et la fiche le prévoit-elle ?
6. **Le manuel** : `docs/manual/fr/admin-manual.tex` (paragraphe *Compte de différences d'arrondi*) et
   `user-manual.tex` (contenu des plans livrés ?), **PDF aplati**
   (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`).
7. **Périmètre** : modules recomptés ; frontière avec la c3-b nette ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact, **preuve** (commande exécutée et sa
  sortie, ou code lu cité), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base
(`scripts/*`, `make`, `latexmk`, `git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`/`switch`,
`sqlx migrate`, `cargo test`/`nextest`, `npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`).
Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers le scratchpad,
`cargo check`.
