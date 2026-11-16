# Prompt — revue de code P3 de la Story 36-5 (#931, KF-056) : passe CIBLÉE, une lentille

Tu relis **seulement la remédiation de la revue P2** de la Story 36-5 de Kesh (SvelteKit 5 runes), release 0.17.8. La P2
(deux lentilles, `kesh-gate-logs/e36-5/review-p2-{E,A}.md`) n'a rien rendu au-dessus de LOW ; ses LOW ont été corrigés dans
la story, et **l'un touche la production** (E2-1) : c'est pourquoi cette passe existe (`CLAUDE.md`, § *Priorités des défauts*,
« Tarir la source »). Contexte frais. Modèle : **Haiku** permis (passe ciblée, non première passe), ou Sonnet. Langue :
français.

Worktree `/home/gcorbaz/devel/kesh-36-5`. Le diff à relire, **aplati**, et rien d'autre :
`git -C /home/gcorbaz/devel/kesh-36-5 diff 94ec4ccb4 HEAD` — `fb2012f1b` (code et tests, **dernier commit de code**),
`eea1e0ea6` (fiche, manuel, `sprint-status.yaml`), puis le compte rendu (Dev Agent Record, Change Log, ce prompt).

## La lentille : Regression Hunter — ce que la remédiation P2 a pu casser

1. **E2-1, production** (`frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte`, `finally` de `confirmImport`) :
   `if (!keepHold && !destroyed)`. Une instance **non** démontée garde-t-elle exactement le comportement d'avant (échéance
   ramenée à B, levée au premier geste) ? Une instance démontée pendant l'import n'a-t-elle plus aucun effet sur le maintien,
   ni raccourci ni écouteur ? Un cas où `destroyed` est vrai mais où le maintien appartient encore à cette instance (et
   devrait être raccourci) existe-t-il ? (Lis `onDestroy` : il lève le maintien.)
2. **E2-3** : le commentaire de tête de `frontend/src/lib/app/stores/session-idle.svelte.ts` dit-il vrai au code (phase 1, échec,
   phase 2) ?
3. **Les quatre tests neufs** (`AdminRestorePanel.test.ts`, fin du fichier) : prouvent-ils ce que leur nom dit ? Le test E2-1
   (deux panneaux rendus l'un après l'autre) peut-il passer à vide (le premier panneau réellement démonté ? sa promesse
   réellement rejetée après ?) ; le test de l'ordre (espion sur `globalThis.setTimeout`) attrape-t-il le bon appel, et
   l'espion est-il restauré même en cas d'échec ? Les tests à horloge simulée laissent-ils une minuterie ou un maintien qui
   fuit au test suivant (`afterEach`) ?
4. **Mutations M27 à M30** (`kesh-gate-logs/e36-5/mut/mutations.py`, à lire ; `mut/M27.log` à `mut/M30.log` ; campagne 3 dans
   `mut/resume.log` et `mut/campagne-3.console.log`) : chaque diff est-il la mutation décrite à la table de la fiche, et chacune
   tuée par le test qu'elle nomme ? Les 36 précédentes toujours tuées (40 sur 40) ?
5. **Fiche et manuel** : la troisième méthode à D1 et aux *Simulacres* ; B relu (définition de B) ; annotations datées (A2-3,
   A2-4) qui **ajoutent** sans réécrire les relevés passés ; la demi-phrase neuve de `docs/manual/fr/admin-manual.tex:2342`
   (« Laissé ouvert sans activité, cet écran mène à la connexion au plus tard au bout de la durée d'inactivité réglée ») —
   est-elle vraie au code après une issue inconnue (maintien ramené à `échec + B`, puis régime ordinaire) ?
6. **Gates** : les journaux `r2-test-fast-ci.log`, `r2-front-*.log`, `repetition-apres-d5-r4.log` portent-ils le commit en
   première ligne, et leurs chiffres sont-ils ceux du Dev Agent Record ? (`gate-ci` et l'E2E complet ne tournent qu'à la clôture :
   le dire n'est pas un finding.)

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. Aucun fichier temporaire ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris).
- Aucun `rm`, `pkill`, `killall`, `kill` ; aucun `git` qui écrit ; aucune commande `gh` ; aucun accès réseau.
- **Rien qui exécute ou compile** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`, `jq`, `unzip` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh` ni aucun script du
  dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `wc`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`. `grep` est une fonction vers
  `ugrep` : emploie `/usr/bin/grep`.
- Toute affirmation de présence ou d'absence : sortie `grep -nF` ou `sed -n` copiée ; tout chemin cité vérifié, ligne lue citée.

## Rapport

Un seul : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/review-p3-ciblee.md`. En-tête (lentille, modèle, commit relu, date) ;
findings numérotés `P3-1`…, chacun avec sa sévérité (`CRITICAL`/`HIGH`/`MEDIUM`/`LOW`), son lieu, la ligne lue citée, la preuve
copiée, le remède ; **axes exercés** et **axes non exercés** (un « 0 finding » sans eux ne compte pas) ; décompte par sévérité
recompté depuis la liste.
