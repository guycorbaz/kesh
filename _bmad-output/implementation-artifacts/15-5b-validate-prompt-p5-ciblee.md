# Prompt — validation P5 CIBLÉE de la spec, Story 15-5b

*Versionné le 2026-10-08. Une seule lentille (Haiku), contexte frais, lecture seule. Passe ciblée (CLAUDE.md § « La passe
ciblée ») sur la dernière remédiation, le commit `67c31c95`, qui ne touche que des fiches. Haiku est réservé à ce type de
passe (décision D6).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. Fiche :
`_bmad-output/implementation-artifacts/15-5b-gardes-surfaces-neuves.md`. Issues (par `gh api repos/guycorbaz/kesh/issues/N`) : 427, 429, 521.

## Lentille unique — Regression hunter sur la dernière remédiation

Lis `git diff 007c4eb1 67c31c95 -- _bmad-output/implementation-artifacts/15-5b-gardes-surfaces-neuves.md` (un seul diff aplati — ne lis pas les commits
séparément), puis la fiche entière, et le registre `epic-15-choix-autonomes.md` (entrées C33–C38). Pour chaque hunk :
contradiction introduite (AC, tâches, Dev Notes, registre, fiches sœurs 15-5a/b/c/d) ? fait faux (vérifie au code ou au
manuel par `grep -nF`) ? test qui ne prouverait rien ? numéro de ligne ou chemin faux ? décompte faux (recompte AC et
tâches) ?

⛔ Pour tout finding affirmant qu'un texte ou un code est absent ou présent : la sortie d'un `grep -nF`, copiée dans le
rapport. ⛔ Un « 0 finding » sans la liste des axes exercés ET non exercés ne compte pas.

## Ce que tu rends

Rapport complet dans `target/gate-logs/15-5b-p5-ciblee.md`. Dernier message : le chemin, le bilan par sévérité, une ligne
par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/` ; aucune commande qui écrit (scripts/*, make, latexmk, git commit/add/
checkout/stash, sqlx, cargo, npm, npx, gh issue create/comment/edit, SQL d'écriture). Autorisés : lecture, grep, sed -n,
git log/show/diff, gh api en lecture.
