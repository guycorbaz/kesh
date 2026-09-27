# Prompt — validation P3 CIBLÉE, Story 25-4-b1 (le résiduel aux agrégats)

*Versionné le 2026-09-27. **Une lentille** (Opus), contexte frais. Passe **ciblée** : la P2 (Haiku)
a rendu « 0 finding » sans exercer l'axe qu'on lui confiait ; l'orchestrateur l'a repris et a
trouvé un chemin oublié. Ce qu'il reste à relire : la **portée de l'AC 5**, et elle seule, contre le
code.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-b-residuel-aux-agregats`. Fiche :
`_bmad-output/implementation-artifacts/25-4-b1-residuel-aux-agregats.md`, AC 5 (bloc « Portée de
l'invariant ») et AC 13 (réserve du manuel). Diff des remédiations : `git diff cfd2a779 58a874ab`.

## La lentille : inventaire des sites NON RÉSOLUS

L'AC 5 affirme : *total de la balance âgée = solde du compte débiteurs au grand livre*, sauf quatre
cas énumérés. ⛔ **Ne vérifie pas que les quatre cas sont vrais — cherche le cinquième.**

1. **Inventorie l'ensemble clos de ce qui écrit une ligne d'écriture** (`journal_entry_lines`) :
   pars du symptôme — `grep -rn "create_in_tx\|journal_entries::create\|INSERT INTO journal_entry_lines" crates/ --include=*.rs`
   hors tests. Pour **chaque** chemin, dis s'il peut imputer le compte débiteurs (le compte
   `default_receivable_account_id` ou le compte de la ligne de débit d'une vente), et, s'il le
   peut, s'il est (a) une facture, un règlement ou un avoir **visibles** de la balance âgée,
   (b) un des quatre cas de la portée, ou (c) **un cas oublié**.
2. Pense aux chemins **de clôture et de réouverture d'exercice**, à la **dévalidation** d'une
   facture, à l'**annulation d'un règlement**, à la **contre-passation** manuelle d'une écriture de
   vente ou de règlement depuis sa fiche, à l'**import** de pièces, au **rapprochement** annulé.
3. Pour chaque cas oublié : sévérité, preuve (code lu), et formulation à ajouter à la portée.
4. La réserve prescrite pour le manuel (AC 13, `:1580-1581`) couvre-t-elle alors tous les cas, en
   mots d'utilisateur ?

## Ce que tu rends

- **Findings** : sévérité, endroit, **preuve**, correction.
- ⛔ **La liste des chemins inventoriés** (fichier:fonction → classement a/b/c) — c'est elle qui
  prouve l'axe — **et des axes non exercés.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`cargo check`.
