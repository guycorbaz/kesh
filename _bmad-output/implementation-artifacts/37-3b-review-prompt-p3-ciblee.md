# Prompt de revue de code P3 ciblée — story 37-3b

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), une lentille, contexte frais, braquée sur la remédiation P2 :
worktree `/home/beat/claude/kesh/.claude/worktrees/kesh-s37-3b` (branche `story/37-3b-ecritures-en-devise-ecrans`),
`git show 9e2e7df6`. N'écris nulle part ailleurs, sauf journaux dans `/home/beat/claude/kesh-gate-logs/s37-3b/review-p3/`.
La remédiation touche plusieurs modules (kesh-db, kesh-api, frontend) : lis-la **entière**, fichier par fichier.

À lire : la fiche 37-3b (AC, entrées « revue de code P1 » et « P2 » du Change Log).

Correctifs de la P2 à relire : refus `SameCurrency` de `journal_entry_conversions.rs` (comment se juge la devise d'une
écriture, rôle des frais) ; 400 `FX_VALUE_OUT_OF_RANGE` de la ligne de différence ; `isTransientFailure` et la
non-mémoïsation des échecs dans `rate-lookup.ts` (`refresh()` compris) ; `selectAccount` qui ne fait rien sur un
compte inchangé (`JournalEntryForm.svelte`) ; `excludeForeignCurrency={!multiCurrency}` ; `opening-fx.test.ts` ;
`ConversionAssistant.svelte` (refus gardés, frais).

Cherche : défauts introduits (une conversion EUR→USD légitime peut-elle être refusée à tort par `SameCurrency`, p. ex.
une écriture faite de frais seuls, ou de lignes CHF de transit ? un 5xx durable fait-il maintenant une requête à
chaque frappe — boucle ? `selectAccount` inchangé empêche-t-il un rafraîchissement voulu, p. ex. après un changement
de date ? le filtre mono-devise cache-t-il un compte déjà choisi sur une écriture existante ?), tests manquants
(**CR-P2-4 n'a pas de test : écris-le**), i18n des 4 locales, résidus du symptôme (grep). Déclare les axes exercés /
non exercés.

Phase 2 : corrige, avec tests ; gate ciblé (fmt, clippy workspace, nextest des binaires touchés ; **complet si
`kesh-db` est touché**, par `/home/beat/claude/kesh-gate-logs/heavy.sh scripts/test-fast.sh --ci` sans `mem-guard.sh`
en plus ; Vitest complet seul puis `npm run build` si le frontend est touché), base `kesh_s37_3b` sur
`kesh-mariadb-dev` (3306), aucun `GRANT` ; commit « fix(37-3b) : revue de code P3 — … » (aucun mot-clé de fermeture,
**aucune mention d'IA**). Change Log « 2026-11-16 — revue de code P3 ciblée (<modèle>) : <décompte> ». Aucun push.

Rapport : findings `37-3b-CR-P3-N` (sévérité, fichier:ligne, constat, correctif), axes, gates (chiffres), décompte.
