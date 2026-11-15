# Story 36-17 : l'assistant de configuration ne laisse plus de page blanche à la fin — « Configuration en cours… », « Configuration terminée », *Réessayer* (#1187)

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-17`, branche
`story/36-17-assistant-fin-configuration`, par le workflow `bmad-create-story`. Release **0.18.0** (phase 2), branche de
release `epic/36-0.18.0`. **Aucune migration**, **aucun changement du serveur ni du contrat de l'API** : frontend, clés
i18n (quatre locales) et manuels. Code lu à la tête du worktree, **`0bf3c3b4b`** (kickoff de la 0.18.0). Ce qui fait foi :
fichier d'epic, Phase 2, rang 16 ; registre `e36-choix-autonomes-e360.md`, **C-e360-18** (et C-e360-3, -11) ; choix propres
à la story : `e36-choix-autonomes-36-17.md` (C-36-17-1 à C-36-17-8). Voie de développement : **voie 2** (bases `kesh_e3617`,
`kesh_e2e_e3617` sur `kesh-mariadb-dev2:3307` ; `kesh_ci_e3617` en voie 1 pour `gate-ci.sh`). Journaux :
`/home/gcorbaz/devel/kesh-gate-logs/e36-17/`. Validation non lancée.*

## Story

En tant que personne qui installe Kesh et termine l'assistant de configuration (chemin « configuration réelle » ou
« données de démonstration »), je veux voir à chaque instant un écran qui dit où j'en suis — la configuration en cours, la
configuration terminée avec ce qui a été créé et ce qui me reste à faire, ou l'échec avec un bouton pour réessayer —, afin
de ne jamais rester devant une page blanche ni devoir recharger la page pour entrer dans Kesh.

## Ce qui se lit au code (`0bf3c3b4b`)

### La cause de la page blanche : deux états de l'assistant n'ont aucun écran

- Le serveur fait passer l'assistant à l'**étape 7** quand le compte bancaire est saisi ou passé
  (`crates/kesh-api/src/routes/onboarding.rs:598` « `/// POST /api/v1/onboarding/bank-account — step 6→7` », `:838`
  « `/// POST /api/v1/onboarding/skip-bank — step 6→7 without creating bank account` »), puis à l'**étape 8** à la
  finalisation (`:1078` « `"UPDATE onboarding_state SET step_completed = 8, version = version + 1 \` »).
- La page `frontend/src/routes/onboarding/+page.svelte` n'a d'écran que pour les étapes 0 à 6 : la chaîne `{#if}` commence
  à `:320` (« `{#if onboardingState.loading && !onboardingState.loaded}` », le chargement initial seulement), sa dernière
  branche est `:478` (« `{:else if onboardingState.stepCompleted === 6}` ») et elle se ferme à `:567` (« `{/if}` ») **sans
  `{:else}`**. Aux étapes 7 et 8, la page ne rend rien ; le cadre `routes/onboarding/+layout.svelte` ne garde que le logo
  (`:10-12`, « `<a href="/" class="text-3xl font-semibold text-primary">Kesh</a>` ») et le pied de page (`:37-39`). C'est
  la « page blanche, avec seulement le logo » de #1187.
- **La finalisation tourne sur cette page vide** : `submitBankAccount` (`:268-298`) et `handleSkipBank` (`:300-315`)
  enchaînent l'étape (qui passe l'état à 7) puis `await onboardingState.finalize()` (`:293`, `:310`). Pendant tout l'appel,
  rien ne s'affiche : la branche de chargement exige `!onboardingState.loaded` (`:320`), vrai depuis longtemps. La
  finalisation est une transaction sous verrous, rejouée en cas d'interblocage (`onboarding.rs:894`
  « `crate::retry::retry_app_on_deadlock("onboarding::finalize", || {` »), qui pose les réglages de facturation (`:985`),
  les quatre taux de TVA (`:1018`) et l'exercice de l'année (`:1046`) : sur le NAS d'un utilisateur, sa durée n'est pas
  négligeable, et un rechargement pendant l'appel est le geste naturel devant une page vide.
- **Succès** : `navigateHomeIfFinalized()` (`:255-266`) appelle `goto('/')` (`:257`) **sans l'attendre** ni traiter son
  échec ; si l'état rendu n'est pas 8, un toast passager (`:259-264`, `onboarding-finalize-incomplete`) et la page reste
  vide.
- **Échec** : `reportFinalizeError` (`:239-250`) n'émet qu'un `toast.error` — passager — et la page reste vide : plus aucun
  bouton (l'écran bancaire a disparu avec l'étape 6). Le store garde son verrou un tour de plus
  (`frontend/src/lib/features/onboarding/onboarding.svelte.ts:153-172`, `setTimeout` à `:166-168`).
