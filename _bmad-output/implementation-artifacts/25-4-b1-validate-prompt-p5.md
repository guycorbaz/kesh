# Prompt — validation P5 CIBLÉE, Story 25-4-b1

*Versionné le 2026-09-27. **Une lentille** (Haiku), contexte frais. Passe ciblée sur la seule
remédiation de la P4.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-b-residuel-aux-agregats`. Diff **unique** à
relire : `git diff 5480b4f8 5824c048` (deux fiches : `25-4-b1-residuel-aux-agregats.md`,
`25-4-a-residuel-juste.md`).

## Ce que tu vérifies

1. **T0** liste cinq sites : chacun existe-t-il à la ligne dite, avec la formulation dite ? Commande
   obligatoire, à recopier dans ton rapport : `grep -rnF "sauvegarde antérieure" crates/ _bmad-output/implementation-artifacts/25-4-a-residuel-juste.md`
   et `grep -rnF "keshbackup" crates/`.
2. **La phrase ajoutée à la règle** (contre-passations dans la portée) : juste ? Lis
   `invoice_settlements_write.rs` (`cancel_settlement_in_tx`) avant de répondre.
3. **La réserve du manuel reformulée** : chacun de ses cas correspond-il à un cas réel de la règle ?
   En manque-t-il un par rapport à la règle ?
4. **La note datée dans la fiche 25-4-a** : exacte (`git show v0.12.0:crates/kesh-db/src/repositories/credit_notes.rs | grep -n "paid_at.is_some"`) ?

⛔ Une fiche `ready-for-dev` décrit du code **à écrire** : ne signale pas que le code ne le fait pas
encore. ⛔ Pour chaque finding, recopie la **sortie** de la commande qui le prouve.

## Ce que tu rends

Findings (sévérité, endroit, preuve recopiée, correction) et **la liste des axes exercés et non
exercés**.

## Interdits

⛔ Aucune écriture dans le dépôt ; aucune commande mutante — `scripts/*.sh`, `make`, `latexmk`,
`git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git show`/`diff`,
`gh issue view`.
