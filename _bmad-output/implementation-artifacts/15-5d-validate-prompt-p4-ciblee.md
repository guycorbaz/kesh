# Prompt — validation P4 CIBLÉE de la spec, Story 15-5d

*Versionné le 2026-10-08. Une lentille (Haiku), contexte frais, lecture seule. Passe ciblée sur le découpage C52 (commit `164e143b`), qui ne touche que des fiches. Haiku est réservé à ce type de passe (D6).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. Fiche : `_bmad-output/implementation-artifacts/15-5d-garde-usage-comptes-reglage.md` ; fiche sœur née du découpage : `15-5e-ordre-des-verrous-reglements.md`.

## Lentille unique — Regression hunter sur le découpage

Lis `git diff 8d312c32 164e143b -- _bmad-output/implementation-artifacts/15-5d-garde-usage-comptes-reglage.md _bmad-output/implementation-artifacts/15-5e-ordre-des-verrous-reglements.md` (un seul diff aplati), puis les deux fiches. Le découpage a-t-il perdu quelque chose (un AC, un test, une mutation, un site, un renvoi) qui n'est plus dans aucune des deux fiches ? Une chose est-elle dans les deux ? La 15-5d suppose-t-elle de la 15-5e un ordre ou un nom qui n'y figure pas ? Recompte AC, tâches, tests de chaque fiche. Faits cités : `grep -nF` au code.

⛔ Toute affirmation de présence ou d'absence : la sortie d'un `grep -nF`, copiée. ⛔ Un « 0 » sans la liste des axes exercés ET non exercés ne compte pas.

Rapport complet dans `target/gate-logs/15-5d-p4-ciblee.md`. Dernier message : le chemin, le bilan, une ligne par MEDIUM+.

⛔ Interdits : n'écris rien hors `target/gate-logs/` ; aucune commande qui écrit, compile ou exécute. Autorisés : lecture, grep, sed -n, git log/show/diff, gh api en lecture.
