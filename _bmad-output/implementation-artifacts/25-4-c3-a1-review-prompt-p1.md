# Prompts — revue de code P1, Story 25-4-c3-a1 (le réglage du compte d'arrondi)

*Versionné le 2026-09-30. Trois lentilles en parallèle, **Sonnet**, contexte frais. Diff : le commit
d'implémentation, `git show 55d98555 (hors `_bmad-output`, PDF et squash)`, écrit dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/c3a1-p1.diff`.*

## Commun aux trois lentilles

**Rendu** : findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact (`fichier:ligne`),
**preuve** (code lu cité, commande et sortie), correction proposée. ⛔ **La liste des axes réellement
exercés ET de ceux qui ne l'ont pas été** — un rapport sans elle ne compte pas.

**Interdits** : n'écrire aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une
base — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers le scratchpad, `cargo check`.

## Lentille 1 — Blind Hunter (`bmad-review-adversarial-general`)

Reçoit **le diff seul**, aucun contexte projet. Revue adversariale générale.
## Lentille 1 — Blind Hunter (`bmad-review-adversarial-general`)

Reçoit **le diff seul**, aucun contexte projet. Revue adversariale générale.

## Lentille 2 — Edge Case Hunter (`bmad-review-edge-case-hunter`)

Diff **et** lecture du dépôt.
- **Tous les sites qui énumèrent les colonnes** de `company_invoice_settings` : `grep -rn "company_invoice_settings\|default_payable_account_id\|defaultPayableAccountId" crates/ frontend/src` —
  une projection oubliée casse au runtime (`ColumnNotFound`), une fixture frontend oubliée casse le type.
- `PUT /company/invoice-settings` : sémantique « absent préservé / `null` effacé » (`double_option`) —
  interaction avec le verrou optimiste, le court-circuit no-op, l'audit ; le déplacement de
  `double_option` vers `crate::helpers` change-t-il quoi que ce soit pour `reconciliation_rules` ?
- `validate_account_of` : types acceptés, `postable`, société, archivé ; les champs historiques gardent-ils
  exactement leur comportement ?
- Migration : P1 (non breaking), P5 (compteurs de `docs/migrations-idempotence-audit.md` — recompte-les),
  P6 (`migrations_upgrade_path.rs` : `total` et soustracteur, frontière), P8 (`migrations.sha384`), squash.
- Sauvegarde : l'import d'un backup antérieur (test `strip_column`) — est-il réellement discriminant ?
  Un backup **postérieur** vers un binaire antérieur ?
- Frontend : sélecteur, `withCurrentAccount`, envoi ; le test Vitest et le Playwright prouvent-ils ce qu'ils
  annoncent ? Le Playwright laisse-t-il la base propre s'il échoue en route ?

## Lentille 3 — Acceptance Auditor

Diff, fiche `_bmad-output/implementation-artifacts/25-4-c3-a1-reglage-compte-arrondi.md`, lecture du dépôt.
Chaque AC (1 à 8) contre le code ; « Ce qu'il ne faut pas faire » (pas de rôle, pas de donnée écrite par la
migration, pas de numéro dans le code, pas d'écriture lisant le réglage) ; le Dev Agent Record (affirme-t-il
seulement ce qui a tourné ? décomptes recomptés ?). ⛔ **Le manuel** : `docs/manual/fr/admin-manual.tex`
(paragraphe *Compte de différences d'arrondi*) et son **PDF aplati**
(`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`) — dit-il vrai sur le code, et
sur ce qui n'est pas encore livré ? Le CHANGELOG dit-il vrai (sémantique de l'API) ?
