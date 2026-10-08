# Prompt — validation P4 CIBLÉE de la spec, Story 15-8a

*Versionné le 2026-10-08. Une lentille (Haiku), contexte frais, lecture seule. Passe ciblée (CLAUDE.md § « La passe ciblée ») : les MEDIUM de la P3 venaient tous de la remédiation P2 ; la remédiation P3 (`ba0965f0`, après rebase sur origin/main) ne touche que des fiches. Haiku est réservé à ce type de passe (D6). ⚠️ Story URGENTE.*

Worktree `/home/gcorbaz/devel/kesh-15-8`, branche `story/15-8-modifier-une-ecriture`. Fiche : `_bmad-output/implementation-artifacts/15-8a-modifier-une-ecriture.md`.

## Lentille unique — Regression hunter sur la dernière remédiation

Lis `git diff ba0965f0~1 ba0965f0 -- _bmad-output/implementation-artifacts/15-8a-modifier-une-ecriture.md` (un seul diff), puis la fiche entière. Pour chaque hunk : contradiction introduite (AC, tâches, Dev Notes, registre C-15-8-22 à 29, fiche sœur) ? fait faux (vérifie au code, maintenant rebasé sur origin/main, par `grep -nF`) ? test qui ne prouverait rien (la règle « exercice postérieur clos » `LATER_FISCAL_YEAR_CLOSED` a-t-elle un test qui mord et sa place dans la précédence) ? décompte faux (recompte AC, tâches, codes d'écran) ?

⛔ Toute affirmation de présence ou d'absence : la sortie d'un `grep -nF`, copiée. ⛔ Un « 0 » sans la liste des axes exercés ET non exercés ne compte pas.

Rapport complet dans `target/gate-logs/15-8a-p4-ciblee.md`. Dernier message : le chemin, le bilan, une ligne par MEDIUM+.

⛔ Interdits : n'écris rien hors `target/gate-logs/` ; aucune commande qui écrit, compile ou exécute. Autorisés : lecture, grep, sed -n, git log/show/diff, gh api en lecture.
