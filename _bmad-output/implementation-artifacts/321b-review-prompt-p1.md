# Prompt — revue de code P1 de la Story 32-1b (l'écran de la sauvegarde automatique), trois lentilles, Sonnet

Tu relis le développement de la clé **32-1b** de la Story 32-1 de Kesh (comptabilité suisse) : *l'écran de la
sauvegarde automatique* — les constats de **#711** traités côté frontend et manuel : **E-6** (une saisie non
enregistrée survit à « Sauvegarder maintenant » sans défaire le verrou optimiste : `formBase`, `applyState`),
**E-7** (une réponse qui manque relit l'état : `runNow` en trois cas, prédicat `isUnknownOutcome` extrait
d'`AdminRestorePanel.svelte`), **E-8** (l'état partagé du bandeau garde le dernier état lu et ignore une réponse
tardive), **L-B5** (le bandeau lit `lastSuccessInFuture`, servi par la 32-1d déjà fusionnée), **L-B6** (la limite
d'import dite en Mio et en unité décimale), **L-B3** (le manuel : le seuil d'espace ne mesure pas la base). Worktree
`/home/gcorbaz/devel/kesh-32-1b`, branche `story/32-1b`, partie de `main` à `b3f52733` (les 32-1a, 32-1c et 32-1d y sont
fusionnées). Le diff à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-1b diff b3f52733...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-1-defauts-courts.md` — **partie B** (AC-b1 à AC-b11, AC-b6 renvoyé à la
32-1d ; tâches T-b0 à T-b7), § *Dev Notes* (*Pourquoi quatre clés*, *Contrôle aplati d'un PDF*, *32-1b — remarques de
conception*, *Inventaire des sites non résolus*, *Fichiers touchés*, CHANGELOG), et la sous-section **32-1b** du Dev
Agent Record — et le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-321.md` (C-32-1-1 à
C-32-1-48, et **C-32-1-49 à C-32-1-56** du développement). Le Dev Agent Record déclare des gates, des rouges et des
mutations : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/321b-*.log`. **Regarde les captures** `/home/gcorbaz/devel/kesh-gate-logs/321-captures/321b-*.png`
(lecture seule ; une image se regarde avec l'outil de lecture de fichiers) : chaque état dit-il ce que l'AC-b9 annonce,
et ce que le manuel en écrit ?

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans le scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`), pas même vers
  un fichier « jetable » : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` (dont `321b-e2e.sh` et `321b-mut.sh`) ou du scratchpad, ni `make`, ni `latexmk`, ni `cargo` (aucune
  sous-commande, `cargo fmt` compris), ni `npm`/`npx`/`node`/`vitest`/`playwright`, ni `docker`, ni `mysql`/`mariadb`,
  ni `sqlx`, ni `curl`, ni `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh` qui écrit ;
  `gh issue view 711|725|727` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`, et le contrôle aplati d'un PDF :
  `pdftotext <f>.pdf - | tr '\n' ' ' | sed "s/’/'/g" | tr -s ' '` (sortie standard seulement) — un PDF de la base se lit
  par `git show b3f52733:docs/manual/fr/admin-manual.pdf | pdftotext - - | …`.
- **Toute affirmation d'absence** (« le site X n'est pas corrigé », « le test Y manque ») s'appuie sur une sortie
  **`grep -nF`** copiée ; toute affirmation de présence, sur une sortie `grep -nF` ou `sed -n` copiée.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `frontend/src/lib/features/admin-backup/AutoBackupSection.svelte` : `resetForm`, `formUnchanged` (comparaison en
   nombres des champs liés), `apply`, `applyState` (garde sur une réponse non objet **avant** toute écriture ; ordre
   « calculer `unchanged`, puis poser l'état »), `loadErrorText`, `reread`, `runNow` (trois branches ; `loadError`
   remis à `null` au départ ; `err.status >= 500` dans la branche `ApiError`) ; `save` envoie `formBase.version`. Un
   chemin pose-t-il `schedule` à une valeur non objet, ou laisse-t-il `formBase` nul alors que le panneau s'affiche ?
   La mention « trop grande » : arguments `mib` et `size`.
