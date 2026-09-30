# Prompt — validation P2, Story 25-4-c2 (le verrou à l'acceptation)

*Versionné le 2026-09-30. **Une lentille** (Haiku 4.5), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c2-verrou-acceptation`. Fiche :
`_bmad-output/implementation-artifacts/25-4-c2-verrou-acceptation.md` — **lis le fichier dans son état
actuel**. Ce que la P1 a changé : `git show cc792871 -- _bmad-output/implementation-artifacts/25-4-c2-verrou-acceptation.md`
(contexte seulement ; les numéros de ligne font foi dans les fichiers courants, jamais dans le diff).
Règles : `CLAUDE.md`.

## Axes — tous obligatoires

1. **La remédiation P1** :
   - AC 4 : une erreur typée dédiée posée par `accept_batch` quand `ROLLBACK TO SAVEPOINT` échoue —
     est-ce réalisable dans `crates/kesh-reconciliation/src/errors.rs` (`ReconciliationError`) et le
     `match` de la route (`crates/kesh-api/src/routes/reconciliation.rs`, autour de `lock_result`) ?
     Ce `match` est-il exhaustif, et la nouvelle variante y aurait-elle un bras ? Le prédicat de
     `retry_with` voit-il un `AppError` ou un `ReconciliationError` ? `with_account_lock`
     (`crates/kesh-reconciliation/src/mutex.rs:66`) relâche-t-il `GET_LOCK` quand la closure échoue ?
   - Les deux inventaires : `credit_notes.rs:283` et `:586`, `invoices.rs:1453` et `:1577` — lis-les.
     Existe-t-il un autre écrivain du statut d'une facture validée ou de son avoir
     (`grep -rn "UPDATE invoices SET" crates/*/src`) qui n'incrémente pas `version` ?
   - AC 7 : la section `docs/api-external.md:287` et suivantes.
2. **Le contre-exemple** : cherche un ordre d'événements où, l'invariant posé, l'acceptation écrit un
   règlement fondé sur un reste périmé **et** réussit son `UPDATE invoices … version = ?`.
3. **Cohérence interne** : AC ↔ tâches, « Ce qu'il ne faut pas faire », Change Log.

## Ce que tu rends

- **Findings** : sévérité, endroit exact, **preuve** : la commande exécutée **et sa sortie copiée**, ou
  l'extrait de code lu avec son numéro de ligne. Pour toute affirmation qu'un élément est **absent**, le
  `grep -rnF` qui le prouve et sa sortie. Un finding sans preuve ne sera pas retenu ; un « 0 finding »
  sans preuve non plus.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `cargo check`.
