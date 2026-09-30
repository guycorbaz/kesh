# Prompts — revue de code P1, Story 25-4-c2 (le verrou à l'acceptation)

*Versionné le 2026-09-30. Trois lentilles en parallèle, **Sonnet**, contexte frais. Diff : le commit
d'implémentation, `git diff 6810cd97 177135f2 -- . ':(exclude)_bmad-output'`, écrit dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/25-4-c2-p1.diff`.*

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

## Lentille 2 — Edge Case Hunter (`bmad-review-edge-case-hunter`)

Diff **et** lecture du dépôt (MariaDB 10.11, REPEATABLE READ, `innodb_snapshot_isolation` OFF).
- `accept_batch` : chaque chemin où la transaction peut être annulée sous le lot — l'erreur remonte-t-elle
  toujours en `TransactionAborted` ou en 1213 direct ? Et `RELEASE SAVEPOINT` après une proposition
  réussie, `SAVEPOINT` suivant, `commit` ? Un 1205 (attente dépassée) est-il pris à tort pour un
  interblocage ?
- `post_accept` / `accept_once` : le rejeu est-il sûr — effets hors transaction (audit, e-mails,
  fichiers, métriques), `GET_LOCK` relâché puis repris, corps réutilisé ? Les validations et le
  pré-vol faits une seule fois restent-ils valables à la tentative suivante ?
- `settle_invoice` : l'`UPDATE` inconditionnel (`paid_at` à `NULL` sur un partiel) peut-il effacer un
  `paid_at` légitime ? `rows_affected` non vérifié — une facture non `validated` ?
- Les tests de course : montages réellement déterministes ? `LOCK TABLES` sur une connexion du pool
  rendue au pool verrouillée si le test panique avant `UNLOCK TABLES` ? Le test d'interblocage : la
  victime est-elle garantie ? Un test qui passerait à vide ?
- Inventaire : tout autre écrivain du reste dû ou du statut d'une facture validée qui n'incrémente pas
  `version` (`grep -rn "UPDATE invoices\|invoice_settlements\|credit_note" crates/*/src`).

## Lentille 3 — Acceptance Auditor

Diff, fiche `_bmad-output/implementation-artifacts/25-4-c2-verrou-acceptation.md`, lecture du dépôt.
Chaque AC (1 à 7) contre le code ; « Ce qu'il ne faut pas faire » (aucun `FOR UPDATE`, pas d'élargissement
de `is_deadlock_error`, pas de classement par le texte, pas de `sleep`, pas de changement d'isolation) ;
le Dev Agent Record (affirme-t-il seulement ce qui a tourné ? décomptes recomptés depuis la source ?
la dérogation au montage de l'AC 5 est-elle justifiée ?). ⛔ **Le manuel** :
`docs/manual/fr/user-manual.tex` (règlement manuel, réconciliation) et `docs/api-external.md` —
l'entrée *Accepter des propositions de rapprochement* dit-elle vrai sur le code (codes, statuts,
champs) ? PDF aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Le
CHANGELOG dit-il vrai ?