2. `frontend/src/lib/shared/utils/unknown-outcome.ts` : import de `./api-error` (non d'`./api-client`) ; règle
   identique à l'ancienne `issueInconnue` ? `AdminRestorePanel.svelte` : l'alias garde-t-il **exactement** le
   comportement de l'import (deux appels) ?
3. `frontend/src/lib/features/admin-backup/backup-schedule.svelte.ts` : le compteur de séquence — une lecture lancée
   avant un `setBackupScheduleState` puis rendue après est-elle ignorée ? deux lectures concurrentes ? un échec garde-t-il
   l'état ? Les quatre doc-comments (module, getter, en-tête du bandeau, en-tête de son test) disent-ils le même
   comportement ?
4. `BackupScheduleBanner.svelte` : la branche `lastSuccessInFuture` (dans `kind === 'overdue'` seulement) ; la date.
5. `backup-schedule.api.ts` : `lastSuccessInFuture: boolean` requis ; fixtures typées complétées partout où
   `BackupSchedule` est construit (`grep -rn 'overdue:' frontend/src frontend/tests`).
6. Les quatre catalogues : deux clés neuves, une modifiée (`$size`) — traductions justes, graphie suisse, même jeu de
   variables, repli Svelte égal au fr-CH à l'octet.
7. Les tests : `AutoBackupSection.test.ts` (blocs E-6, E-7, mention ; `afterEach(vi.resetAllMocks)`),
   `BackupScheduleBanner.test.ts`, `unknown-outcome.test.ts`, `i18n-keys.test.ts` (`sitesTotal` 2332 → 2334),
   `i18n-libelle-en-dur.test.ts` (70 → 71) — un test passerait-il à vide (assertion satisfaite par une valeur
   inchangée, attente asynchrone insuffisante) ?
8. `frontend/tests/e2e/backup-schedule.spec.ts` : la saisie gardée (AC-b10), le 504 intercepté et la relecture, le
   bandeau de l'horloge reculée, la mention neuve ; les captures t, u, v.
9. `CHANGELOG.md` : la ligne neuve sous `### Corrigé` du bloc `## [0.16.0] — Non publié` — vraie contre le code ?

## Lentille E — Edge cases

1. E-6 : saisie modifiée puis remise à sa valeur d'origine avant le run (le formulaire suit-il alors le réglage
   rendu ?) ; case « Activée » seule modifiée ; un run qui rend une `version` différente sans saisie (le formulaire et
   `formBase` suivent — l'enregistrement suivant part-il avec la version neuve ?) ; « Recharger » après un 409.
2. E-7 : un 503 (le client ne rejoue pas un `POST`) ; un 500 dont le corps n'est pas du JSON ; un rejet pendant la
   relecture de la **liste** seule ; un double clic pendant la relecture (`running` couvre-t-il `reread`) ; un 401 (le
   client redirige-t-il avant `runNow` ?) — lis `frontend/src/lib/shared/utils/api-client.ts`, n'exécute rien.
3. E-8 : le bandeau monté avant toute lecture réussie, puis échec ; `setBackupScheduleState(null)` (posé par qui ?)
   après un état lu ; le sondage de 30 min et l'action du panneau qui se croisent.
4. L-B6 : `importMaxMib` à 0, très grand ; une locale dont `Intl` rend une autre espace (U+202F en fr-CH, observé).
5. **Mutations** (journaux `321b-mut-*.log`) : M-b1, M-b2 (et `M-b2-test-renforce` — pourquoi deux journaux ?), M-b3,
   M-b4, M-b7 (deux formes : `M-b7-site`, `M-b7-de-CH`), M-b8, M-b9, M-b10, M-b11 — chacune tuée par le test que la
   fiche nomme ? diff résiduel 0 ? Une mutation plausible non jouée survivrait-elle — par exemple `formUnchanged` qui
   ne compare pas `enabled`, `reread` qui relit l'état mais pas la liste, la séquence incrémentée seulement par
   `refresh` ? Raisonne sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-b1 à AC-b5 et AC-b7 à AC-b11, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la
preuve. Points d'appui :

- **Rouges constatés avant le correctif** : `321b-rouge-avant.log` (seize échecs, dont « liste vide » par fuite d'une
  file `Once`, C-32-1-54) — les cas (i) d'AC-b1, TIMEOUT, 504, 500, `undefined` d'AC-b2, (i) et (ii) d'AC-b3, AC-b4,
  AC-b5 y sont-ils rouges ?
- **AC-b7 — manuel, PDF compris.** Le `.tex` (`docs/manual/fr/admin-manual.tex`, § *Sauvegarde automatique* :
  *Ce que l'écran montre*, *Le bandeau de l'application*, *Espace disque requis*, *Les sauvegardes du serveur*,
  *Horloge du serveur déréglée*) dit-il vrai contre le code — `seuil_d_espace` (`crates/kesh-api/src/auto_backup.rs`),
  `AutoBackupSection.svelte`, `BackupScheduleBanner.svelte` ? **Contrôle le PDF**, aplati, pas seulement le `.tex` :
  chaque phrase neuve y sort-elle (fragment « un cinquième de la taille des documents » nommément) ? `??` : 0 ? U+FFFD :
  0 ? La locution « au moins deux intervalles » est-elle gardée, et aucune tournure « plus de deux intervalles »
  introduite (G22) ? Le PDF commité est-il celui du `.tex` de `HEAD` (comparer une phrase neuve des deux) ? Seul
  `admin-manual.pdf` est-il touché (`git diff --stat b3f52733...HEAD -- docs/manual`) ? Journaux
  `321b-make-admin.log`, `321b-pdf-aplati.log`, `321b-cible-textes.log`.
- **AC-b8** : `grep -c '^admin-backup-' crates/kesh-i18n/locales/*/messages.ftl` à la base (`git show
  b3f52733:<f> | grep -c …`) et à `HEAD` : **+2** par locale ; `sitesTotal` relevé de 2, commentaire daté recompté par
  fichier (`git show b3f52733:<f> | grep -oE '(i18nMsg|[^a-zA-Z]msg)\(' | wc -l` contre `HEAD`) ; parité
  (`321b-cible-i18n.log`) ; gardes frontend (`321b-frontend-*.log`).
- **AC-b9** : quatre captures (a) à (d), et pour (b) et (c) dit **comment** elles ont été obtenues (C-32-1-53).
- **AC-b10** : E2E complet **au dernier commit de code** (compare la révision des journaux à
  `git log --oneline b3f52733..HEAD` : les commits postérieurs ne doivent toucher que de la documentation) ; échecs
  jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* (`321b-e2e.log`) ; gate complet
  (`321b-gate-complet.log`) et ses chiffres.
- **AC-b11** : neuf mutations (dix journaux), diff résiduel 0.
- **Décomptes** : tests ajoutés recomptés aux deux bornes (`git show b3f52733:<f> | grep -cE "^\s*it\(" ` contre
  `HEAD`, boucles `for … it(` comptées à part), périmètre déclaré ; le Vitest complet (`321b-frontend-unit.log`)
  cohérent avec ce décompte (1911 au gate de la 32-1d).
- **Messages de commit** : `git log b3f52733..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?)
  #[0-9]'` doit ne rien rendre ; chaque commit porte `refs #711`.
- **Registre et statut** : la clé `32-1b-defauts-sauvegarde-automatique` de `sprint-status.yaml` est à `review` avec un
  commentaire daté ; le Dev Agent Record n'écrit que la sous-section 32-1b ; registre numéroté à partir de C-32-1-49.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/321b-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — le PDF et les captures compris ; un « 0
  finding » sans cette liste ne compte pas comme passe ;
- rien d'autre n'est écrit nulle part.
