# Prompt — validation P1, Story 25-4-c3-a1 (le réglage du compte d'arrondi)

*Versionné le 2026-09-30. **Une lentille** (Sonnet), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c3-arrondi-centime`. Fiche à valider :
`_bmad-output/implementation-artifacts/25-4-c3-a1-reglage-compte-arrondi.md`. Issue : `gh issue view 476`
(lire les commentaires : arbitrages de Guy). Registre : `sprint-status.yaml`, lignes `25-4-c3*`. Règles :
`CLAUDE.md` (§ *Migration breaking policy* P1–P8 en particulier). Checklist :
`.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages de Guy (réglage et non rôle ; compte créable dans le plan et choisi dans les paramètres ;
découpage a1 / a2 / b) sont **retenus** : ne pas les contester, en contester la mise en œuvre.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **Inventaire des sites qui énumèrent les colonnes de `company_invoice_settings`** — pars du symptôme :
   `grep -rn "default_vat_decompte_account_id\|defaultVatDecompteAccountId\|company_invoice_settings" crates/ frontend/src docs --include=*.rs --include=*.ts --include=*.svelte --include=*.sql --include=*.md --include=*.tex`.
   Chaque site qui énumère les champs (projection SQL, `FromRow`, JSON d'audit, export, tests, types
   frontend, fixtures E2E, seed, onboarding) doit apparaître dans la fiche ou être explicitement sans
   objet. Un site oublié qui casse au **runtime** (`ColumnNotFound`) est au moins HIGH.
3. **La migration** : est-elle réellement non breaking au sens de P1 (un binaire antérieur lit-il une
   table avec une colonne en plus — `SELECT *` quelque part ?) ; P5 (compteurs de l'audit) ; P6 (sites
   positionnels) ; le squash et `test_schema_guard` ; y a-t-il un manifeste de sommes de contrôle des
   migrations (`find crates -name "*.sha384"`) ; `migrations_upgrade_path.rs` a-t-il un `assert_eq!(total, N)` ?
4. **La sauvegarde (AC 6)** : lis `crates/kesh-db/src/backup.rs` et l'import. Un `.keshbackup` antérieur,
   sans la colonne, s'importe-t-il (insertion par nom de colonnes ? `parse_and_verify` compare-t-il les
   colonnes ?). Et un `.keshbackup` **postérieur** importé par un binaire antérieur ?
5. **La validation (AC 3)** : `validate_account` — l'étendre sans changer le comportement des champs
   existants est-il faisable ? Les comptes TVA sont-ils contrôlés `postable` ailleurs ? Un compte de
   groupe (`postable = false`) peut-il aujourd'hui être choisi comme compte TVA ?
6. **Les gardes voisines (AC 5)** : que se passe-t-il aujourd'hui quand on archive ou supprime un compte
   désigné comme compte TVA dans les réglages (route d'archivage des comptes, message d'erreur de la FK) ?
7. **L'écran (AC 4)** : faisabilité dans `settings/invoicing/+page.svelte`, type frontend, garde i18n
   (compteur exact), E2E existant de cet écran (`frontend/tests/e2e/`).
8. **Le manuel** : `docs/manual/fr/admin-manual.tex` autour de `:1994`, et **PDF aplati**
   (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`).
   `docs/api-external.md` décrit-il les réglages de facturation ?
9. **Périmètre** : modules recomptés depuis les tâches ; frontière avec la c3-a2 et la c3-b nette ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact, **preuve** (commande exécutée et sa
  sortie, ou code lu cité), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un rapport sans elle ne
  compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers le scratchpad, `cargo check`.
