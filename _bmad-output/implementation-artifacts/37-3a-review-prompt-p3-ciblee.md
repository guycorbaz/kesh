# Prompt de revue de code P3 ciblée — story 37-3a

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), une seule lentille, contexte frais, braquée sur la remédiation P2 :
worktree `<worktree>/kesh-s37-3a` (branche `story/37-3a-ecritures-en-devise-modele`),
`git show 21b84c09`. N'écris nulle part ailleurs, sauf journaux dans `$GATE_LOGS/s37-3a/review-p3/`.

À lire : la fiche 37-3a (AC, entrées « revue de code P1 » et « P2 » du Change Log).

Correctifs de la P2 à relire : devise du compte de charge lue dans le `SELECT … FOR UPDATE` de `validate_in_tx`
(`supplier_invoices.rs`) avec refus sur l'identifiant le plus petit ; `checked_add` de `debit + credit` dans
`parse_line_amounts` (route) et `check_foreign_line` (noyau), 400 `FX_VALUE_OUT_OF_RANGE` / `OutOfRange` ; message de
`MULTI_CURRENCY_DISABLED` (quatre locales et repli) ; test de `chf_text`.

Cherche : défauts introduits (ordre des verrous dans `validate_in_tx` et risque d'interblocage, rejeu de la route ;
le refus déterministe rend-il le bon compte ? le débordement peut-il encore paniquer ailleurs — grep des autres
`debit + credit` ou sommes `Decimal` du chemin FX ; le nouveau message est-il juste sur toutes les routes qui le
rendent ; les tests rougissent-ils sans leur correctif ?). Déclare les axes exercés / non exercés.

Phase 2 : corrige, avec tests ; gate ciblé (fmt, clippy workspace, nextest des binaires touchés ; **complet si
`kesh-db` est touché**) par `$GATE_LOGS/heavy.sh scripts/mem-guard.sh <commande>`, base
`kesh_s37_3a` (aucun `GRANT` à la main) ; commit « fix(37-3a) : revue de code P3 — … » (aucun mot-clé de fermeture,
**aucune mention d'IA**). Change Log « 2026-11-16 — revue de code P3 ciblée (<modèle>) : <décompte> ». Aucun push.
Si tu ne corriges rien : aucun commit, aucun gate.

Rapport : findings `37-3a-CR-P3-N` (sévérité, fichier:ligne, constat, correctif), axes, gates (chiffres), décompte.
