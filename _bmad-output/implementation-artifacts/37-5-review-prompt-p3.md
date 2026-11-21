# Prompt de revue de code P3 — story 37-5

Passe braquée sur la remédiation P2 (`git show 2a370c4e`), mais **complète sur son rayon** : elle change une règle
métier (le prédicat des comptes candidats à la réévaluation ne regarde plus `postable`), donc relis tous les sites qui
en dépendent, pas seulement le diff. Worktree `<dépôt>/.claude/worktrees/kesh-s37-5` (branche
`story/37-5-reevaluation-fin-exercice`). N'écris nulle part ailleurs, sauf journaux dans
`<station>/kesh-gate-logs/s37-5/review-p3/`.

À lire : la fiche 37-5 (AC, règle D1, entrées « revue de code P1 » et « P2 » du Change Log), l'epic (CR-8, A37, A41).

Cherche :
1. **Règle D1** : un compte de bilan en devise non imputable à solde bloque l'aperçu (non sélectionnable,
   `ACCOUNT_NOT_POSTABLE`), K9 (`NOT_POSTABLE`) et la clôture (409). Les **mêmes** comptes sont-ils candidats aux trois
   endroits (une seule définition, `lines_for_fiscal_year` / `not_revalued` / `closing_checks`) ? Un compte **parent**
   (non imputable parce qu'il a des sous-comptes) dont le solde n'est que la somme de ses enfants est-il compté deux
   fois, ou bloque-t-il à tort alors que ses enfants sont réévalués ? Un compte archivé, un compte de résultat, un
   compte CHF restent-ils hors du prédicat ? Le message dit-il quoi faire, dans les 4 locales ?
2. **D2** (blocker `FX_VALUE_OUT_OF_RANGE` par compte), « Récupérer le cours » qui recharge l'aperçu, « Réévaluer »
   désactivé pendant le chargement, `is_eligible`.
3. Fiche, epic (six codes de `blockers[]`), guide d'API et manuel `.tex` contre le code.
4. Tests : rougissent-ils sans leur correctif ? Tests manquants (compte parent ci-dessus).
Déclare les axes exercés / non exercés.

Phase 2 : corrige, avec tests ; gate complet si `kesh-db` est touché (`<station>/kesh-gate-logs/heavy.sh
scripts/test-fast.sh --ci`, sans `mem-guard.sh` en plus), Vitest complet seul et build si le frontend est touché ; base
`kesh_s37_5` sur `kesh-mariadb-dev3` (3308), aucun `GRANT` ; commit « fix(37-5) : revue de code P3 — … » (aucun mot-clé
de fermeture, **aucune mention d'IA**). Change Log « 2026-11-17 — revue de code P3 (<modèle>) : <décompte> ». Aucun
push. Si tu ne corriges rien : aucun commit, aucun gate.

Rapport : findings `37-5-CR-P3-N` (sévérité, fichier:ligne, constat, correctif), axes, gates (chiffres), décompte.
