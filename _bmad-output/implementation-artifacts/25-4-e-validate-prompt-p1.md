# Prompt — validation P1, Story 25-4-e (le montant minimum d'une facture)

*Versionné le 2026-10-01. **Une lentille** (Sonnet), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-e-montant-minimum-facture` (sur la 25-4-c4, faite). Fiche :
`_bmad-output/implementation-artifacts/25-4-e-montant-minimum-facture.md`. CR : `gh issue view 495`. Sœurs :
`25-4-c4-a-arrondi-fige-a-la-validation.md`, `25-4-c4-b-arrondi-visible-et-reglage.md`. Règles : `CLAUDE.md`
(§ *Migration breaking policy* P1–P8). Checklist : `.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages sont **retenus** : conteste la mise en œuvre, pas le principe. ⚠️ La fiche décrit du travail **à
faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **Le refus (AC 2)** : son emplacement dans `validate_invoice` (après le total arrondi nul, avant le compte
   d'arrondi) est-il le bon ? Existe-t-il **d'autres chemins qui émettent une facture** sans passer par
   `validate_invoice` (import, revalidation après dévalidation, seed, API externe) ? Les variantes de `DbError` à
   champs et leur mapping HTTP : patron existant (`PeriodLocked`) ? Où le frontend affiche-t-il l'erreur de
   validation (le message du serveur est-il montré tel quel) ?
3. **Le réglage (AC 4)** : tous les sites qui énumèrent les champs des réglages
   (`grep -rn "round_to_5_centimes\|roundTo5Centimes" crates/ frontend/src`) sont-ils couverts ? `double_option`
   convient-il à un `Decimal` ? La validation « strictement positif, deux décimales » a-t-elle un patron
   (`scale_within`) ?
4. **La migration (AC 1)** : non breaking ; P5/P6/P7/P8 ; squash ; export ; sauvegarde antérieure.
5. **Le manuel (AC 5)** : où `user-manual.tex` et `admin-manual.tex` décrivent-ils la validation et les
   paramètres de facturation (PDF aplati : `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`) ?
6. **Faisabilité des tests (AC 6)**, dont l'E2E (montage de la société `with-company`, effet de bord sur les
   autres specs si le seuil n'est pas remis à vide) ; **périmètre** recompté.

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie, ou code cité), correction. ⛔ La liste des axes
exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`,
`gh issue view`, `cargo check`, `pdftotext` vers le scratchpad.
