# Prompt — validation P1, Story 25-4-c2 (le verrou à l'acceptation)

*Versionné le 2026-09-30. **Une lentille** (Sonnet), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c2-verrou-acceptation` (empilée sur la 25-4-c,
PR #483). Fiche à valider : `_bmad-output/implementation-artifacts/25-4-c2-verrou-acceptation.md`.
Source des faits : `25-4-c-residuel-au-rapprochement.md` (Change Log, validation P3, findings F1–F3).
Issue : `gh issue view 480`. Règles : `CLAUDE.md`. Checklist : `.claude/skills/bmad-create-story/checklist.md`.

Le choix par défaut (rejeu inclus) est **retenu** : ne pas le contester, en contester la mise en œuvre.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **L'inventaire des écrivains** — pars du symptôme, pas de la liste : tout ce qui change le reste dû
   d'une facture client, c'est-à-dire `invoice_settlements`, les lignes de facture (`invoice_lines`),
   les avoirs (`credit_notes`, `credit_note_lines`), le statut (`invoices.status`).
   `grep -rn "invoice_settlements\|invoice_lines\|credit_note" crates/*/src --include=*.rs | grep -iE "insert|delete|update"`.
   Chacun incrémente-t-il `invoices.version` dans la même transaction ? Un écrivain oublié qui
   n'incrémente pas est au moins HIGH : il rouvre la course.
3. **Le raisonnement d'isolation** : sous REPEATABLE READ (MariaDB 10.11, `innodb_snapshot_isolation`
   OFF), le verrou optimiste `UPDATE … WHERE version = ?` ferme-t-il réellement la course n° 1 une
   fois l'invariant posé ? Cherche un contre-exemple : un ordre d'événements où l'acceptation lit un
   reste périmé et où son `UPDATE` réussit quand même. Et `due_after` / `fully_settled`, calculés sur
   l'instantané après l'insertion du règlement : peuvent-ils poser `paid_at` à tort quand le contrôle
   de version a réussi ?
4. **Le rejeu (AC 4)** : l'affirmation « le 1213 remonte en 1305 au `ROLLBACK TO SAVEPOINT` » est-elle
   exacte sur MariaDB 10.11 (lis la doc de MariaDB si tu peux ; sinon dis-le) ? Les autres chemins du
   lot (`accept_one_split`, `accept_one_rule`) sont-ils touchés par la même reconnaissance ? Rejouer
   un lot entier est-il sûr (effets hors transaction : audit, e-mails, fichiers) ? Le verrou
   `GET_LOCK` est-il bien relâché puis repris entre deux tentatives ?
5. **Faisabilité des tests (AC 5)** : le montage proposé (verrou `fiscal_years` tenu par le test,
   règlement concurrent daté dans un autre exercice) fonctionne-t-il vraiment ? L'acceptation
   atteint-elle `find_open_covering_date` **après** la garde de trop-perçu ? Un règlement manuel
   dans un autre exercice est-il permis par `settle_invoice` (date ≥ date de facture, exercice ouvert) ?
   Un interblocage déterministe est-il montable ? Mutations tuables ?
6. **Effets de bord de l'invariant** : qui lit `invoices.version` d'une facture validée (frontend,
   API, clés d'API, tests) et casserait si un règlement partiel l'incrémente ? La réponse de
   `POST /invoices/{id}/settlements` porte-t-elle la facture relue après validation ?
7. **Le manuel et les textes** : `docs/manual/fr/user-manual.tex` (règlement manuel, réconciliation)
   et `docs/api-external.md` — que promettent-ils sur la concurrence ou sur `version` ? PDF aplati
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`).
8. **Périmètre et cohérence** : AC ↔ tâches, modules recomptés ; la story laisse-t-elle un état pire
   qu'aujourd'hui sur un point quelconque ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact, **preuve** (commande exécutée et
  sa sortie, ou code lu cité), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un rapport sans elle
  ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`, toute requête d'écriture sur MariaDB.
Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers le
scratchpad, `cargo check`, requêtes `SELECT`/`SHOW` en lecture seule.
