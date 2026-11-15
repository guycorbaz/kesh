# Prompt — revue de code P1 de la Story 36-5 (#931, KF-056), trois lentilles, première passe complète

Tu relis la **Story 36-5** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.8**, Epic 36,
phase 1 : **le compte rendu d'une restauration ne se perd plus sous un battement de session refusé**. L'import complet invalide la
session au serveur ; un battement d'activité (`POST /api/v1/auth/activity`) parti avant la réponse rendait 401, le renouvellement
échouait, et le chemin 401 du client effaçait la session et menait à `/login?reason=session_expired` avant que l'Administrateur ait lu
le compte rendu. Le correctif est **entièrement frontend** : un **maintien de session** porté par `authState`
(`holdForRestore`/`releaseRestoreHold`/`restoreHold`, D1), lu par le chemin 401 du client (`refreshTokens`, D1) et par le module de
session (`session-idle.svelte.ts`, D2), posé, réarmé, borné et levé par le **seul** panneau de restauration (`AdminRestorePanel.svelte`,
D3) ; `expireSession()` extraite d'`api-client.ts` ; `importHoldMs`/`RESTORE_HOLD_MARGIN_MS` dans `admin-restore.api.ts`. **Aucune
migration, aucun changement serveur ni d'API, aucune clé i18n.**

Worktree `/home/gcorbaz/devel/kesh-36-5`, branche `story/36-5-restauration-compte-rendu`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-5 diff f43562b4d HEAD` (`f43562b4d` = la fiche validée ; `58a9af582` = l'E2E déterministe
et l'étape 0 ; `29762b89a` = l'API du maintien **inerte** et les tests, rouges ; `69f18b23a` = le correctif, **dernier commit qui
touche du code de production** ; `fe40d7bcc` puis `0e9da291c` = l'E2E de D5 récrit (C-36-5-13), **dernier commit qui touche du
code** ; `f4e6afd57` = `docs/testing.md`, le manuel, le registre ; le dernier commit = la fiche, le `sprint-status.yaml` et ce
prompt). Le code de production seul : `git diff f43562b4d 69f18b23a -- frontend/src ':!*.test.ts'`. Langue du rapport : français. Modèle : **Sonnet ou Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-5-restauration-compte-rendu.md` en entier (*Ce qui se lit au code*, inventaire, D1 à D6,
AC1 à AC9, mutations M1 à M23, *Dev Agent Record*, *Change Log*) ; les registres `e36-choix-autonomes-e360.md` (C-e360-8, -11) et
`e36-choix-autonomes-36-5.md` (C-36-5-1 à -9, et ceux ajoutés au développement). Ne prends pas le Dev Agent Record pour argent
comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-5/` : `rouges.log`,
`rouges-vitest.log`, `verts-vitest.log`, `repetition-avant.log`, `repetition-avant-d5.log`, `-d5-r2.log`, `-d5-r3.log`, `repetition-apres.log`,
`repetition-apres-d5.log`, `-d5-r2.log`, `test-results-avant-d5/`, `test-results-avant-d5-r3/`, `backend-*.log`, `mut/resume.log`
et `mut/M*.log` (diff de chaque mutation et tests qui la tuent), `mut/mutations.py` (à **lire**), `e2e-e36-5.sh` (à **lire**),
`check-*.log`, `lint-i18n-*.log`, `front-*.log`, `test-fast-ci.log`, `gate-ci.log`.

⚠️ **L'E2E complet n'a pas encore tourné** (il se joue à la clôture de la story, après la revue ; consigne de l'Epic 36). Les deux
répétitions (D4) et le test déterministe (D5) ont tourné, ainsi que `test-fast --ci` et `gate-ci` au passage en `review`. Le dire
n'est pas un finding ; affirmer le contraire au Dev Agent Record en serait un.

**Capture** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/captures/j-compte-rendu-apres-battement-refuse.png` — le compte
rendu resté affiché après le battement forcé ; regarde-la si ton outil lit les images. **Manuel** : `docs/manual/fr/admin-manual.tex`,
paragraphe *Ce que dit l'écran à la fin de l'import* (la demi-phrase « Laissé ouvert, ou quitté, l'écran mène à la connexion… »,
D6). ⚠️ **Le PDF n'est pas régénéré dans la story** (fiche, D6 : à l'intégration, après l'union avec la 36-4, qui touche le même
manuel) : contrôle `docs/manual/fr/admin-manual.pdf` sous la seule forme permise et attends **`1`** pour « Prenez le temps de lire,
puis cliquez » et **`0`** pour la phrase neuve — un `1` pour la phrase neuve, ou un PDF modifié sur la branche, serait un écart :
`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`,
  `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport. **Tout chemin
  cité** se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`), ligne lue citée.

