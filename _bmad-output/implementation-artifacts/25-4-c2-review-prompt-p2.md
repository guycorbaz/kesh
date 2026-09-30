# Prompt — revue de code P2 ciblée, Story 25-4-c2 (le verrou à l'acceptation)

*Versionné le 2026-09-30. **Passe ciblée** (CLAUDE.md § « La passe ciblée ») : une lentille (Haiku 4.5),
contexte frais, braquée sur la remédiation P1 seule — `git show b3a740a8`, écrit dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/25-4-c2-p2.diff`.
Dépôt `/home/gcorbaz/devel/kesh`, lecture seule. Fiche : `_bmad-output/implementation-artifacts/25-4-c2-verrou-acceptation.md`.*

## Axes — tous obligatoires

1. **Les gardes `rows_affected() != 1`** (`crates/kesh-db/src/repositories/invoice_settlements_write.rs`,
   dans `settle_invoice` et `cancel_settlement_in_tx`) : lis les deux fonctions en entier dans le fichier
   courant. Un chemin **légitime** peut-il rendre 0 ligne et désormais échouer à tort (facture `cancelled`
   par un avoir, facture dont le règlement s'annule après avoir été créditée, `WHERE` sans garde de
   statut dans l'annulation) ? Qui appelle `cancel_settlement_in_tx` (`grep -rn "cancel_settlement_in_tx" crates/`),
   et chacun garantit-il la ligne ? Comment `DbError::Invariant` est-il rendu au client
   (`crates/kesh-api/src/errors.rs`) ?
2. **`RELEASE SAVEPOINT`** dans `accept_batch` (`crates/kesh-api/src/routes/reconciliation.rs`) : la
   nouvelle branche est-elle correcte ; l'erreur d'origine est-elle préservée ?
3. **`transaction_aborted_outside_accept`** : les trois sites l'appellent-ils, et `drop(tx_outer)` est-il
   toujours fait **avant** ?
4. **Le test** : la connexion détachée (`.detach()`) — `&mut verrou` sur un `MySqlConnection`, les deux
   requêtes passent-elles bien par elle ? Une connexion détachée non fermée explicitement bloque-t-elle
   quoi que ce soit en fin de test ?
5. **`docs/api-external.md`** : la phrase ajoutée est-elle exacte sur le code (`500 INTERNAL_ERROR`,
   « rien n'a été écrit ») ?

## Ce que tu rends

- **Findings** : sévérité, endroit exact, **preuve** : la commande exécutée **et sa sortie copiée**, ou
  l'extrait de code lu avec son numéro de ligne dans le fichier courant. Pour toute affirmation qu'un
  élément est absent, le `grep -rnF` qui le prouve. Un finding sans preuve ne sera pas retenu ; un
  « 0 finding » sans preuve non plus.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `cargo check`.
