# Prompt de revue de code P4 ciblée — story 37-3a

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), une seule lentille, contexte frais, braquée sur la remédiation P3 :
worktree `<dépôt>/.claude/worktrees/kesh-s37-3a` (branche `story/37-3a-ecritures-en-devise-modele`),
`git show f5ab5716`. N'écris nulle part ailleurs, sauf journaux dans `<station>/kesh-gate-logs/s37-3a/review-p4/`.

À lire : la fiche 37-3a (AC, entrées « revue de code P1 », « P2 » et « P3 ciblée » du Change Log).

Correctif de la P3 à relire : `somme_bornee` (checked_add) dans `validate` de `crates/kesh-core/src/accounting/balance.rs`,
nouvelle variante `CoreError::EntryTotalOutOfRange`, bras de la route `journal_entries.rs` (400 `VALIDATION_ERROR`),
tests de non-régression.

Cherche : défauts introduits (tous les appelants de `validate` et tous les `match` sur `CoreError` traitent-ils la
nouvelle variante — grep — dont les chemins automatiques, les lots (`FailedProposal`), le MCP et la restauration ? un
`match` non exhaustif ou un bras générique qui la rendrait en 500 ? le message dans les quatre locales si la route le
traduit ? les tests rougissent-ils sans le correctif ?). Déclare les axes exercés / non exercés.

Phase 2 : corrige, avec tests ; gate ciblé (fmt, clippy workspace, nextest des binaires touchés ; **complet si
`kesh-db` est touché**) par `<station>/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>`, base
`kesh_s37_3a` (aucun `GRANT` à la main) ; commit « fix(37-3a) : revue de code P4 — … » (aucun mot-clé de fermeture,
**aucune mention d'IA**). Change Log « 2026-11-16 — revue de code P4 ciblée (<modèle>) : <décompte> ». Aucun push.
Si tu ne corriges rien : aucun commit, aucun gate.

Rapport : findings `37-3a-CR-P4-N` (sévérité, fichier:ligne, constat, correctif), axes, gates (chiffres), décompte.