## Les axes obligatoires, pour chaque lentille

1. **Le chemin 401 (D1)** : `refreshTokens` et `expireSession` (`frontend/src/lib/shared/utils/api-client.ts`). Sous maintien, le
   renouvellement est-il **toujours** tenté, la requête rejouée s'il réussit (le même `FormData` pour `postFormData`), et seuls
   l'effacement et la redirection suspendus s'il échoue ? Hors maintien, le comportement est-il identique octet pour octet à
   `f43562b4d` (les tests 401 existants passent-ils sans changement de corps ni d'assertion — compare `git diff f43562b4d 69f18b23a --
   frontend/src/lib/shared/utils/api-client.test.ts`) ? `requestRaw` est-il couvert par la même garde ? Le mutex (`sharedRefresh`)
   et `tryRefresh` (sans effacement) sont-ils inchangés ?
2. **L'état du maintien (D1)** : `auth.svelte.ts`. `restoreHold` vaut-il exactement `_restoreHoldUntil !== null && Date.now() <
   _restoreHoldUntil` ? `holdForRestore` **remplace**-t-il l'échéance ? Levée par `logout()` **en première instruction** (avant le
   `await fetch`), par `clearSession()`, par `releaseRestoreHold()` — et **jamais** par `login()` (que `doRefresh` appelle) ? Un autre
   chemin vers une connexion réelle (`login/+page.svelte`, `setup.api.ts`, la diffusion `auth-change` et `hydrate()`) peut-il hériter
   d'un maintien dans le même onglet ? Lis la fiche, *Hors périmètre*, avant de conclure.
3. **Le module de session (D2)** : `session-idle.svelte.ts`, `tick`, `recordActivity`, `stayActive`. Avant `last + w`, sous maintien,
   tout est-il d'aujourd'hui (battement, avertissement, *Rester connecté*) ? À partir de `last + w`, sous maintien : ni fermeture, ni
   écriture d'activité, ni avertissement (`warningRemainingMs` à `null`), et la fermeture au tic qui suit la levée ou l'échéance du
   maintien ? Sans stockage partagé (`shared` faux), rien ne change ? Le commentaire de `sendHeartbeat` et l'en-tête (« aucune
   suspension ») disent-ils vrai ?
4. **Le panneau (D3)** : `AdminRestorePanel.svelte`, `confirmImport`, `armReleaseOnNextGesture`, `endSession`, `onDestroy`. Ordre :
   maintien posé **avant** `uploadFullImport`, avant tout `await` ; au succès, réarmé à B, `sessionIdle.stop()`, minuterie de B qui
   lève puis appelle `expireSession` ; échec certain et issue inconnue : levée au **premier geste qui suit le message**, écouteur retiré
   après usage et au démontage, jamais armé pendant l'import ; 401 de l'import : levée puis `expireSession`, aucun message ;
   démontage : levée, et `expireSession` en phase 2 seulement, jamais si `reconnecting` ; réponse arrivée après le démontage : rien
   d'armé (`destroyed`). La phase 2 est-elle bien identifiée par `readingTimer !== null` — un cas où l'une est vraie sans l'autre ?
   `keepHold` et le `finally` : une exception du `catch` laisse-t-elle toujours une levée prévue ? B se calcule-t-il en un seul point
   (`readingMs`), et la borne de la phase 1 n'est-elle écrite **qu'une fois** (`importHoldMs`) ?
5. **Les tests (AC1 à AC6)** : chaque test prouve-t-il ce que son nom dit, ou passe-t-il à vide ? Les simulacres
   (`AdminRestorePanel.test.ts` : délégation au **vrai** `authState` chargé par `vi.importActual`, simulacres partiels d'`api-client`
   et d'`admin-restore.api`, `sessionIdle` simulé ; `session-idle.svelte.test.ts` : `restoreHold` réglable) peuvent-ils masquer un
   défaut ? Les minuteries simulées (`toFake: ['setTimeout', 'clearTimeout', 'Date']`, `setInterval` réel pour testing-library)
   laissent-elles une dérive de temps qui rendrait un test flou (AC5 bis (b) à `B − 1`) ? L'état de module du maintien fuit-il d'un
   test à l'autre ? Les **33 mutations** (`mut/resume.log`) : chacune est-elle tuée **par le test que la fiche nomme**, et le diff de
   chaque `mut/M*.log` est-il bien la mutation décrite (M5″, M7″, M10′, M11′, M12, M12′, M23 surtout) ?
6. **L'E2E déterministe (D5)** : le test « KF-056 » de `frontend/tests/e2e/backup-restore-new-install.spec.ts:381`. Il retient la
   réponse de l'import **dans la page** (`window.fetch` enveloppé), non par `page.route` + `route.fetch()` comme D5 l'écrivait
   (C-36-5-13 : la requête rejouée arrivait illisible, 400 — et le premier rouge « avant » a d'abord été pris à tort pour celui
   de la KF-056, `rouges.log`) : l'enveloppe est-elle sûre (la réponse retenue est-elle bien celle d'un import abouti au serveur,
   session invalidée ; le statut de l'import et du battement sont-ils contrôlés avant l'affichage ; la libération après une
   navigation est-elle sans effet) ? L'attente d'1,1 s précède-t-elle chaque jeu de mouvements ? Le rouge avant le correctif
   (`repetition-avant-d5-r3.log`, `test-results-avant-d5-r3/`, `backend-avant-d5-r3.log`) est-il bien celui de la KF-056 ? La
   seconde partie prouve-t-elle une **absence** (compteur posé avant les mouvements, trois tics) — et tuerait-elle M8 en E2E
   (non joué) ? `:140` et `:262` n'ont-ils pas bougé ?
7. **La preuve par répétition (D4, AC7)** : `repetition-avant.log` (60 exécutions, fréquence observée de chaque test, commit et
   frontend servi nommés — le « avant » sert le build d'un **export** du commit `29762b89a`, comportement d'avant le correctif : est-ce
   équivalent ? C-36-5-10) et `repetition-apres.log` (60) avec `repetition-apres-d5.log` (le test de D5 dix fois) : zéro rouge ? Le Dev Agent Record en tire-t-il ce qu'ils établissent, et pas plus ?
8. **Le manuel et `docs/testing.md` (D6, AC8)** : la demi-phrase du `.tex` dit-elle ce que fait le code (« au bout de la durée
   d'inactivité réglée » — B — et « aussitôt » quand on quitte l'écran ; et l'assistant `/onboarding/restore`, que la phrase couvre
   aussi) ? Le paragraphe de l'issue inconnue (`:2342`) reste-t-il juste (levée au premier geste) ? La note KF-056 de `docs/testing.md`
   est-elle placée avant `⇒ **Le compte attendu…**`, sans casser la référence de la note KF-052 (« Les deux paragraphes suivants ») ?
   Dit-elle ce qui n'est pas établi ?
9. **Inventaire (fiche, § Inventaire par le symptôme)** : rejoue les deux greps et celui des minuteries. Un site **neuf** qui quitte
   l'écran de restauration sans geste, apparu avec la story (`expireSession` y compris), est-il classé ?
10. **`textes_coherents.rs`** (`crates/kesh-api/tests/textes_coherents.rs`, lit `frontend/src`) : un commentaire ou un texte neuf des
    fichiers touchés le fait-il rougir (lis les gardes ; recoupe avec `test-fast-ci.log` et `gate-ci.log`) ?
11. **Le compte rendu** : chaque chiffre du Dev Agent Record (tests ajoutés, rouges, mutations, gates) se recompte-t-il depuis la
    source, avec son périmètre ? Chaque gate cite-t-il son journal et son commit ? `git log origin/main..HEAD --format=%B |
    /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` est-il vide ?

## Les trois lentilles

- **B — Blind Hunter** : le diff aplati seul, sans la fiche d'abord ; ce qui est faux, fragile ou incohérent dans le code tel qu'il est.
- **E — Edge Case Hunter** : chaque branche et chaque borne (échéance exacte, `null`, démontage à chaque phase, deux onglets, import
  long, renouvellement pendant l'import, réponse tardive, exception dans le `catch`, stockage refusé), contre le code.
- **A — Acceptance Auditor** : chaque AC (AC1 à AC9) et chaque mutation de la fiche contre ce qui a été livré et exécuté ; chaque
  écart entre la fiche, le registre, le Dev Agent Record et les journaux.

## Rapport

Un rapport par lentille, et **seulement** là :

- B : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/review-p1-B.md`
- E : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/review-p1-E.md`
- A : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/review-p1-A.md`

Chacun porte : l'en-tête (lentille, modèle, commit relu, date) ; les findings numérotés (`B1-1`, `E1-1`, `A1-1`…), chacun avec sa
**sévérité** (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), son lieu (`fichier:ligne`), la **ligne lue citée**, la preuve (sortie copiée), le
remède proposé ; la liste des **axes exercés** et celle des **axes non exercés** (un « 0 finding » sans elle ne compte pas) ; le
décompte des findings par sévérité, recompté depuis la liste.
