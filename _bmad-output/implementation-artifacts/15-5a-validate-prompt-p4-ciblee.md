# Prompt — validation P4 CIBLÉE de la spec, Story 15-5a

*Versionné le 2026-10-08. Une seule lentille (Haiku), contexte frais, lecture seule. Passe ciblée (CLAUDE.md § « La passe
ciblée ») : la P3 n'a laissé qu'un MEDIUM (test du pont count/numbers), remédié avec des LOW par le commit `007c4eb1`, qui
ne touche que des fiches. Haiku est réservé à ce type de passe (décision D6).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. Fiche :
`_bmad-output/implementation-artifacts/15-5a-refus-non-imputable.md`.

## Lentille unique — Regression hunter sur la dernière remédiation

Lis `git diff 92770300 007c4eb1 -- _bmad-output/implementation-artifacts/15-5a-refus-non-imputable.md` (un seul diff
aplati — ne lis pas les commits séparément), puis la fiche entière. Pour chaque hunk : la remédiation a-t-elle introduit
une contradiction (avec les autres AC, tâches, Dev Notes, ou le registre C27–C32 de
`epic-15-choix-autonomes.md`), un fait faux (vérifie au code par `grep -nF`), un test qui ne prouverait rien (le test du
bras API distingue-t-il vraiment singulier et pluriel ? le mutant est-il décrit de façon exécutable ?), ou un accesseur
`details` de `NonPostableAccounts` incompatible avec l'usage qu'en fait la 15-5b (`15-5b-gardes-surfaces-neuves.md`) ?

⛔ Pour tout finding affirmant qu'un texte ou un code est absent ou présent : la sortie d'un `grep -nF`.

## Ce que tu rends

Écris ton rapport complet dans `target/gate-logs/15-5a-p4-ciblee.md` : findings numérotés avec sévérité, preuve,
correction ; **axes exercés ET non exercés**. Dernier message : le chemin, le bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/` ; aucune commande qui écrit (scripts/*, make, latexmk, git commit/add/
checkout/stash, sqlx, cargo, npm, npx, gh issue create/comment/edit, SQL d'écriture). Autorisés : lecture, grep, sed -n,
git log/show/diff, gh api en lecture.
