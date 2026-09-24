# Prompt — passe 3 de `bmad-code-review`, Story 25-3-a-1 — PASSE CIBLÉE

*Versionné le 2026-09-24. **Une** lentille en contexte frais (Opus), braquée sur la seule
remédiation de la passe 2 — cf. CLAUDE.md § « La passe ciblée ». La passe 2 n'a trouvé qu'un
MEDIUM, né de la remédiation de la passe 1 (un test) ; ce qui reste à relire est ce qu'on vient
d'écrire.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement`. **Périmètre :
`git show 3a939b8f`** — le test `une_cloture_concurrente_attend_l_annulation` et son aide
`attendre_un_verrou_sur_l_exercice` (`crates/kesh-db/tests/invoice_settlement.rs`). Lis aussi,
pour juger, le correctif qu'il prouve : l'étape 2-bis de `cancel_settlement_in_tx`
(`crates/kesh-db/src/repositories/invoice_settlements_write.rs`, commit `dd29630d`), et le socle
`reverse_in_tx_inner` (`crates/kesh-db/src/repositories/journal_entries.rs`).

## Axes

1. **Le test prouve-t-il ce qu'il dit ?** Déroule les deux mondes — avec et sans le verrou de
   l'étape 2-bis — requête par requête : où l'annulation attend-elle, que lit-elle, et pourquoi
   l'issue diffère-t-elle ? Existe-t-il un ordonnancement où le test passe **sans** le correctif
   (faux vert) ou échoue **avec** lui (faux rouge) ?
2. **La synchronisation est-elle sûre ?** `information_schema.PROCESSLIST` filtré par
   `DB = DATABASE()` et `ID <> CONNECTION_ID()` : le motif `INFO LIKE '%fiscal_years%' AND INFO
   LIKE '%FOR UPDATE%'` peut-il être satisfait par **autre chose** que la requête visée —
   la transaction de clôture elle-même (connexion du test), une connexion du pool, une requête
   de l'annulation qui ne **bloque** pas ? Une requête `FOR UPDATE` vue un instant sans être
   bloquée suffit-elle à déclencher la validation trop tôt, et est-ce grave ? Que montre `INFO`
   pour une instruction **préparée** (`sqlx` prépare ses requêtes) sous MariaDB 10.11 ?
3. **Fiabilité** : délai maximal de dix secondes, intervalle de dix millisecondes, pool partagé
   avec la tâche d'annulation — risque d'épuisement du pool, d'interblocage du test lui-même ?
4. **Le premier échec inexpliqué** de la passe 1 (consigné au Change Log) : vois-tu une
   explication plausible dans le code ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve**, correction.
- ⛔ **La liste des axes exercés ET non exercés.** Un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`,
tout `git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`,
`cargo nextest`, `npm run`. Lecture, `grep`, `git show`/`log`/`diff` et `cargo check` autorisés.
