# Prompt — passe 3 de `bmad-create-story validate`, Story 25-3-b — PASSE CIBLÉE

*Versionné le 2026-09-25. **Une** lentille en contexte frais (Opus), braquée sur la seule
remédiation de la passe 2 — cf. CLAUDE.md § « La passe ciblée ». La boucle converge (P1 : 2 HIGH /
3 MEDIUM → P2 : 2 MEDIUM) ; ce qui reste à relire est ce qu'on vient d'écrire.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-b-annuler-rapprochement`. Objet :
`_bmad-output/implementation-artifacts/25-3-b-annuler-rapprochement.md`. **Périmètre** : la
remédiation de la passe 2, aplatie dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/5bef2124-6ec8-41d8-9004-39b0653420a3/scratchpad/25-3-b-p2-remediation.diff`
(72 lignes) — son Change Log (ligne `validate P2`) dit pourquoi.

⚠️ Ne conteste pas les arbitrages de Guy (Q1-Q3, Q5).

## Axes

1. **L'emboîtement du rejeu (AC 6)** : le pseudo-code est-il **juste** contre le code réel —
   `kesh_db::retry::retry_with` (signature, contrainte `Fn`, backoff), `with_account_lock`
   (`crates/kesh-reconciliation/src/mutex.rs:66` : il prend `&mut Transaction`, relâche-t-il le
   verrou sur **toute** sortie, y compris quand la closure rend une erreur 1213 ?), et le mapping
   de `post_manual` (`routes/reconciliation.rs:3006-3080`) ? Une tentative rejouée peut-elle
   trouver un état **partiel** de la précédente ? Le verrou nommé, pris sur la connexion de la
   transaction avortée, est-il vraiment relâché avant la tentative suivante ? `kesh-db` est-il
   dans les dépendances de `kesh-api` pour `retry` (oui/non, preuve) ?
2. **« Préserver `DbError::Sqlx` »** : dans le chemin **réel** du geste (`cancel_in_tx` →
   `cancel_settlement_in_tx` / `reverse_in_tx` → `map_db_error`), un 1213 ressort-il bien en
   `DbError::Sqlx` — ou `map_db_error` le transforme-t-il en une autre variante, ce qui rendrait le
   rejeu muet **quel que soit** le mapping de la route ? Lis `map_db_error`
   (`crates/kesh-db/src/errors.rs`).
3. **« Qui affiche quoi » (AC 9)** : cohérent avec la table de l'AC 3, l'AC 10 (le dialogue lit
   `code` et `details`), les Dev Notes « Erreurs » ? Reste-t-il une phrase qui dit l'inverse ?
4. **`.entry.id`** : `JournalEntryWithLines` a-t-il bien un champ `entry` portant `id` ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), endroit exact, **preuve** (code lu,
  commande et résultat), correction.
- ⛔ **La liste des axes exercés ET non exercés.** Un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`,
tout `git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`,
`cargo nextest`, `npm run`. Lecture, `grep`, `git show`/`log`/`diff` et `cargo check` autorisés.