- **Rechargement** : la garde de `/onboarding` renvoie à `/` dès l'étape 7, finalisation faite ou non
  (`frontend/src/routes/onboarding/+layout.ts:28-32`, « `// Path B (production) : step >= 7 → onboarding terminé` »,
  « `if (!isDemo && step >= 7) {` »), et la garde de l'application laisse entrer dès l'étape 6
  (`frontend/src/routes/(app)/+layout.ts:42`, « `if (!isDemo && step < 6) {` »). Une installation restée à 7 entre donc dans
  l'application **sans réglages de facturation, sans taux de TVA et sans exercice**, et rien ne le signale : la bannière
  « Configuration incomplète » ne s'affiche qu'à l'étape 6 (`(app)/+layout.svelte:369`, « `… onboardingState.stepCompleted
  >= 6 && onboardingState.stepCompleted < 7}` »).

**Ce qui n'est pas établi** : laquelle de ces branches a frappé l'utilisateur de #1187. Sa finalisation a abouti (comptes
par défaut et journal remplis, dit l'issue) : il a vu soit la page vide pendant l'appel, soit une navigation qui n'a pas
eu lieu ou une réponse perdue après le commit. Le correctif les ferme toutes : chaque état a un écran, et aucune
navigation ne se fait d'office après la finalisation. Le nominal local ne reproduit rien de visible :
`frontend/tests/e2e/onboarding-bank-account.spec.ts:82-83` (« `await page.locator('form button[type="submit"]').click();` »,
« `await expect(page).toHaveURL(/\/$/);` ») passe, mais n'asserte que l'URL.

### Le serveur n'est pas en cause, et ne change pas

- La finalisation rend l'état mis à jour (`onboarding.rs:1133`, « `response_with_stub(pool, updated).await` ») ; elle
  accepte les étapes 7 et 8 (`:955`, « `if onboarding.step_completed < 7 || onboarding.step_completed > 8 {` ») et rend
  l'état sans rien écrire à l'étape 8 (`:966`, « `if onboarding.step_completed == 8 {` ») : *Réessayer* et *Terminer la
  configuration* peuvent rappeler la même route sans risque ; le chemin démo y est refusé (`:949`).
- Une restauration pose l'étape à 8 au moins (`crates/kesh-db/src/backup.rs:930`, « `ON DUPLICATE KEY UPDATE step_completed
  = GREATEST(step_completed, 8), is_demo = FALSE` ») ; le préréglage de test `post-onboarding` la pose à 10
  (`crates/kesh-db/src/test_fixtures.rs:508`) : « terminé » se lit donc `≥ 8`.
- Les routes du récapitulatif existent, ouvertes à tout utilisateur authentifié : `GET /api/v1/companies/current`
  (`crates/kesh-api/src/lib.rs:1160-1163`), qui rend la société et ses comptes bancaires
  (`frontend/src/lib/features/settings/settings.types.ts:84-87`, `fetchCompanyCurrent`, `settings.api.ts:12`), et
  `GET /api/v1/fiscal-years` (`lib.rs:1210-1213`, `listFiscalYears`, `fiscal-years.api.ts:14`).

### Le chemin démo

- `chooseDemoPath` (`+page.svelte:68-77`) attend `seedDemo()` puis `goto('/')` (`:71`) **sans l'attendre**. Pendant le
  chargement, l'écran de l'étape 2 reste, ses boutons désactivés (`:357`, `:361`), sans indicateur. Après lui, l'état est
  « étape 3, démo » : la branche de l'étape 3 (`:367`, « `{:else if onboardingState.stepCompleted === 3}` ») ne regarde pas
  `isDemo` et montre l'écran du **type d'organisation** du chemin production, jusqu'à ce que la navigation aboutisse — ou
  indéfiniment si elle échoue. La garde de `/onboarding` renvoie la démo à `/` dès l'étape 3 au rechargement
  (`+layout.ts:24-27`).

### Les tests existants touchés

- `frontend/src/routes/onboarding/onboarding-motif.test.ts:235-290` attendent le motif de la finalisation **en toast**
  (`expect(toastError).toHaveBeenCalledWith('MOTIF-FIN')`, `… (DEJA)`, `… (ECHEC)`) : à réécrire vers l'écran (D2).
- `onboarding-catch-guard.test.ts:27-28` : la garde ne lit que `+page.svelte`, plancher de **onze** `catch`.
- `frontend/tests/e2e/onboarding-bank-account.spec.ts:83` attend `/` après l'envoi de l'étape bancaire : il passera par la
  page terminée et *Commencer* (D1).
- Les E2E de fin du chemin production `onboarding-path-b.spec.ts:65`, `:92` et `onboarding.spec.ts:150` sont **déjà
  rouges**, KF-029 (#97), par des sélecteurs périmés (`#coord-address`, `onboarding-path-b.spec.ts:73`, champ disparu avec
  l'adresse structurée) — `docs/testing.md:398-402`. Ils ne sont pas un appui ; la story ne les répare pas (hors périmètre),
  mais leur attente de `/` après *Configurer plus tard* devient fausse et se note au commentaire de chacun.

### Inventaire par le symptôme

Symptôme 1 — **un état de l'assistant sans écran**. Détecteur par exécution (règle de `e35-dev-commun.md`) : un test Vitest
qui rend la page pour chaque état `stepCompleted ∈ {−1, 0, …, 10}` × `isDemo ∈ {faux, vrai}` × `loading ∈ {faux, vrai}`
(état chargé) et exige un écran identifié (`data-testid` de la forme `onboarding-screen-*`) ; **sonde positive** : à
`0bf3c3b4b`, (7, faux, faux) et (8, faux, faux) ne rendent rien — le test doit y rougir avant le correctif (journal
`rouges.log`). Par lecture, les sites qui branchent sur l'étape :

```
$ git grep -nE "stepCompleted (===|>=|<|>|<=) [0-9]+" -- frontend/src ':!*.test.ts'
frontend/src/routes/(app)/+layout.svelte:39:  … onboardingState.stepCompleted >= 3) {
frontend/src/routes/(app)/+layout.svelte:369: {:else if … stepCompleted >= 6 && onboardingState.stepCompleted < 7}
frontend/src/routes/onboarding/+page.svelte:31:  onboardingState.stepCompleted <= 2 &&
frontend/src/routes/onboarding/+page.svelte:37:  if (onboardingState.stepCompleted >= 1) {
frontend/src/routes/onboarding/+page.svelte:224:  if (onboardingState.stepCompleted === 6 && !bankLedgerRequested) {
frontend/src/routes/onboarding/+page.svelte:256:  if (onboardingState.stepCompleted === 8) {
frontend/src/routes/onboarding/+page.svelte:325 … :478  (branches 0 à 6)
```

Angle mort de cette commande : elle ne voit pas les gardes écrites `step >= 7` (variable locale) — d'où, à part :
`routes/onboarding/+layout.ts:25`, `:30` et `(app)/+layout.ts:39`, `:42`, lus ci-dessus. Classement : `+page.svelte:325-567`
et `+layout.ts:30` **dans le périmètre** (D3, D4) ; `(app)/+layout.svelte:369` **dans le périmètre** (D5) ;
`(app)/+layout.svelte:39` (Ctrl+N, mode expert, étape ≥ 3) **hors** — raccourci, aucun écran en jeu ;
`+page.svelte:31` (lien de restauration, étapes 0 à 2) et `:37` (chargement des messages) **hors**, inchangés ;
`(app)/+layout.ts:39-44` **hors**, inchangé par décision (D5).

Symptôme 2 — **une navigation lancée sans être attendue** dans l'assistant et ses bannières :

```
$ git grep -nE "goto\(" -- frontend/src/routes/onboarding frontend/src/lib/features/onboarding \
    frontend/src/lib/features/setup frontend/src/lib/shared/components ':!*.test.ts' | /usr/bin/grep -v "await goto"
frontend/src/lib/features/setup/SetupForm.svelte:82:            void goto('/login');
frontend/src/lib/features/setup/setup.api.ts:29: * `await` cette fonction puis `goto('/onboarding')`.
frontend/src/lib/shared/components/DemoBanner.svelte:28:        goto('/onboarding');
frontend/src/routes/onboarding/+page.svelte:71:         goto('/');
frontend/src/routes/onboarding/+page.svelte:257:            goto('/');
```

Sonde positive : la commande rend bien `+page.svelte:257`, le site connu de #1187. Classement : `+page.svelte:257`
**retiré** (D1) ; `+page.svelte:71` **dans le périmètre** (D7) ; `setup.api.ts:29` est un commentaire ;
`SetupForm.svelte:82` **hors** — renvoi au login après un 410, dans un `setTimeout`, voulu (« `void` ») ;
`DemoBanner.svelte:28` **hors** — après la remise à zéro de la démo, l'application reste affichée si la navigation
échoue (pas de page blanche) ; **issue à demander** (« Bannière de démo : la navigation vers l'assistant après la remise à
zéro n'est ni attendue ni traitée en échec », `technical-debt`, P3).

### Les manuels

```
$ git grep -n -i "finalis\|configurer plus tard\|Configuration terminée" docs/manual/fr/user-manual.tex docs/manual/fr/admin-manual.tex
```

- `docs/manual/fr/user-manual.tex:217` : « Dès que vous avez choisi la langue comptable, Kesh met en place le plan
  comptable … À la finalisation, il crée votre premier exercice, l'année civile en cours. Vous êtes alors prêt à saisir vos
  premières écritures. » — rien de l'écran de fin ; **change** (T7). Contrôle du PDF actuel :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'Vous êtes alors prêt à saisir vos premières écritures'` → `1`.
- `user-manual.tex:219` (comptes de TVA non désignés d'office) : reste vrai ; la page terminée le rappelle.
- `docs/manual/fr/admin-manual.tex:1460` (« `\item \textbf{Compte bancaire} principal (optionnel, configurable plus
  tard).` ») et `:1109` (« Le wizard d'onboarding démarre automatiquement (langue, mode, coordonnées de la société,
  banque). ») : une phrase s'ajoute après la liste de `:1449-1461` (T7).

## Décisions

- **D1 — Le chemin production se termine sur la page « Configuration terminée », sans navigation d'office**
  (C-36-17-1). L'écran de l'état « production, étape ≥ 8 » est la page terminée ; *Commencer* est un lien `href="/"` avec
  `data-sveltekit-reload` (chargement complet). `navigateHomeIfFinalized` disparaît.
  *Écartées* : `await goto('/')` (une navigation cliente qui échoue redemande un écran) ; une navigation après délai.
- **D2 — L'échec de la finalisation s'affiche à l'écran, avec *Réessayer*, sans toast** (C-36-17-2). Motif par
  `errorMessageOf(err, msg('onboarding-finalize-error', …))`, région `role="alert"` de l'écran de l'étape 7 ; *Réessayer*
  rappelle `onboardingState.finalize()`. Sur `ONBOARDING_STEP_ALREADY_COMPLETED`, la page relit l'état
  (`onboardingState.fetchState()`) et montre l'écran de l'état relu ; une relecture en échec s'affiche de même. Les clés
  `onboarding-already-finalized` et `onboarding-finalize-incomplete` (« Veuillez recharger la page ») ne sont plus lues :
  retirées des quatre locales si `git grep` n'en trouve plus aucun lecteur.
  *Écartées* : toast et écran (message doublé) ; toast seul.
- **D3 — L'écran de l'étape 7 a trois états** (C-36-17-3) : **en cours** — « Configuration en cours… » sur toute la zone de
  l'assistant (`role="status"`, `aria-busy="true"`), tant que `onboardingState.loading` ; **échec** — le motif et
  *Réessayer* ; **repris** (après un rechargement, ni appel ni échec) — « La configuration n'est pas terminée. » et
  *Terminer la configuration*, même geste que *Réessayer*. **Aucun `POST` au chargement.**
  *Écartée* : finaliser d'office au chargement de la page.
- **D4 — Gardes de `/onboarding`** (C-36-17-4) : production à l'étape 7 → écran de reprise (plus de renvoi à `/`) ; à
  l'étape ≥ 8 → page terminée. Démo à l'étape ≥ 3 → `/`, inchangé. `/onboarding/restore` à l'étape > 2 renvoie toujours à
  `/onboarding` (`restore/+page.ts:32-33`), qui montre désormais l'écran de l'état au lieu de renvoyer à `/`.
  *Écartée* : renvoi à `/` à l'étape ≥ 8 (la page terminée invisible au rechargement, contraire à #1187, point 2).
- **D5 — L'application reste ouverte à l'étape 7 ; la bannière « Configuration incomplète » s'y affiche** (C-36-17-5) :
  condition `stepCompleted >= 6 && stepCompleted < 8` à `(app)/+layout.svelte:369` ; son lien mène à `/onboarding`, donc à
  l'écran de reprise. La garde `(app)/+layout.ts` ne change pas.
  *Écartée* : renvoyer l'application à l'assistant à l'étape 7 (une finalisation refusée en 422, `onboarding.rs:991`,
  enfermerait l'utilisateur hors des écrans qui la réparent).
- **D6 — Le récapitulatif** (C-36-17-6), lu à l'affichage de la page terminée par `fetchCompanyCurrent()` et
  `listFiscalYears()` :
  - *Créé* : la société (nom) ; le plan comptable (type d'organisation, par les clés existantes `onboarding-org-independant`,
    `-association`, `-pme`, et langue comptable) ; le ou les exercices (nom et période, dates suisses par
    `formatSwissDate`, `frontend/src/lib/features/reports/reports.api.ts:238`) ; chaque compte bancaire (nom de la banque,
    description s'il y en a une ; **pas d'IBAN** à l'écran).
  - *À faire* : toujours, « Si vous êtes assujetti à la TVA, désignez les comptes de TVA dans Administration →
    Facturation avant votre première facture avec TVA. » (`user-manual.tex:219`) ; sans compte bancaire, « Ajoutez votre
    compte bancaire dans Administration → Comptes bancaires. »
  - Lecture en échec : une ligne le dit, avec le motif (`errorMessageOf`) ; le titre, le *À faire* fixe et *Commencer*
    restent.
  *Écartée* : une route neuve de récapitulatif (le serveur ne change pas).
- **D7 — Chemin démo** (C-36-17-7) : « Chargement des données de démonstration… » sur toute la zone pendant `seedDemo()`
  (`role="status"`) ; ensuite `await goto('/')` dans son propre `try` (son `catch` passe par `errorMessageOf`) ; l'état
  « démo, étape ≥ 3 » a son écran — « Les données de démonstration sont prêtes. » et *Commencer* (même lien qu'en D1) ;
  l'écran du type d'organisation exige `!isDemo`.
  *Écartée* : une page terminée obligatoire en démo.
- **D8 — Structure** (C-36-17-8) : les écrans neufs sont des composants **présentationnels** sous
  `frontend/src/lib/features/onboarding/` (sans appel d'API ni `catch`) ; chargement du récapitulatif, finalisation et
  navigation démo, avec leurs `catch`, restent dans `+page.svelte`, sous la garde `onboarding-catch-guard.test.ts`, dont le
  plancher (`:28`) monte au nombre recompté. Un `{:else}` final rend un écran de repli (« État de l'assistant inattendu »,
  lien *Recharger la page* vers `/onboarding` avec `data-sveltekit-reload`) : plus aucune combinaison sans écran. Les
  libellés des langues (`+page.svelte:328` et l'étape 5) se partagent par une constante, non recopiés.
  *Écartée* : étendre la garde aux composants.

## Critères d'acceptation

1. **AC1 — Fin nominale, sans rechargement.** Sur une installation neuve, l'assistant joué au navigateur jusqu'à l'étape
   bancaire, puis *Enregistrer* (compte saisi) ou *Configurer plus tard* : pendant la finalisation, l'écran « Configuration
   en cours… » est visible (`data-testid="onboarding-screen-finalizing"`) ; ensuite, sans rechargement ni navigation,
   l'URL reste `/onboarding` et la page « Configuration terminée » s'affiche (`onboarding-screen-done`) avec *Commencer*.
2. **AC2 — *Commencer*.** Le clic charge `/` et l'application s'affiche (menu latéral présent) ; aucune bannière
   « Configuration incomplète » à l'étape 8.
3. **AC3 — Récapitulatif.** La page terminée nomme la société saisie, le type d'organisation et la langue comptable,
   l'exercice créé (nom et période en dates suisses) et, s'il a été saisi, le compte bancaire (banque, description), sans
   IBAN ; elle porte la phrase des comptes de TVA, et, sans compte bancaire, celle de son ajout. Si `companies/current` ou
   `fiscal-years` échoue, une ligne le dit avec le motif ; titre, *À faire* fixe et *Commencer* restent.
4. **AC4 — Échec et *Réessayer*.** Une finalisation refusée (motif du serveur) ou interrompue (réseau) laisse l'écran de
   l'étape 7 avec le motif (`role="alert"`) — ou, sans motif lisible, « Finalisation échouée — réessayez. » — et
   *Réessayer* ; aucun toast. *Réessayer* rappelle la finalisation ; à son succès, la page terminée s'affiche. Sur
   `ONBOARDING_STEP_ALREADY_COMPLETED`, l'état est relu et son écran s'affiche.
5. **AC5 — Rechargement.** `/onboarding` rechargé à l'étape 7 (production) montre l'écran de reprise et *Terminer la
   configuration*, qui finalise et mène à la page terminée ; aucun `POST` n'est envoyé au chargement. Rechargé à
   l'étape ≥ 8, il montre la page terminée et son récapitulatif. La démo à l'étape ≥ 3 est toujours renvoyée à `/`.
6. **AC6 — L'application à l'étape 7** montre la bannière « Configuration incomplète », dont le lien mène à l'écran de
   reprise ; aux étapes < 6 et ≥ 8, la garde et la bannière se comportent comme avant.
7. **AC7 — Un écran pour chaque état.** Pour tout `stepCompleted` de −1 à 10, `isDemo` vrai ou faux, chargement en cours
   ou non (état chargé), la page rend exactement un écran identifié `onboarding-screen-*` ; l'écran du type
   d'organisation n'apparaît jamais en démo.
8. **AC8 — Chemin démo.** Pendant le chargement des données de démonstration, « Chargement des données de
   démonstration… » occupe la zone de l'assistant ; à son succès, `/` s'ouvre ; si la navigation échoue, l'écran « Les
   données de démonstration sont prêtes » et son *Commencer* restent, avec le motif.
9. **AC9 — Textes.** Chaque texte neuf a sa clé `onboarding-*` dans les quatre locales (`fr-CH`, `de-CH`, `it-CH`,
   `en-CH`), son repli français égal à la valeur `fr-CH` ; `parity_between_locales` (`crates/kesh-i18n/src/loader.rs:771`)
   et `npm run lint-i18n-ownership` verts ; les deux clés mortes de D2 retirées (ou leur lecteur restant cité).
10. **AC10 — Manuels.** `user-manual.tex` (§ *Onboarding*, après `:217`) décrit l'indicateur, la page terminée et son
    récapitulatif, *Commencer*, l'échec et *Réessayer*, la reprise au rechargement, l'indicateur de la démo ;
    `admin-manual.tex` dit en une phrase, après `:1461`, que l'assistant se termine sur la page « Configuration terminée ».
    PDF régénérés à l'intégration (contrôle aplati, forme E6, de la phrase neuve).
11. **AC11 — Périmètre serveur nul.** `git diff --stat origin/epic/36-0.18.0 -- crates/ ':!crates/kesh-i18n/locales'` ne
    rend rien ; aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 91).
12. **AC12 — Recette visuelle** : captures, thème clair, base de démonstration jetable, de chaque état touché — en cours,
    échec avec *Réessayer*, reprise, terminée avec compte bancaire, terminée sans compte bancaire, récapitulatif en échec,
    chargement de la démo, démo prête, écran de repli, application à l'étape 7 avec la bannière — dans
    `/home/gcorbaz/devel/kesh-gate-logs/e36-17/captures/`, jointes au Dev Agent Record et au prompt de la revue.

## Tests à écrire, et la preuve du rouge

Écrits d'abord, sur le code de `0bf3c3b4b`, journal `/home/gcorbaz/devel/kesh-gate-logs/e36-17/rouges.log` ; puis verts
au correctif. Le code se commite avant toute mutation (E5) ; scripts de mutation dans `kesh-gate-logs/e36-17/mut/` ;
restauration par `git restore --worktree -- <fichier>` puis `touch`.

**Vitest** (fichiers à côté de la page, nom sans préfixe `+`, patron de `onboarding-bank-step.test.ts:20-60` : API de
l'assistant, i18n, `$app/navigation`, `svelte-sonner` simulés) :

- `onboarding-screens.test.ts` — **le détecteur d'AC7** : la matrice des états, un écran identifié pour chacun. Rouge au
  départ sur (7, production) et (8, production), et sur (3, démo) qui rend `onboarding-org-type-*` (sonde positive).
- `onboarding-end.test.ts` :
  - AC1 : *Configurer plus tard* avec `finalize` en attente (promesse non résolue) → `onboarding-screen-finalizing` ;
    résolue à l'étape 8 → `onboarding-screen-done`, `goto` jamais appelé. Rouge au départ (rien rendu, `goto('/')`
    appelé).
  - AC3 : récapitulatif (société, type, langue, exercice en date suisse, banque et description, pas d'IBAN — la chaîne
    `CH93` absente du rendu) ; sans compte bancaire, la phrase d'ajout ; `fetchCompanyCurrent` en échec → la ligne et le
    motif, *Commencer* présent.
  - AC4 : refus `VALIDATION_ERROR` → motif à l'écran, `toastError` non appelé ; `TypeError` → « Finalisation échouée —
    réessayez. » ; *Réessayer* → `finalize` appelé une seconde fois → page terminée ; `ONBOARDING_STEP_ALREADY_COMPLETED`
    → `fetchState` appelé, écran de l'état relu.
  - AC5 : état chargé à 7 sans appel → écran de reprise, `finalize` **non** appelé au rendu ; *Terminer la configuration*
    → appelé.
  - AC8 : `seedDemo` en attente → indicateur de la démo ; `goto` rejeté → écran « démo prête » et motif.
- `onboarding-motif.test.ts:235-290` : réécrit — le motif lu à l'écran, `toastError` non appelé (D2). Rouge au départ par
  construction (le toast est l'état d'aujourd'hui).
- `onboarding-catch-guard.test.ts` : plancher monté au nombre recompté (le recompte se cite au Change Log).
- `onboarding-guard.test.ts` (neuf) — la fonction `load` de `routes/onboarding/+layout.ts` : production 7 et 8 → pas de
  `redirect` ; démo 3 → `redirect(302, '/')` ; non authentifié → `/login`. Rouge au départ sur 7 et 8.
- Bannière (AC6) : un test du rendu de `(app)/+layout.svelte` ou, à défaut, de la condition extraite en fonction pure
  (`showIncompleteBanner(state)`) dans `features/onboarding/` : 5 → non, 6 → oui, 7 → oui, 8 → non, démo → non. Rouge au
  départ sur 7.

**E2E** (le défaut se voit dans le navigateur : exigé) — `frontend/tests/e2e/onboarding-fin.spec.ts`, préréglage `fresh`,
connexion `changeme`, **assistant joué à l'interface** jusqu'à l'étape bancaire (sélecteurs `id` et `data-testid`, aucun
sélecteur de texte traduit ; champs de coordonnées `#coord-name`, `#coord-street`, `#coord-npa`, `#coord-city`,
`+page.svelte:414-452`), **aucun `page.reload()` ni `page.goto()` entre le clic bancaire et l'assertion de la page
terminée** :

- **E1 — compte saisi** : `onboarding-screen-done` visible, URL `/onboarding`, récapitulatif (société, banque) ; *Commencer*
  → `/`, menu latéral visible ; contrôle par `authedApiContext` que `GET /api/v1/onboarding/state` rend l'étape 8.
- **E2 — compte passé, finalisation lente puis interrompue** : `page.route('**/api/v1/onboarding/finalize', …)` retarde la
  première requête (l'indicateur se constate) puis l'interrompt (`route.abort()`) → message et *Réessayer* ; la route
  rendue au serveur, *Réessayer* → page terminée, sans compte bancaire, avec la phrase d'ajout.
- **E3 — reprise** : étapes jouées par l'API jusqu'à `skip-bank` (étape 7, sans finalisation), puis `page.goto('/onboarding')`
  → écran de reprise (et non `/`) ; *Terminer la configuration* → page terminée ; `page.reload()` → page terminée de
  nouveau.
- **E4 — démo** : chemin démo avec `seed-demo` retardé par `page.route` → indicateur, puis `/`.

Preuve du rouge de l'E2E : la spec jouée contre le frontend construit au code d'avant le correctif (`git stash` du seul
frontend, ou construction à `0bf3c3b4b`) — E1 rouge à l'attente de `onboarding-screen-done` — journal
`kesh-gate-logs/e36-17/e2e-rouge.log`. `onboarding-bank-account.spec.ts:83` adapté (page terminée, puis *Commencer*).

**Mutations** (journal `kesh-gate-logs/e36-17/mut/*.log`) : **M1** branche de l'étape 7 retirée (AC7, AC1) ; **M2**
`goto('/')` d'office rétabli après la finalisation (AC1 : `goto` non appelé) ; **M3** retour au toast dans
`reportFinalizeError` (AC4) ; **M4** finalisation d'office au rendu de l'étape 7 (AC5 : `finalize` non appelé au rendu) ;
**M5** garde `step >= 7 → '/'` rétablie (AC5, `onboarding-guard.test.ts`) ; **M6** condition de bannière `< 7` rétablie
(AC6) ; **M7** l'écran du type d'organisation sans `!isDemo` (AC7) ; **M8** IBAN ajouté au récapitulatif (AC3) ; **M9**
`fetchState` retiré du cas « déjà finalisé » (AC4). Une mutation équivalente se dit.

## Tâches

- [ ] **T0 — Étape 0** : relire le dernier rapport de validation ; recontrôler au code les lignes citées ci-dessus à la tête
  de la branche (elle a pu bouger : 36-7, 36-15 en parallèle), ligne lue citée au Change Log.
- [ ] **T1 — Rouges** : les tests de la section précédente, écrits et journalisés rouges (`rouges.log`), sonde positive du
  détecteur comprise.
- [ ] **T2 — Écrans** (D3, D6, D7, D8) : composants présentationnels sous `frontend/src/lib/features/onboarding/`
  (en cours, échec/reprise de l'étape 7, terminée avec récapitulatif, démo en chargement, démo prête, repli) ;
  `data-testid="onboarding-screen-*"`.
- [ ] **T3 — Page** (D1, D2, D3, D7, D8) : `+page.svelte` — branches 7, ≥ 8, démo ≥ 3, `{:else}` ; `navigateHomeIfFinalized`
  retiré ; `reportFinalizeError` vers l'état d'écran ; *Réessayer* / *Terminer la configuration* ; chargement du
  récapitulatif ; `chooseDemoPath` attend `goto` ; plancher de la garde recompté.
- [ ] **T4 — Gardes et bannière** (D4, D5) : `routes/onboarding/+layout.ts` ; `(app)/+layout.svelte:369`.
- [ ] **T5 — Textes** (AC9) : clés `onboarding-*` neuves aux quatre `messages.ftl` ; clés mortes retirées ; replis égaux à
  `fr-CH`.
- [ ] **T6 — E2E** : `onboarding-fin.spec.ts` (E1 à E4) ; `onboarding-bank-account.spec.ts:83` adapté ; une ligne de
  commentaire à `onboarding-path-b.spec.ts:65`, `:92` et `onboarding.spec.ts:150` : leur fin passe désormais par
  *Commencer* (KF-029 inchangée).
- [ ] **T7 — Manuels** (AC10) : `user-manual.tex` § *Onboarding* ; `admin-manual.tex` après `:1461`. PDF à l'intégration.
- [ ] **T8 — Captures** (AC12), backend lancé pour elles, sur `kesh_e2e_e3617` recréée.
- [ ] **T9 — Mutations** M1 à M9, après commit du code.
- [ ] **T10 — Gates** (voir Dev Notes) ; Dev Agent Record ; `Status: review` et clé `review` dans le même commit ; prompt
  de la revue P1 (trois lentilles B, E, A ; manuel, PDF et captures nommés ; axes exercés et non exercés déclarés).

## Note de version

*Corrigé* (C-e360-11 ; l'orchestrateur l'assemble au CHANGELOG à l'intégration) :

- **Assistant de configuration : plus de page blanche à la fin.** Après l'étape du compte bancaire, l'assistant affiche
  « Configuration en cours… » pendant la mise en place, puis la page « Configuration terminée », qui rappelle ce qui a été
  créé (société, plan comptable, exercice, compte bancaire s'il a été saisi) et ce qui reste à faire (les comptes de TVA si
  vous êtes assujetti, le compte bancaire si vous l'avez passé) ; le bouton *Commencer* ouvre Kesh. Si la mise en place
  échoue, son motif reste affiché avec *Réessayer* ; si vous rechargez la page avant la fin, l'assistant vous propose de
  terminer la configuration au lieu de vous laisser entrer dans Kesh sans exercice ni réglages de facturation, et la
  bannière « Configuration incomplète » le rappelle dans l'application. Le chargement des données de démonstration affiche
  lui aussi un indicateur. (refs #1187)

## Dev Notes

### Hors périmètre, et les issues demandées

- `DemoBanner.svelte:28` — navigation non attendue après la remise à zéro de la démo : **issue à ouvrir** (inventaire,
  symptôme 2 ; `technical-debt`, P3).
- KF-029 (#97) : `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:150` restent rouges (sélecteurs périmés) ; la
  story n'y ajoute qu'un commentaire.
- Le nom de la société non modifiable après l'assistant (#1097) : hors epic, sans clé.
- Le serveur : aucune ligne. Si l'étape 0 trouvait une cause côté serveur, la story s'arrête et le dit (périmètre changé).

### Fichiers communs avec les autres stories de la phase 2

| Fichier | Autres stories | Règle |
|---|---|---|
| `crates/kesh-i18n/locales/*/messages.ftl` | 36-15 (vocabulaire « compte analytique », quatre locales), 36-18 | union à l'intégration ; clés `onboarding-*` seulement |
| `docs/manual/fr/user-manual.tex`, `admin-manual.tex` | 36-15, 36-10, 36-9 | union ; § *Onboarding* et `sec:onboarding-config` seulement |
| `frontend/src/routes/(app)/+layout.svelte` | 36-15 (libellés du menu, possibles) | union ; une condition à `:369` |

### Modules touchés (règle de splitting)

1. `frontend/src/routes/onboarding/` (page, garde) ; 2. `frontend/src/lib/features/onboarding/` (écrans) ;
3. `frontend/src/routes/(app)/+layout.svelte` (une condition) ; 4. `crates/kesh-i18n/locales/` (textes) ;
5. `docs/manual/fr/`. **Cinq** : à la limite, pas au-delà — pas de découpage. Les E2E accompagnent le module 1.

### Gates (voie 2 ; `e35-dev-commun.md` § *Quels gates, quand*)

- **En cours** : `cargo fmt --check`, `clippy -D warnings` (workspace) ; `nextest -E 'package(kesh-i18n) |
  binary(textes_coherents)'` (les locales sont lues par ces deux binaires) ; Vitest ciblé des fichiers de l'assistant ;
  `kesh-db` n'est pas touché.
- **Passage en `review`** : bases remises à neuf ; `scripts/test-fast.sh --ci` en voie 2
  (`DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e3617`, `TMPDIR=/tmp/kesh-voie2`, verrou
  `.gate-base2.lock`) ; `scripts/gate-ci.sh kesh_ci_e3617` **en voie 1** ; frontend complet (`check`,
  `lint-i18n-ownership`, `test:unit`, `build`). Pas d'E2E.
- **Remédiation** : `test-fast --ci` et frontend complet. **Clôture** : `gate-ci` et E2E complet sur `kesh_e2e_e3617`
  recréée (voie 2, port attribué par l'orchestrateur), au dernier commit de code ; échecs nommés contre `docs/testing.md`
  (les sept de la KF-029). Une commande lourde à la fois (E7).

### Ce qui doit rester vrai

- Les étapes 0 à 6 de l'assistant, leurs motifs de refus (`onboarding-motif.test.ts`, hors finalisation) et le lien de
  restauration (étapes 0 à 2) sont inchangés.
- La finalisation n'est appelée que par un geste (envoi de l'étape bancaire, *Configurer plus tard*, *Réessayer*,
  *Terminer la configuration*).
- Le récapitulatif n'affiche aucun IBAN ; aucune donnée réelle dans les tests, captures ou journaux (fixtures
  synthétiques : « Exemple Banque SA », IBAN de démonstration de la suite).
- Le chemin démo : rechargé à l'étape ≥ 3, toujours `/`.

### Références

- Issue #1187 ; fichier d'epic, Phase 2, rang 16 ; C-e360-18 ; registre `e36-choix-autonomes-36-17.md`.
- `CLAUDE.md` § *Recette visuelle* ; `docs/testing.md` § *Les échecs attendus* ; fiche modèle
  `35-7-autocomplete-compte-hors-liste.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — **Création** (Opus 5.5, `bmad-create-story`), code lu à `0bf3c3b4b`. Cause établie au code : aucun écran
  pour les étapes 7 et 8 (`+page.svelte:320-567`), finalisation sans indicateur, `goto('/')` non attendu (`:257`), échec
  en toast passager, garde de `/onboarding` qui renvoie à `/` dès l'étape 7 (`+layout.ts:30`) ; serveur hors de cause.
  Choix C-36-17-1 à C-36-17-8. Une issue à ouvrir (`DemoBanner.svelte:28`). Cinq modules, sans migration.
