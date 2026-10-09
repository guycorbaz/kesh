# Prompt — validation P4 CIBLÉE de la spec, Story 15-6c

*Versionné le 2026-10-08. Une lentille (Haiku), contexte frais, lecture seule. Passe ciblée (CLAUDE.md § « La passe ciblée ») : la P3 n'a laissé que des MEDIUM de tests manquants, remédiés par le commit `89e8987a`, qui ne touche que des fiches. Haiku est réservé à ce type de passe (décision D6).*

Worktree `/home/gcorbaz/devel/kesh-15-6`, branche `story/15-6-creance-juste-avoir-reglement`. Fiche : `_bmad-output/implementation-artifacts/15-6c-configuration-sans-ecriture-nulle.md`.

## Lentille unique — Regression hunter sur la dernière remédiation

Lis `git diff 4964eeb4 89e8987a -- _bmad-output/implementation-artifacts/15-6c-configuration-sans-ecriture-nulle.md` (un seul diff aplati), puis la fiche entière. Pour chaque hunk : contradiction introduite (AC, tâches, Dev Notes, registre C-15-6-*) ? fait faux (vérifie au code par `grep -nF`) ? test qui ne prouverait rien (le test 12 bis mord-il sur le `FOR UPDATE` de `before` ? `attendre_une_requete_en_cours` est-il employé avec un motif qui existe ?) ? décompte faux (recompte AC, tâches, tests) ?

⛔ Toute affirmation de présence ou d'absence : la sortie d'un `grep -nF`, copiée. ⛔ Un « 0 » sans la liste des axes exercés ET non exercés ne compte pas.

Rapport complet dans `target/gate-logs/15-6c-p4-ciblee.md`. Dernier message : le chemin, le bilan, une ligne par MEDIUM+.

⛔ Interdits : n'écris rien hors `target/gate-logs/` ; aucune commande qui écrit, compile ou exécute (scripts/*, make, latexmk, git commit/add/checkout/stash, sqlx, cargo, npm, npx, gh issue create/comment/edit, SQL d'écriture). Autorisés : lecture, grep, sed -n, git log/show/diff, gh api en lecture.
