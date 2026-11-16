# Prompt — revue de code P4 de la Story 36-5 (#931, KF-056) : passe CIBLÉE, une lentille

Tu relis **seulement la remédiation de la passe ciblée P3** de la Story 36-5 de Kesh (SvelteKit 5 runes), release 0.17.8. La P3
(`kesh-gate-logs/e36-5/review-p3-ciblee.md`) a rendu un MEDIUM au manuel (P3-1) et trois LOW. La remédiation ne touche aucune
ligne de code de production : une phrase **retirée** du manuel, un test réécrit, des annotations. Contexte frais. Modèle :
**Haiku** permis (passe ciblée), ou Sonnet. Langue : français.

Worktree `/home/gcorbaz/devel/kesh-36-5`. Le diff à relire, **aplati**, et rien d'autre :
`git -C /home/gcorbaz/devel/kesh-36-5 diff a0cb67b36 HEAD` — `68c949fa2` (manuel et test : **dernier commit de code**), puis
la fiche (Dev Notes, Dev Agent Record, Change Log) et ce prompt.

## La lentille : Regression Hunter — ce que la remédiation P3 a pu casser

1. **P3-1, le manuel** (`docs/manual/fr/admin-manual.tex:2342`) : la demi-phrase « Laissé ouvert sans activité, cet écran mène
   à la connexion au plus tard au bout de la durée d'inactivité réglée » est-elle **entièrement** retirée, et rien d'autre de la
   ligne n'a-t-il bougé ? Compare `git show f43562b4d:docs/manual/fr/admin-manual.tex | sed -n 2342p` et `sed -n 2342p` à `HEAD` :
   ils doivent être identiques. La seule différence du manuel avec `f43562b4d` est-elle la ligne `:2340` (la demi-phrase de D6
   et son incise) ? (`git diff f43562b4d HEAD -- docs/manual/fr/admin-manual.tex`.) Le PDF n'est pas régénéré (à
   l'intégration) ; contrôle-le sous la seule forme permise :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` — attendu `1` pour
   « Prenez le temps de lire, puis cliquez », `0` pour « Laissé ouvert sans activité » et `0` pour « Laissé ouvert, ou quitté ».
   La phrase de `:2340` (« Laissé ouvert, ou quitté, l'écran mène à la connexion — au bout de la durée d'inactivité réglée… »)
   dépend-elle, elle aussi, du stockage local partagé ? Lis `AdminRestorePanel.svelte` (minuterie de lecture `readingTimer`,
   `onDestroy`) avant de conclure.
2. **P3-3, le test** (`frontend/src/lib/features/admin-restore/AdminRestorePanel.test.ts`, test « (B1-3) ») : le `try/finally`
   démonte-t-il le panneau et restaure-t-il l'espion dans tous les cas ? Le démontage explicite, suivi du nettoyage automatique
   de testing-library, peut-il jeter ou rendre le test vide ? Le test attrape-t-il toujours l'ordre (minuterie avant le
   maintien), et la mutation M30 le ferait-elle toujours rougir (lis-la dans `kesh-gate-logs/e36-5/mut/mutations.py`) ?
3. **P3-2** : les Dev Notes citent-elles **#1193** et disent-elles juste (propriété du maintien au démontage) ?
4. **P3-4** : les annotations datées en fin de `r1-vitest-cible.log` et `r2-vitest-cible.log` nomment-elles les fichiers
   réellement exécutés (recoupe avec le corps de chaque journal), sans réécrire le relevé ?
5. **Compte rendu** : Dev Agent Record et Change Log disent-ils P3-1 retiré (décision de l'orchestrateur) et A2-6 facultatif ?
   Les chiffres des gates de cette remédiation (`r3-front-*.log`, commit en première ligne ; `r3-vitest-cible.log`) sont-ils ceux
   des journaux ? `gate-ci` et l'E2E complet ne tournent qu'à la clôture, et `test-fast` n'est pas relancé (aucun fichier Rust
   ni `.ftl` touché) : le dire n'est pas un finding.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. Aucun fichier temporaire ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris).
- Aucun `rm`, `pkill`, `killall`, `kill` ; aucun `git` qui écrit ; aucune commande `gh` ; aucun accès réseau.
- **Rien qui exécute ou compile** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`, `jq`, `unzip` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh` ni aucun script du
  dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `wc`, `cmp`, et le contrôle du PDF ci-dessus. `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep`.
- Toute affirmation de présence ou d'absence : sortie `grep -nF` ou `sed -n` copiée ; tout chemin cité vérifié, ligne lue citée.

## Rapport

Un seul : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/review-p4-ciblee.md`. En-tête (lentille, modèle, commit relu, date) ;
findings numérotés `P4-1`…, chacun avec sa sévérité (`CRITICAL`/`HIGH`/`MEDIUM`/`LOW`), son lieu, la ligne lue citée, la preuve
copiée, le remède ; **axes exercés** et **axes non exercés** (un « 0 finding » sans eux ne compte pas) ; décompte par sévérité
recompté depuis la liste.
