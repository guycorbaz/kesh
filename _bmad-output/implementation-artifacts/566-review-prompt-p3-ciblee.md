# Revue de code P3 ciblée — issue #566 — une lentille sur la dernière remédiation

Prompt versionné. Modèle : Haiku, contexte frais, lecture seule. Passe CIBLÉE (CLAUDE.md, « La passe ciblée ») : une
seule lentille, braquée sur le seul commit `2b15f877` (remédiation des LOW de la passe 2), dans le worktree
`/home/gcorbaz/devel/kesh-566`.

Lis le diff aplati : `git -C /home/gcorbaz/devel/kesh-566 show 2b15f877`. Contexte : rapport de la passe 2,
`/home/gcorbaz/devel/kesh-gate-logs/566-review-p2-R.md` (findings P2-1 à P2-5 que ce commit corrige).

## ⛔ Interdits

N'exécute JAMAIS `scripts/prepare-release.sh` (il écrit : bump des crates, CHANGELOG), ni
`scripts/tests/prepare-release.test.sh`, ni `cargo`, ni aucun script du dépôt. Aucune commande qui écrit (git
commit/checkout/stash/reset/tag, gh). Autorisés : `git show`, `git diff`, `grep`, `sed -n`, `cat`.

## Question

Ce commit a-t-il introduit un défaut ? En particulier :
1. Les motifs ancrés (`grep -qE "^## \[$ESC_VERSION\] — Non publié"`, `sed -i "s|^## \[$ESC_VERSION\] — Non publié|…|"`)
   sont-ils corrects — échappement des `[`, `]` et `.`, guillemets, `|` comme délimiteur du sed, `$ESC_VERSION`
   défini AVANT ces usages ?
2. La garde « version de kesh-api illisible » : sort-elle avant toute écriture ? `|| true` sous `pipefail` correct ?
3. L'indentation du bloc de bump : la structure `if/else/fi` est-elle restée identique (aucune instruction déplacée
   hors ou dans un bloc) ?
4. Le `unset GIT_DIR …` du test, et les compteurs ancrés du test (`grep -c "^## \\[$2\\] — "`) : corrects, et le test
   ne peut-il pas passer à vide ?
5. Le texte ajouté au `CLAUDE.md` (point 6) et à l'en-tête du script dit-il exactement ce que fait le code ?

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/566-review-p3-R.md` : axes exercés / NON exercés ; findings `P3-n` avec
sévérité (CRITICAL/HIGH/MEDIUM/LOW), fichier:ligne, preuve par sortie `grep -nF` copiée, correctif ; verdict chiffré.
