# Prompt — passe ciblée sur la remédiation du fichier d'epic (2026-11-26)

Worktree `<worktree>/kesh-e55`. Passe **ciblée**, contexte frais, modèle Sonnet, en
français. Juge **seulement** ce que le commit `ca6c934de` change au fichier d'epic
(`_bmad-output/planning-artifacts/epic-55-navigation-interface.md`) et à `sprint-status.yaml` (bloc de l'Epic 55) : il
remédie Pv2p7-6 (ligne du tableau des fichiers communs coupée en deux) et Pv2p7-7 (renvoi à une section inexistante ;
commentaire de tête du bloc) du rapport `$GATE_LOGS/s55-decisions-2/55-3a-validation-pv2p7.md`.
Les ajouts aux Change Logs des fiches du même commit ne sont que des renvois au T0 : vérifie seulement qu'ils citent
fidèlement les findings des rapports `55-3a-validation-pv2p7.md` et `55-2-55-1b-validation-p8.md` (même dossier).

Axes : A1 la remédiation est-elle juste (le tableau se lit-il en Markdown, chaque rangée commençant par `|` :
`awk '/^## Fichiers communs/,/^## Réponses/' … | grep -v '^|'`) ; A2 aucun défaut neuf ; A3 fidélité des renvois au T0.
Déclare les axes exercés et non exercés. Sévérités : CRITICAL / HIGH / MEDIUM / LOW.

Interdits : lecture seule ; n'écris aucun fichier du worktree ; aucun `cargo`, `npm`, `scripts/*`, `gh`, ni `git` qui
modifie ; aucun port. Rapport : un seul fichier,
`$GATE_LOGS/s55-decisions-2/epic-55-validation-remediation.md`, puis le même contenu en réponse.
