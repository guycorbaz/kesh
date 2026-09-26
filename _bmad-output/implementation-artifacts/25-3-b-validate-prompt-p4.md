# Prompt — passe 4 de `bmad-create-story validate`, Story 25-3-b — PASSE CIBLÉE

*Versionné le 2026-09-25. **Une** lentille en contexte frais (Sonnet — cycle Sonnet → Haiku → Opus
→ Sonnet), braquée sur la seule remédiation de la passe 3. Signal de découpage (MEDIUM → MEDIUM
entre P2 et P3) soumis à Guy : **pas de découpage**, passe ciblée.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-b-annuler-rapprochement`. Objet :
`_bmad-output/implementation-artifacts/25-3-b-annuler-rapprochement.md`. **Périmètre** : la
remédiation de la passe 3, aplatie dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/5bef2124-6ec8-41d8-9004-39b0653420a3/scratchpad/25-3-b-p3-remediation.diff`
(91 lignes) — son Change Log (ligne `validate P3`) dit pourquoi. Le commit suivant (`cd96d7a3`)
n'a fait que remplacer deux renvois par les issues #463 et #431.

⚠️ Ne conteste pas les arbitrages de Guy (Q1-Q3, Q5).

## Axes

1. **La signature du geste** (AC 1) gagne `actor_api_key_id: Option<i64>`. Tous les endroits de la
   fiche qui citent la signature, l'appel ou l'audit sont-ils cohérents (AC 4, AC 6, pseudo-code,
   Tasks, Dev Notes) ? `NewAuditLogEntry::for_actor` : signature exacte (lis `kesh-db`) ? La limite
   assumée (les lignes d'audit de la sœur et du socle sans la clé) est-elle **vraie** ligne par
   ligne (`invoice_settlements_write.rs:466`, `journal_entries.rs:1659`) ?
2. **Les codes hors motifs** (AC 9) : la liste est-elle **close** ? Refais-la depuis la route telle
   que la fiche la décrit (verrou de compte, geste, socle, commit, rejeu) et depuis
   `crates/kesh-api/src/errors.rs` : `PERIOD_LOCKED`, `OPTIMISTIC_LOCK_CONFLICT`,
   `RECONCILIATION_ACCOUNT_LOCKED`, `RECONCILIATION_LOCK_RELEASE_FAILED`, `NOT_FOUND` — en
   manque-t-il (403, 500, `VALIDATION_ERROR`…) ? Les cinq numéros de ligne cités sont-ils exacts ?
   Chacun de ces codes porte-t-il bien un `message` **traduit** côté serveur, que le dialogue peut
   afficher ?
3. **Le pseudo-code du rejeu** (AC 6) : juste contre `retry.rs`, `mutex.rs:66-75` (`AsyncFnOnce`),
   `ReconciliationError::Db(#[from] DbError)`, `onboarding.rs:614-620` ?
4. **La limite du verrou nommé** : fidèle à `mutex.rs` (bras d'erreur, `tracing`), et au L22 cité ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), endroit exact, **preuve** (code lu,
  commande et résultat), correction.
- ⛔ **La liste des axes exercés ET non exercés.** Un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`,
tout `git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`,
`cargo nextest`, `npm run`. Lecture, `grep`, `git show`/`log`/`diff` et `cargo check` autorisés.
