# Prompt — validation P4 CIBLÉE, Story 25-4-b2 (le résiduel aux rappels)

*Versionné le 2026-09-27. **Une lentille** (Sonnet), contexte frais, **passe ciblée** (§ *La passe
ciblée* du `CLAUDE.md`) : elle ne relit pas la story, elle relit **la dernière remédiation**.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-b2-residuel-aux-rappels`. Fiche :
`_bmad-output/implementation-artifacts/25-4-b2-residuel-aux-rappels.md`. **Périmètre : le seul commit
`6197bffd`** (`git show 6197bffd`) — la remédiation de la validation P3, dont le rapport est résumé au
Change Log (entrée « Validation P3 »). Arbitrages de Guy : **ne pas les contester**.

## Axes — tous obligatoires

1. **Chaque phrase ajoutée ou modifiée par `6197bffd`** : chaque `fichier:ligne` cité existe et dit ce
   que la fiche affirme (`sed -n`, `grep -nF`). Cite la sortie.
2. **Contradictions introduites** : une correction de P3 contredit-elle un autre passage de la fiche
   (AC, tâches, Dev Notes, « Où regarder », tests de l'AC 10) ? En particulier : le retrait de la
   ligne « avoir » a-t-il laissé des résidus (« quatre lignes », `amount_credited`, « avoir » dans un
   AC ou un test) ? Le choix Rust-par-`Language` pour `{feeNotice}` contredit-il une mention de clés
   Fluent ailleurs ? Le nom `rappel-{n}.pdf` est-il cohérent avec tous les tests cités ?
3. **Justesse des corrections** : la mention des frais sur une ligne à elle (AC 8) tient-elle dans la
   réserve et dans la largeur (`pdf.rs`, `col_desc`, largeur utile de la page) ? Le variant ajouté à
   `classify_render_error` : la forme proposée est-elle compatible avec le `match` actuel
   (`invoice_email.rs:937-950`) et avec le test `:1557` ? Le variant `AppError::ReminderNothingDue`
   et sa clé : le patron `DunningPaused` (`errors.rs`) est-il bien celui décrit ?
4. **Ce que la remédiation n'a pas traité** : les dix findings de P3 (M1-M5, L1-L5) sont-ils tous
   traités ou explicitement renvoyés (M5 est en attente d'arbitrage, c'est voulu) ?
5. **#476** (`gh issue view 476`) dit-il ce que l'AC 7 lui fait dire ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact, **preuve** (commande et
  résultat), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » sans
  cette liste et sans commandes citées ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`, `gh issue create/edit/comment`. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`, `gh issue view`, `cargo check`.
