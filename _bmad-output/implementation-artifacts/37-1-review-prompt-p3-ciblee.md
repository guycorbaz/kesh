# Prompt de revue de code P3 ciblée — story 37-1

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), une seule lentille, contexte frais, braquée sur la remédiation P2 :
`git show a7a1d70a` (et `git diff 822fe8b5 a7a1d70a`), worktree `<dépôt>/.claude/worktrees/kesh-s37-1`
(branche `story/37-1-reglage-et-devise-des-comptes`). N'écris nulle part ailleurs, sauf journaux dans
`<station>/kesh-gate-logs/s37-1/review-p3/`.

À lire : la fiche 37-1 (AC, entrée « revue de code P2 » du Change Log), le rapport implicite des correctifs : refus
`foreignCurrency` jugé sur l'état résultant (`check_own_fx_accounts_in_tx`, prop `accountId` d'`AccountCurrencyFields`),
`accounts::reactivate` (verrous `companies` S puis parent S, refus `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`), `accounts::create`
(`notPostable` sous son propre compte de différence), `finalize` (refus gardé puis rendu après lecture de l'étape),
`CurrencySettingsSection` (rechargement conditionnel), messages i18n neufs, manuel, doc de `check_fx_accounts_in_tx`.

Cherche : défauts introduits par ces correctifs (ordre des verrous et cycles nouveaux — `reactivate` prend désormais
`companies` : contre qui ? rejeu ?), régressions, tests muets ou manquants (un test rougit-il sans le correctif ?),
i18n 4 locales, manuel `.tex`, résidus du symptôme ailleurs (grep). Déclare les axes exercés / non exercés.

Phase 2 : corrige, avec tests ; gate ciblé (fmt, clippy workspace, nextest des binaires touchés ; **complet si
`kesh-db` est touché**) sous `scripts/mem-guard.sh`, verrou `flock`, base `kesh_s37_1` (pas de `GRANT` à la main) ;
commit « fix(37-1) : revue de code P3 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log
« 2026-11-16 — revue de code P3 ciblée (<modèle>) : <décompte> ». Aucun push, aucun script mutant hors gates.

Rapport : findings `37-1-CR-P3-N` (sévérité, fichier:ligne, constat, correctif), axes, gates (chiffres), décompte.
