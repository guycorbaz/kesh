# Prompt de revue de code P3 ciblée — story 37-2

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), une seule lentille, contexte frais, braquée sur la remédiation P2 :
worktree `<worktree>/kesh-s37-2`, `git show 9a665075` (P2 : `copied` de l'audit de `POST /fetch` compté par société dans `recuperation.rs::ecrire`, test `fetch_audit_ne_compte_que_la_societe_de_l_utilisateur` ; doc de `un_tour` ; `legend` de `CorrectionDialog.svelte`, `sitesTotal` 3185 ; manuel de l'acquittement), (branche `story/37-2-cours-mensuels-afc`). N'écris nulle part ailleurs, sauf journaux dans
`$GATE_LOGS/s37-2/review-p3/`.

À lire : la fiche 37-2 (AC, entrées « revue de code P1 » et « P2 » du Change Log).

Cherche : défauts introduits par ces correctifs (le décompte par société est-il juste avec plusieurs sociétés et
aucune copie ? le test rougit-il sans le correctif ?), régressions, i18n, résidus du symptôme (grep des autres
compteurs `copied`). Déclare les axes exercés / non exercés.

Phase 2 : corrige, avec tests ; gate ciblé (fmt, clippy workspace, nextest des binaires touchés ; **complet si
`kesh-db` est touché**) par `$GATE_LOGS/heavy.sh scripts/mem-guard.sh <commande>`, base `kesh_s37_2` (pas de `GRANT` à la main) ;
commit « fix(37-2) : revue de code P3 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log
« 2026-11-16 — revue de code P3 ciblée (<modèle>) : <décompte> ». Aucun push, aucun script mutant hors gates.

Rapport : findings `37-2-CR-P3-N` (sévérité, fichier:ligne, constat, correctif), axes, gates (chiffres), décompte.
