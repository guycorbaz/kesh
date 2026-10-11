# Prompt — passe ciblée P5 de la revue de code de la Story 26-3 (Sonnet, une lentille)

Une seule lentille, **chasseur de régression de la remédiation P4**, braquée sur **le seul commit**
`21dac14f` : `git -C /home/gcorbaz/devel/kesh-26-3 show 21dac14f`. Contexte : rapport P4
`/home/gcorbaz/devel/kesh-gate-logs/26-3-review-p4-ciblee.md` ; journal des mutations réelles
`/home/gcorbaz/devel/kesh-gate-logs/26-3-mutations-p4.log` ; fiche
`_bmad-output/implementation-artifacts/26-3-manuel-et-restauration-synology.md` (AC 7.5 et 7.6, Change Log,
entrée « Passe ciblée P4 remédiée », Dev Agent Record « Gates au commit de remédiation P4 »). Langue : français.
⚠️ Lis les fichiers **à HEAD** (`sed -n`, `grep -nF`) pour toute affirmation de présence ou d'absence, et
copie la sortie dans le rapport.

## Interdits absolus

Lecture seule. N'exécute AUCUN script du dépôt — nommément `scripts/prepare-release.sh` (il bumpe les
versions du workspace), `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/install-hooks.sh`,
`scripts/synology/*.sh` —, ni `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `sqlx`, `mariadb`.
Aucun `git` qui écrit (`commit`, `checkout`, `stash`, `reset`, `add`, `tag`, `push`), aucune commande `gh`
qui écrit. N'écris AUCUN fichier hors de ton rapport. Permis : `git show`, `git diff`, `git log`, `git grep`,
`grep`, `sed -n`, `cat`, `bash -n`. Tu peux **raisonner** sur une mutation ; tu ne peux pas la compiler —
dis-le dans le rapport et ne déclare « rouge » ou « vert » qu'en le qualifiant de raisonnement.

## Axes

1. **`boucle_lit_la_liste`** (`crates/kesh-api/tests/configuration_transmise.rs`) contre
   `scripts/synology/kesh-restore.sh` à HEAD :
   - la profondeur est-elle juste ? Le regex `(?:^|;)\s*do$` compte-t-il toute ouverture (`for … do` sur
     sa propre ligne `do`, `while …; do # commentaire`), et `^done\b` toute fermeture ? Une ouverture non
     comptée ferait prendre un `done` intérieur pour la fermeture — faux rouge ou faux vert ?
   - la fonction ne lit que du code passé par `code_shell` : quelles formes un commentaire **de fin de
     ligne** peut-il encore prendre pour tromper une des trois conditions (ligne `LISTE=`, `while`,
     lecture `IFS='|'`, fermeture) ? Les égalités exactes (`*l == format`, `*l == "while …; do"`) sont-elles
     trop rigides (P4-2) ou trop lâches ?
   - `LISTE` réaffectée entre sa lecture et la boucle, ou `while` placé avant `LISTE=` : la fonction le
     voit-elle ? Si non, est-ce écrit comme angle mort ?
2. **Ensembles de mutation de l'AC 7.6** : la clause sortie du tableau de regex garde-t-elle **le même
   libellé** d'erreur, de sorte que (a) donne toujours 7.1 + 7.5 et (e) 7.5 seule ? Le journal
   `26-3-mutations-p4.log` le montre-t-il pour chaque ligne de la table, mutation (f) comprise ?
3. **Ordre des étapes** : la clause 7.5 n'est plus dans le tableau ordonné ni dans le tableau des clauses ;
   a-t-elle perdu un contrôle d'ordre qu'elle portait auparavant (comparer `git show 21dac14f^:…` et HEAD) ?
4. **Comptes rendus** : le Change Log P4, le paragraphe « Gates au commit de remédiation P4 » et la
   doc-comment de G16 disent-ils vrai contre le code et les journaux (`kesh-gate-logs/26-3-gate-p4.log`,
   `26-3-e2e-p4.log`) ? Recompte chaque nombre (tests, échecs, trend des passes P1→P4) depuis sa source.
5. **Propagation** : `git grep -nF 'done <<< "$LISTE"'` et `git grep -n 'alimentée'` — un site du même
   symptôme reste-t-il faux (fiche AC 5.2, AC 7.5, registre `epic-26-choix-autonomes.md` C-26-3-22,
   commentaires du script) ?
6. **Manuel** : la remédiation ne touche pas `docs/manual/` ; vérifie-le (`git show --stat 21dac14f`) et
   dis si un texte du manuel décrit la garde d'une façon que la nouvelle clause rend fausse.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/26-3-review-p5-ciblee.md` : findings avec sévérité (CRITICAL / HIGH /
MEDIUM / LOW), origine (introduit par `21dac14f` ou préexistant), fichier:ligne, **sortie `grep -nF`/`sed -n`
copiée** ; **tableau des axes exercés ET non exercés** (un « 0 finding » sans ce tableau ne compte pas) ;
bilan chiffré.
