# Prompt — revue de code P2 de la Story 32-1b, passe ciblée, une lentille (regression hunter), Opus

Tu relis **la seule remédiation de la revue de code P1** de la clé **32-1b** de la Story 32-1 de Kesh (comptabilité
suisse) : *l'écran de la sauvegarde automatique*, constats de **#711**. Worktree `/home/gcorbaz/devel/kesh-32-1b`,
branche `story/32-1b`. La passe P1 (Sonnet, trois lentilles ; rapports
`/home/gcorbaz/devel/kesh-gate-logs/321b-review-p1-{B,E,A}.md`) a rendu **B 0/0/0/3, E 0/0/1/4, A 0/0/0/2**. Le seul
finding au-dessus de LOW, **E1 (MEDIUM)** : `formUnchanged` (`frontend/src/lib/features/admin-backup/AutoBackupSection.svelte`)
compare `enabled`, mais aucun test ne le distinguait, ni le cas « saisie modifiée puis remise à l'origine ». **A1
(LOW)** : un décompte de journaux de mutation (« dix » pour onze). Les autres LOW ne sont pas traités (ils seront
tracés par l'orchestrateur) et **ne sont pas l'objet de cette passe**.

Le diff à relire, **aplati**, est celui de la remédiation seule (le commit de test et le commit de documentation) :
`git -C /home/gcorbaz/devel/kesh-32-1b diff dae40211...HEAD`. **Aucune ligne de production n'a changé** : la
remédiation ajoute deux tests (iv) et (v) au bloc E-6 de `AutoBackupSection.test.ts`, joue la mutation **M-b12**
(retrait de `enabled === formBase.enabled` dans `formUnchanged`), annote deux décomptes, ajoute une entrée au Change
Log de la fiche `_bmad-output/implementation-artifacts/32-1-defauts-courts.md` et ce prompt. Langue du rapport :
français. Modèle : **Opus**. Lentille : **regression hunter** — *la sévérité se déplace vers ce qu'on vient d'écrire*.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans le scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`), pas même vers
  un fichier « jetable » : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` (dont `321b-e2e.sh`, `321b-p1rem-e2e.sh` et `321b-mut.sh`) ou du scratchpad, ni `make`, ni
  `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris), ni `npm`/`npx`/`node`/`vitest`/`playwright`, ni
  `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh` qui
  écrit ; `gh issue view 711|725|727` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`.
- **Toute affirmation d'absence** (« le cas X n'est pas testé », « l'assertion Y manque ») s'appuie sur une sortie
  **`grep -nF`** copiée ; toute affirmation de présence, sur une sortie `grep -nF` ou `sed -n` copiée.

## Axes

1. **Les tests (iv) et (v) prouvent-ils ce qu'ils annoncent ?** Lis `formUnchanged`, `resetForm`, `applyState`,
   `runNow` et `save` dans `AutoBackupSection.svelte`, puis les deux tests.
   - (iv) : seule la case « Activée » est modifiée ; le run rend un réglage dont `enabled` égale la base, avec une
     `version` et une rétention différentes. Si `formUnchanged` ignorait `enabled`, le formulaire suivrait-il le run
     (case recochée, rétention 9, version 4) ? Chaque assertion de (iv) est-elle **discriminante** — une valeur
     inchangée ou un défaut de la fixture (`schedule()` : `enabled: true`, `intervalHours: 24`, `retentionCount: 7`,
     `version: 3`) pourrait-il la satisfaire à vide ?
   - (v) : la case cliquée deux fois et la fréquence 12 puis 24 rendent-elles bien la saisie égale à sa base, **en
     nombres** (`bind:value` d'un `<select>` peut rendre une chaîne) ? Les assertions (case cochée, rétention 9,
     `version: 4` à l'envoi) établissent-elles que le formulaire **et** sa base ont suivi ? Une mutation plausible
     (comparaison stricte sans `Number(...)`, `resetForm` qui ne pose pas `formBase`) serait-elle tuée par (v), ou
     (v) ne tue-t-il rien ? Dis-le, sans l'exiger : (v) documente un comportement.
   - L'attente asynchrone (`lancer()` attend que le bouton se réactive) suffit-elle avant les assertions ? Les files
     `mockResolvedValue` et `afterEach(vi.resetAllMocks)` / `beforeEach(vi.clearAllMocks)` : une fuite d'un test à
     l'autre est-elle possible ?
2. **La mutation M-b12** : `/home/gcorbaz/devel/kesh-gate-logs/321b-p1rem-mut-M-b12.{diff,log}` — le diff est-il
   exactement le retrait annoncé, le rouge est-il **(iv)** et lui seul, la restauration par copie de sauvegarde
   rend-elle un diff résiduel nul, le contrôle après restauration est-il vert ?
3. **Les annotations (A1)** : la fiche (Dev Agent Record, sous-section 32-1b) et `321b-review-prompt-p1.md` — ajout
   daté sans réécriture de l'entrée d'origine ? Le nombre écrit (onze) se recompte-t-il
   (`ls /home/gcorbaz/devel/kesh-gate-logs/ | grep -c '^321b-mut-.*\.log$'`) ?
4. **L'entrée de Change Log « 32-1b, revue de code P1 »** : trend, modèles, sort de chaque finding (E1 traité, A1
   annoté, B1-B3, E2-E5 et A2 non traités et listés), gates déclarés — chaque chiffre contre son journal
   (`321b-p1rem-cible.log`, `321b-p1rem-frontend-unit.log` : 1937 + 2 = 1939 attendu, `321b-p1rem-frontend-build.log`,
   `321b-p1rem-e2e.log` et sa révision) ; aucune entrée passée réécrite (`git diff dae40211...HEAD --
   _bmad-output/implementation-artifacts/32-1-defauts-courts.md` : ne doit montrer que des ajouts).
5. **E2E au dernier commit de code** : la révision du journal E2E égale-t-elle le commit de test ; les commits
   postérieurs ne touchent-ils que de la documentation (`git show --stat`) ? Échecs jugés fichier par fichier contre
   `docs/testing.md` § *Les échecs attendus*.
6. **Messages de commit** : `git log dae40211..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?)
   #[0-9]'` ne doit rien rendre ; chaque commit porte `refs #711`.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/321b-review-p2.md` :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `R1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`, `sed -n` ou
  `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** ; un « 0 finding » sans cette liste ne
  compte pas comme passe ;
- rien d'autre n'est écrit nulle part.
