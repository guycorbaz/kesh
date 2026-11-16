# Prompt — revue de code P1 de la Story 36-17 (#1187), trois lentilles, première passe complète

Tu relis la **Story 36-17** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.18.0**,
Epic 36, phase 2 : **l'assistant de configuration ne laisse plus de page blanche à la fin**. Avant la story, la page
`/onboarding` n'avait aucun écran aux étapes 7 et 8 : la finalisation tournait sur une page vide, le succès naviguait par un
`goto('/')` non attendu, l'échec n'était qu'un toast, et la garde renvoyait à `/` dès l'étape 7, finalisation faite ou non.
Désormais un écran pour chaque état (`onboardingScreenFor`) : « Configuration en cours… », page « Configuration terminée » et
son récapitulatif, échec affiché avec *Réessayer* (phrase de remède avant le motif sous `VALIDATION_ERROR`, limite #1200),
reprise au rechargement sans finalisation d'office, état non lu, repli, chemin démo ; bannière « Configuration incomplète » aux
étapes 6 et 7, au texte neuf. **Frontend, clés i18n (quatre locales) et manuels seulement ; aucun changement du serveur ; aucune
migration.**

Worktree `/home/gcorbaz/devel/kesh-36-17`, branche `story/36-17-assistant-fin-configuration`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-17 diff 327c7edbd 074453a9f` (`327c7edbd` = fiche validée ; `074453a9f` = dernier commit
de code ; `e3cbe6c7d` ne porte que l'étape 0 de la fiche, le commit de passage en revue que le compte rendu). Langue du rapport :
français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-17-assistant-fin-configuration.md` en entier (D1 à D9, AC1 à AC12, tests, mutations
M1 à M17, Dev Notes, **Change Log** et **Dev Agent Record**) ; les registres `e36-choix-autonomes-e360.md` (C-e360-18, -3, -11)
et `e36-choix-autonomes-36-17.md` (C-36-17-1 à 14 ; 11 à 14 écrits au développement). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec
les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-17/` : `rouges.log` (et
`rouges-vitest-brut.log`), `vitest-cible-*.log`, `nextest-cible-*.log`, `front-*.log`, `test-fast-ci.log`, `gate-ci.log`,
`gates-review.log`, `mut/resultats.log` et `mut/M*.log`, `captures.log`, `captures-2.log`, `captures-3.log`. Les scripts qui
les ont produits sont à côté (`reset-db-e3617.sh`, `captures-36-17.sh`, `captures-36-17.cjs`, `gates-review-36-17.sh`,
`mut/mutations-36-17.mjs`) : lis-les, ne les lance pas. **Captures** (thème clair) sous
`/home/gcorbaz/devel/kesh-gate-logs/e36-17/captures/` (`01-en-cours.png` à `12-application-etape-7-banniere.png`) — regarde-les si
ton outil lit les images, et compare chacune à l'état qu'elle prétend montrer.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle
  part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se
  **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`,
  `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` (seule forme permise du contrôle de PDF, E6).
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **La correspondance état → écran** (`frontend/src/lib/features/onboarding/onboarding-screens.ts`, `onboardingScreenFor`) contre
   D8 : non lu ; étape négative ou non entière ; 0 à 2 avec et sans `demoLoading` ; démo ≥ 3 ; production 3 à 6 ; production 7
   et la précédence échec > en cours > reprise ; ≥ 8 (9, 10). Une combinaison sans écran ? Un écran du type d'organisation en démo ?
   `demoLoading` lu hors des étapes 0 à 2, `finalizing` ou l'erreur hors de l'étape 7 ?
2. **La finalisation** (`+page.svelte` : `submitBankAccount`, `handleSkipBank`, `runFinalize`, `reportFinalizeError`,
   `finalizeDisabled`) : ordre du geste (effacer l'erreur, poser `finalizing`, appeler) ; `finalizing` posé avant l'étape bancaire,
   remis à faux à son échec ; aucune fenêtre où l'écran de reprise se rend entre `skipBank` et `finalize` ; le verrou d'un tour du
   store (`onboarding.svelte.ts`, `setTimeout(0)`) et le court-circuit `if (_loading) return` de `finalize()` — un clic avalé
   sans rien dire est-il encore possible ? `ONBOARDING_STEP_ALREADY_COMPLETED` → relecture ; relecture en échec. Un refus de
   finalisation qui arriverait à une étape autre que 7 (état rendu par l'étape bancaire ≠ 7) : que voit l'utilisateur ?
   `VALIDATION_ERROR` reconnu au code, jamais au texte ; phrase de remède avant le motif.
3. **Aucune navigation d'office ni aucun `POST` au chargement** : `goto` n'est plus appelé qu'au chemin démo, attendu, dans son
   `try` ; *Commencer* est un lien `href="/"` à `data-sveltekit-reload` ; l'écran de reprise ne finalise qu'au clic. Inventaire par
   le symptôme : `git grep -nE "goto\(" -- frontend/src/routes/onboarding frontend/src/lib/features/onboarding` et
   `git grep -n "finalize()" -- frontend/src`.
4. **Les gardes et la bannière** : `routes/onboarding/+layout.ts` (production : plus aucun renvoi ; démo ≥ 3 → `/`) ;
   `(app)/+layout.svelte` par `showIncompleteBanner` (étapes 6 et 7, état lu, hors démo) ; `(app)/+layout.ts` inchangée ;
   `onboarding/restore/+page.ts` qui renvoie à `/onboarding` à l'étape > 2 (que voit-on désormais ?). Une installation restaurée
   (étape 8) ou au préréglage `post-onboarding` (10) qui ouvre `/onboarding` voit la page terminée : est-ce juste ?
5. **Le récapitulatif** (`loadSummary`, `OnboardingDone.svelte`) : **aucun IBAN** n'atteint l'écran (le type `OnboardingSummary`
   le prouve-t-il ?), carte de crédit (`accountKind`) ; exercices triés, dates suisses ; lecture en échec partielle (une route sur
   deux) ; lecture rejouée ou non à un second passage à l'écran `done` ; phrase « compte bancaire à ajouter » seulement si la
   lecture a réussi sans compte.
6. **Les textes** : les quatre `messages.ftl` (clés neuves `onboarding-*`, `incomplete-banner-text` réécrite, deux clés mortes
   retirées) ; justesse des traductions DE/IT/EN, des chemins de menu (« Administration → Facturation » et leurs équivalents) ;
   replis égaux à `fr-CH` (`onboarding-fin-i18n.test.ts`, liste fermée — clé oubliée ?) ; `lint-i18n-ownership` (les clés de la
   bannière passées en props à l'écran de reprise, C-36-17-11) ; l'accessibilité (`role="status"`, `aria-busy`, `role="alert"`).
7. **Le manuel contre le code** : `docs/manual/fr/user-manual.tex` (§ *Onboarding*, paragraphe « La fin de l'assistant » et
   l'encadré *Limite connue (#1200)*) et `admin-manual.tex` (après la liste de `sec:onboarding-config`) — chaque phrase est-elle
   vraie au code (par exemple « il ne la relance jamais de lui-même », « Rechargée une fois la configuration terminée, la page de
   l'assistant montre de nouveau la page Configuration terminée », la liste des écrans déclencheurs) ? **PDF** : non régénérés
   dans la story (AC10 : à l'intégration) — vérifie que le PDF actuel ne porte PAS encore la phrase neuve, par la forme E6 :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'La fin de l'"'"'assistant'`, et
   nomme-le comme point d'intégration, non comme défaut.
8. **Les tests et les témoins** : le détecteur d'AC7 (`onboarding-screens.test.ts`, plus `onboarding-unloaded.test.ts` pour l'état
   non lu — l'ordre imposé dans un seul `it` est-il sûr ?) ; la sonde positive (`rouges.log`) ; `onboarding-end.test.ts` (le test de
   la fenêtre par `MutationObserver` prouve-t-il quelque chose — M10 ?) ; `onboarding-guard.test.ts` (simulacre `browser: true`) ;
   `onboarding-motif.test.ts` réécrit (attendus retouchés : motivés ?) ; la spec E2E `onboarding-fin.spec.ts` (sélecteurs par
   `data-testid` seulement, aucun `reload`/`goto` entre le clic bancaire et la page terminée en E1/E2, `page.route` sans délai
   fixe) — **elle n'a pas tourné** au passage en revue (E2E à la clôture, consigne de l'Epic 35) : relis-la comme du code non
   exécuté ; les commentaires des specs KF-029 (lignes des tests inchangées, `docs/testing.md` § *Les échecs attendus*). Un test
   qui passe **à vide** ? Les mutations M1 à M17 (`mut/resultats.log`) : chacune tuée par le test qu'elle vise ? Propose des
   mutations plausibles non jouées.
9. **Le périmètre** : `git diff --stat origin/epic/36-0.18.0...074453a9f -- crates/ ':!crates/kesh-i18n/locales'`
   vide (AC11) ; aucun `CHANGELOG.md` ; la note de version de la fiche et ses deux alinéas exclusifs (#1200, 36-19).

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne la page, les cinq composants neufs, `onboarding-screens.ts`, la garde, la bannière, les tests et la spec E2E.
Cherche : un état sans écran ou à deux écrans ; un drapeau qui reste levé (un `finally` manquant, un `return` qui saute une remise
à zéro) ; une course entre un appel en vol et un démontage ; un `$effect` qui relance une lecture en boucle ; un texte en dur hors
i18n ; un test qui passe **à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

Double clic sur *Configurer plus tard* ; *Réessayer* pendant le verrou d'un tour ; finalisation qui réussit mais rend l'étape 7
(instantané périmé) ; `fetchState` de « déjà finalisé » qui rend 6, 7, 8 ou échoue ; récapitulatif sans exercice, à deux
exercices, compte bancaire sans description, carte de crédit ; démo : `seedDemo` rejeté (toast, retour à l'écran 2 ?), `goto`
rejeté, rechargement à l'étape 3 démo ; état non lu puis *Réessayer* qui rend 7 ; Administrateur et autres rôles (le lien de
restauration) ; langue d'interface autre que FR (replis, `loadI18nMessages` au montage seulement si étape ≥ 1 — la page
terminée d'une installation restaurée est-elle traduite ?). Chaque cas : comportement au code, test qui le couvre (ou absence,
prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC12 : livré, testé, au lieu prescrit ? L'étape 0 (six LOW de la P3, lignes lues citées au Change Log) ;
la clôture de la validation écrite ; les rouges de départ et la sonde ; M1 à M17 ; les gates du passage en revue (`test-fast --ci`
en voie 2, `gate-ci` en voie 1, frontend complet) — chaque chiffre se retrouve-t-il au journal, horodaté après le dernier commit de
code ? Ce qui n'a pas tourné (E2E, PDF) est-il dit comme tel ? Les écarts déclarés (fichiers de test ajoutés ou scindés par
rapport à la fiche, identifiants `data-testid` ajoutés aux étapes 0 à 6, script de captures hors dépôt) sont-ils exacts et sans
effet sur le livré ? Les captures montrent-elles chacune l'état annoncé (AC12, douze états) ? Le Dev Agent Record n'affirme-t-il
que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e36-17/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte
final **recompté depuis la liste**. Un « 0 finding » non adossé à cette liste ne compte pas comme passe.
