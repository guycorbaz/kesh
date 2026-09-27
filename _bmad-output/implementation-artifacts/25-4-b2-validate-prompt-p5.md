# Prompt — validation P5 CIBLÉE, Story 25-4-b2 (le résiduel aux rappels)

*Versionné le 2026-09-27. **Une lentille** (Haiku), contexte frais, **passe ciblée** (§ *La passe
ciblée* du `CLAUDE.md`) : elle ne relit pas la story, elle relit **la dernière remédiation**.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-b2-residuel-aux-rappels`. Fiche :
`_bmad-output/implementation-artifacts/25-4-b2-residuel-aux-rappels.md`. **Périmètre : le seul commit
`d06b5c6c`** (`git show d06b5c6c`) — la remédiation de la validation P4, dont le rapport est résumé au
Change Log (entrée « Validation P4 ciblée »). Arbitrages de Guy : **ne pas les contester**.

## Axes — tous obligatoires

1. **Chaque chemin et chaque `fichier:ligne` ajouté ou modifié par `d06b5c6c`** existe et dit ce que
   la fiche affirme — vérifie avec `sed -n '<ligne>p' <chemin>` et cite la sortie.
2. **Toutes les citations de fichier NU restantes de la fiche** (un nom de fichier sans chemin) :
   liste-les (`grep -noE '\(`[a-z_]+\.(rs|ts|svelte)[:0-9, -]*`' <fiche>`), et pour chacune
   `find crates frontend -name <nom>` — un nom présent dans plusieurs dossiers est ambigu. Dis
   lesquels restent ambigus (MEDIUM si la ligne citée n'existe que dans l'un des fichiers).
3. **Contradictions introduites** par `d06b5c6c` avec le reste de la fiche.
4. Les liens Markdown `[#NNN]` de la fiche ont-ils tous une définition, et chaque définition est-elle
   invoquée ?

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
