# Prompt — validation P1 des fiches de l'Epic 33 (v0.17.5), Sonnet, contexte frais

Tu valides des **fiches** de story de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11). Kesh 0.17.4 tient les
livres réels de son auteur. Worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`, partie de `c2051805`.
Langue du rapport : français. **Première passe, complète.** Ne t'appuie pas sur ce que les fiches disent d'elles-mêmes :
refais les relevés au code.

**Lentilles** : **R, la règle et sa preuve** : chaque prémisse est-elle vraie au code ? Chaque conception ferme-t-elle le
défaut sans en ouvrir un autre ? Chaque AC est-il décidable, chaque test rouge avant le correctif, chaque mutation tuée par le
test qu'elle nomme ? Chaque inventaire clos l'est-il ? **F, les textes** : manuels (`.tex` et PDF), CHANGELOG, API, i18n,
décomptes, numéros de ligne, cohérence entre fiches et registre.

**À lire** : `_bmad-output/planning-artifacts/epic-33-numerotation-et-defauts.md` (les décisions de l'auteur font foi : une
fiche qui s'en écarte sans le déclarer est un finding) ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/e33-commun.md` ; les
fiches de ton lot, en entier ; leur registre `e33-choix-autonomes-*.md` ; les issues en lecture seule
(`gh api repos/guycorbaz/kesh/issues/<n>` et `…/comments`) ; le code qu'elles citent et ses voisins ; les règles du `CLAUDE.md`
qui s'appliquent (*Migration breaking policy* P1-P8, *Review Iteration Rule*, *Recette visuelle*).

**Faits à connaître** : #742 et #287 sont fermées, reportées dans #606 et #424 ; #165 et #166 sont fermées, remplacées par
#918 : une fiche qui prévoit de les laisser ouvertes ou de les commenter est un LOW à signaler. #926 et #922 sont en P1.

**Sévérités** : CRITICAL, HIGH, MEDIUM, LOW, au sens du `CLAUDE.md`. Un finding cite le fichier et la ligne lus, le scénario
d'échec, et une remédiation. N'invente pas de défaut pour remplir : un « 0 » adossé à la liste des axes exercés est une
réponse valable. **Le rapport déclare les axes réellement exercés et ceux qui ne l'ont pas été** ; sans cela la passe ne
compte pas.

⛔ **Lecture seule** : aucune écriture sauf ton rapport, ni compilation, ni test, ni `make`, ni `scripts/prepare-release.sh`, ni
`git` qui écrit, ni `gh` qui écrit, ni `rm`, ni redirection ou fichier temporaire. `grep` est une fonction vers `ugrep` :
utilise `/usr/bin/grep` ou `git grep`. Pas d'accès à `~/kesh/`, `~/kesh-test/` ni au NAS.

Message final : le décompte C/H/M/L recompté, chaque finding au-dessus de LOW en une ligne, et le chemin du rapport.
