# Prompt — validation P4 ciblée des fiches de l'Epic 33 (v0.17.5), Haiku, contexte frais

**Passe ciblée, une seule lentille : la dernière remédiation.** Worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`. Tu relis **uniquement** le diff du commit de remédiation, aplati : `git show --stat HEAD~1` puis `git diff HEAD~2 HEAD~1 -- _bmad-output/implementation-artifacts/`. Rapport en français.

Pour chaque changement, dans son contexte de fiche : ce qu'il affirme est-il vrai au code cité (lis la ligne) ? Contredit-il une autre phrase de la même fiche ? Le test ou la mutation qu'il nomme existe-t-il dans la fiche, et le décompte de mutations est-il juste ? Lis en particulier : **33-11** (F3, conditions `needs.changes.result`, AC-5, AC-6 (d)) ; **33-2** (AC-F4, coupe à la largeur de colonne) ; **33-3b** (prémisse 5, `tickLabel`) ; **33-6** (M5) ; **33-5** (cas `SU`).

Décisions de l'auteur à ne pas contester : #435 se ferme, connexion, déconnexion, renouvellement et signal d'activité ne se tracent pas ; `{FY}` retiré du format fournisseur.

Sévérités CRITICAL, HIGH, MEDIUM, LOW du `CLAUDE.md`. Tout finding CRITICAL, HIGH ou MEDIUM cite la ligne lue et la commande `grep -nF` qui l'établit. Déclare les axes exercés et non exercés.

⛔ Lecture seule : aucune écriture sauf ton rapport `/home/gcorbaz/devel/kesh-gate-logs/e33-validate-p4.md`, ni compilation, ni test, ni script, ni `git` ou `gh` qui écrit, ni `rm`. `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep`.

Message final : décompte C/H/M/L recompté, chaque finding au-dessus de LOW en une ligne.
