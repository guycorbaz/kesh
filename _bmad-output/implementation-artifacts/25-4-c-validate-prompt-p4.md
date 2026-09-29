# Prompt — validation P4, Story 25-4-c (le résiduel au rapprochement) — périmètre réduit

*Versionné le 2026-09-29. **Une lentille** (Sonnet), contexte frais. La story vient d'être **découpée**
(règle de splitting, sévérité P2 → P3 non décroissante) : le verrou part en 25-4-c2 (#480), l'arrondi
en 25-4-c3 (#476). Cette passe valide le périmètre réduit, en entier.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c-residuel-au-rapprochement`. Fiche :
`_bmad-output/implementation-artifacts/25-4-c-residuel-au-rapprochement.md` (état courant). Le
découpage : `git show 5d743613`. Issues : `gh issue view 420`, `gh issue view 476`, `gh issue view 480`.
Mère : `25-4-propager-le-residuel.md` ; sœurs `25-4-a`, `25-4-b1`, `25-4-b2`. Règles : `CLAUDE.md`.
Checklist : `.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages (Q1 retiré, Q2, Q3) et le découpage sont **retenus** : ne pas les contester, en contester
la mise en œuvre.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme — y compris celles
   ajoutées au découpage (Dev Notes « Montage du Playwright », manuel `:1011`, `:1014`, `:1224`).
2. **La frontière du découpage** : la fiche réduite est-elle réalisable **sans** rien de la c2 ni de
   la c3 ? Relis l'inventaire § 5 et § 6. Le filtre sur le reste **brut** avec la tolérance ± 0.05 :
   l'argument |brut − arrondi| ≤ 0.005 tient-il ? La story laisse-t-elle un état **pire**
   qu'aujourd'hui sur un point quelconque (course acceptation / règlement manuel, reste à 4 décimales
   affiché, score) ? Reste-t-il dans la fiche une phrase, une tâche ou un test qui appartient à la c2 ou
   à la c3 ?
3. **Inventaire des sites** — pars du symptôme :
   `grep -rn "total_ttc\|invoice_amount\|invoiceAmount\|find_unpaid_invoices_for_window\|propose_matches" crates/ frontend/src`.
   Tout lecteur du champ candidat ou du montant affiché hors de la fiche : couvert, légitime, oublié ?
4. **AC 1 et AC 3** : le candidat porte `amount_due` et `total_ttc` — la requête jointe peut-elle
   rendre les deux (GROUP BY / HAVING, `INVOICE_TTC_SUBQUERY_SQL` corrélé encore utilisé ?) sans
   retomber dans la forme corrélée interdite ? Le champ TTC ajouté à la réponse : type frontend
   (`reconciliation.types.ts`), sérialisation camelCase, `docs/api-external.md`, clé i18n et
   `lint-i18n-ownership`.
5. **Les tests (AC 5)** : chacun **aurait-il échoué avant le patch** ? Le test d'acceptation (hors
   score de référence et de contact) : lis `matching.rs` et vérifie qu'aucun autre sous-score ne le fait
   passer sur le code actuel. Le Playwright : la recette des Dev Notes est-elle exécutable (lis les
   helpers cités) ? Mutations tuables ?
6. **Le manuel (AC 6)** : lignes citées, **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`).
   L'affirmation sur `:1014` (versement « reste + frais » non proposé, refusé en trop-perçu seulement
   avec la référence) est-elle exacte sur le code ?
7. **Cohérence interne** : numérotation des AC après renumérotation (ex-7 → 5, ex-8 → 6) partout —
   tâches, Dev Notes, Change Log ; modules recomptés ; `sprint-status.yaml` (lignes 25-4-c, c2, c3)
   cohérent avec la fiche.

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact, **preuve** (commande exécutée et
  sa sortie, ou code lu cité), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un rapport sans elle
  ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers le scratchpad, `cargo check`.
