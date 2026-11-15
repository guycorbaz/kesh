# Story 36-17 : l'assistant de configuration ne laisse plus de page blanche à la fin — « Configuration en cours… », « Configuration terminée », *Réessayer* (#1187)

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-17`, branche
`story/36-17-assistant-fin-configuration`, par le workflow `bmad-create-story`. Release **0.18.0** (phase 2), branche de
release `epic/36-0.18.0`. **Aucune migration**, **aucun changement du serveur ni du contrat de l'API** : frontend, clés
i18n (quatre locales) et manuels. Code lu à la tête du worktree, **`0bf3c3b4b`** (kickoff de la 0.18.0). Ce qui fait foi :
fichier d'epic, Phase 2, rang 16 ; registre `e36-choix-autonomes-e360.md`, **C-e360-18** (et C-e360-3, -11) ; choix propres
à la story : `e36-choix-autonomes-36-17.md` (C-36-17-1 à C-36-17-9). Voie de développement : **voie 2** (bases `kesh_e3617`,
`kesh_e2e_e3617` sur `kesh-mariadb-dev2:3307` ; `kesh_ci_e3617` en voie 1 pour `gate-ci.sh`). Journaux :
`/home/gcorbaz/devel/kesh-gate-logs/e36-17/`. Validation : passe P1 remédiée (Change Log), passe suivante à lancer.*

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

### Une limite connue de l'application ouverte avant la finalisation (étapes 6 et 7)

*Ajouté à la remédiation de la validation P1 (V1-1 ; prémisse lue au code, corrigée sur son issue).*

- La garde de l'application laisse entrer dès l'étape 6 (`(app)/+layout.ts:42`, cité plus haut), donc avant la
  finalisation. **Tout** `GET /api/v1/company/invoice-settings` y crée la ligne de réglages **sans comptes** : la route
  (`crates/kesh-api/src/routes/company_invoice_settings.rs:300`, et le `PUT` à `:336`,
  « `company_invoice_settings::get_or_create_default(&state.pool, company.id).await?;` ») appelle
  `get_or_create_default`, qui fait « `INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)` »
  (`crates/kesh-db/src/repositories/company_invoice_settings.rs:129`) ; l'export global aussi
  (`crates/kesh-api/src/exports/global.rs:335`). *(Corrigé à la remédiation de la validation P2, V2-1 : la version
  précédente ne nommait que les réglages de facturation.)* Le frontend lance ce `GET` par `getInvoiceSettings()`
  (`frontend/src/lib/features/invoices/invoices.api.ts:99-100`) depuis **cinq écrans** :
  ```
  $ git grep -n "getInvoiceSettings" -- frontend/src ':!*.test.ts' | /usr/bin/grep -v "export\|import\|//"
  frontend/src/lib/components/invoices/InvoiceForm.svelte:408:      const settings = await getInvoiceSettings();
  frontend/src/lib/components/invoices/InvoiceForm.svelte:620:      const freshSettings = await getInvoiceSettings();
  frontend/src/routes/(app)/bank-accounts/+page.svelte:163:      getInvoiceSettings(),
  frontend/src/routes/(app)/invoices/[id]/+page.svelte:289:      invoiceSettings = await getInvoiceSettings();
  frontend/src/routes/(app)/journal-entries/+page.svelte:160:    void getInvoiceSettings()
  frontend/src/routes/(app)/journal-entries/[id]/+page.svelte:306:        getInvoiceSettings(),
  frontend/src/routes/(app)/settings/invoicing/+page.svelte:193:      const [s, a] = await Promise.all([getInvoiceSettings(), fetchAccounts(true)]);
  frontend/src/routes/(app)/settings/invoicing/+page.svelte:293:          const fresh = await getInvoiceSettings();
  ```
  Ouvrir le journal, les comptes bancaires (ce que la page terminée recommande quand le compte a été passé), une facture
  ou son formulaire, le détail d'une écriture, les réglages de facturation, ou exporter, suffit donc. La validation d'une
  facture, elle, passe par `get_or_create_default_in_tx` (`crates/kesh-db/src/repositories/invoices.rs:2156`) dans une
  transaction annulée par son propre refus (`ConfigurationRequired`, `:2158-2163`) : sans effet durable.
- La finalisation pose ensuite les comptes par un autre `INSERT IGNORE` (`company_invoice_settings.rs:1250`), sans effet
  sur une ligne déjà là ; elle relit alors la ligne **jointe aux comptes de créance et de produit actifs**
  (`:1280-1306`) — une ligne aux comptes vides ne se joint pas — et rend une erreur (`:1305`,
  « `None => Err(DbError::InactiveOrInvalidAccounts),` »), que la route traduit en refus « Impossible de pré-remplir les
  comptes de facturation (1100, 3000 manquants du plan comptable). … » (`onboarding.rs:1008`).
- **Effet** : la finalisation n'aboutit pas « sans comptes » comme le suppose V1-1 : elle est **refusée à chaque essai**,
  avec un motif faux (les comptes 1100 et 3000 existent), tant que les comptes de créance et de produit ne sont pas
  désignés et enregistrés dans *Administration → Facturation* (un `PUT` qui les pose rend la jointure possible, `update`,
  `:323`). Le défaut existe déjà à l'étape 6 ; D5 le garde à l'étape 7, où la finalisation interrompue de #1187 le rend
  plus probable. **Effet résiduel** (V2-3) : sur une ligne neuve, la finalisation pose aussi les comptes créanciers,
  d'arrondi, d'escompte, de frais bancaires et de pertes (`company_invoice_settings.rs:1251-1265`, `.bind(payable)`,
  `.bind(designated.rounding)`…) ; sur la ligne créée avant elle, ils restent vides après le remède, à désigner aussi.
- **La bannière actuelle ne prévient pas** : « `incomplete-banner-text = Configuration incomplète — Terminer la
  configuration` » (`crates/kesh-i18n/locales/fr-CH/messages.ftl:159` ; même sens en `de-CH`, `it-CH`, `en-CH`, `:159`) ne
  dit rien de la facturation.
- Corriger la finalisation (compléter une ligne existante aux comptes vides) touche `kesh-db` et `kesh-api` : hors de la
  story (arbitrage de l'orchestrateur, cinq modules) ; **#1200** (Dev Notes), **corrigée dans la même release 0.18.0 par
  une story dédiée, 36-19** (arbitrage de l'orchestrateur sur V2-1 de la validation P2). La 36-17 **n'en dépend pas** :
  elle écrit la limite — bannière, écran de reprise, écran d'échec, manuels (D5) — de façon à rester juste si la 36-19
  glissait ; la 36-19 fusionnée, la limite disparaît et ces textes se relisent à l'intégration (Dev Notes).

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
- Les E2E de fin du chemin production `onboarding-path-b.spec.ts:65` (*Enregistrer*), `:92` (*Configurer plus tard*) et
  `onboarding.spec.ts:150` (*Configurer plus tard*) sont **déjà rouges**, KF-029 (#97) — `docs/testing.md:398-402` —, par
  un sélecteur périmé, `#coord-address` (`onboarding-path-b.spec.ts:73`, `:106` ; `onboarding.spec.ts:168`), champ disparu
  avec l'adresse structurée. Ils ne sont pas un appui ; la story ne les répare pas (hors périmètre), mais leur attente de
  `/` après l'étape bancaire devient fausse et se note au commentaire de chacun.
- `onboarding-path-b.spec.ts:27` ne tourne pas : `test.fixme` à `:32`, « `KF-032 (#124) — bannière config-incomplète
  jamais affichée après finalize` » (#124 est fermée). Il attend la bannière après *Configurer plus tard* ; avec D5 elle
  s'affiche à l'étape 7, mais la finalisation mène à l'étape 8, où elle ne s'affiche pas : son attente reste fausse, le
  `fixme` reste, un commentaire le dit (T6).

### Inventaire par le symptôme

Symptôme 1 — **un état de l'assistant sans écran**. Détecteur par exécution (règle de `e35-dev-commun.md`) : un test Vitest
qui rend la page pour chaque état `stepCompleted ∈ {−1, 0, …, 10}` × `isDemo ∈ {faux, vrai}` × `loading ∈ {faux, vrai}`,
l'état chargé, plus l'état **non chargé** (`loaded` faux, `loading` vrai ou faux), et exige exactement un écran identifié
(`data-testid` de la forme `onboarding-screen-*`, posé par la story sur les écrans neufs **et** sur les branches 0 à 6
existantes, D8). Aucun identifiant de cette forme n'existe à `0bf3c3b4b` (`git grep onboarding-screen frontend/src` →
vide) : avant le correctif, le détecteur rougit sur tous les états ; sa **sonde positive** se lit donc sur des cas qui
rougissent **pour la raison visée** et non faute d'identifiant — (7, production) et (8, production) ne rendent **aucun**
élément dans la zone de l'assistant (assertion « zone non vide », écrite avec la matrice), et (3, démo) rend
`onboarding-org-type-*` (journal `rouges.log`). Par lecture, les sites qui branchent sur l'étape :

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

Même classe, autre forme (V1-4) : **un état jamais lu**. La garde de `/onboarding` avale l'échec de `fetchState()`
(`routes/onboarding/+layout.ts:33-38`, « `console.error('[onboarding layout] fetchState failed:', err);` ») ; l'état
reste celui du module (`onboarding.svelte.ts:14-21`, `stepCompleted: 0`, `_loaded = false`), et la page rend alors
l'écran de l'étape 0 (choix de la langue) pour une installation peut-être à l'étape 7 : **dans le périmètre** (D9).

Symptôme 2 — **une navigation lancée sans être attendue** dans l'assistant et ses bannières :

```
$ git grep -nE "goto\(" -- frontend/src/routes/onboarding frontend/src/lib/features/onboarding \
    frontend/src/lib/features/setup frontend/src/lib/shared/components ':!*.test.ts' | /usr/bin/grep -v "await goto"
frontend/src/lib/features/setup/SetupForm.svelte:27:  // `goto('/login')` s'exécute hors contexte → navigation surprenante.
frontend/src/lib/features/setup/SetupForm.svelte:82:            void goto('/login');
frontend/src/lib/features/setup/setup.api.ts:29: * `await` cette fonction puis `goto('/onboarding')`.
frontend/src/lib/shared/components/DemoBanner.svelte:28:        goto('/onboarding');
frontend/src/routes/onboarding/+page.svelte:71:         goto('/');
frontend/src/routes/onboarding/+page.svelte:257:            goto('/');
```

Sonde positive : la commande rend bien `+page.svelte:257`, le site connu de #1187. Classement : `+page.svelte:257`
**retiré** (D1) ; `+page.svelte:71` **dans le périmètre** (D7) ; `setup.api.ts:29` et `SetupForm.svelte:27` sont des
commentaires (la ligne `:27` manquait à la sortie citée jusqu'à la validation P2, V2-6) ;
`SetupForm.svelte:82` **hors** — renvoi au login après un 410, dans un `setTimeout`, voulu (« `void` ») ;
`DemoBanner.svelte:28` **hors** — après la remise à zéro de la démo, l'application reste affichée si la navigation
échoue (pas de page blanche) ; porté par **#1199** (« Bannière de démo : la navigation vers l'assistant après la remise à
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
  l'assistant (`role="status"`, `aria-busy="true"`) ; **échec** — le motif et *Réessayer* ; **repris** (après un
  rechargement, ni appel ni échec) — « Configuration incomplète : terminez-la avant d'utiliser Kesh. » (même phrase et
  même clé que la bannière, D5) et
  *Terminer la configuration*, même geste que *Réessayer*. **Aucun `POST` au chargement.**
  **Ordre du geste** (V2-4) : *Réessayer* et *Terminer la configuration* **effacent d'abord l'erreur**, puis posent
  `finalizing`, puis appellent `finalize()` : pendant la relance, « Configuration en cours… » s'affiche, non l'écran
  d'échec. `finalizing` et l'erreur de finalisation **ne se lisent qu'à l'étape 7** : posé à l'étape 6 avant
  `setBankAccount` / `skipBank`, `finalizing` laisse l'écran bancaire en place (boutons désactivés, comme aujourd'hui)
  jusqu'au passage à l'étape 7 ; un refus de l'étape bancaire elle-même reste un toast sur l'écran bancaire (inchangé) et
  remet `finalizing` à faux.
  L'état « en cours » se lit sur un **drapeau local de la page**, `finalizing`, et non sur `onboardingState.loading`
  (V1-3) : posé à vrai **avant** l'appel de l'étape bancaire (`setBankAccount` / `skipBank`) et avant tout rappel de
  `finalize()`, remis à faux dans le `finally` de la finalisation, ou à l'échec de l'étape bancaire. Sans lui, l'état
  (7, chargé, `loading` faux) existe entre la fin de `skipBank()` (`onboarding.svelte.ts:144-150`, `finally`) et le
  `_loading = true` de `finalize()` (`:160`) : l'écran de reprise s'y rendrait un instant. **Précédence** : échec
  > en cours > repris. *Réessayer* et *Terminer la configuration* sont **désactivés** tant que `finalizing` est vrai ou que
  `onboardingState.loading` l'est (le verrou d'un tour de `finalize()`, `:166-168`, avalerait sinon le clic sans rien
  dire, `:159`).
  *Écartée* : finaliser d'office au chargement de la page ; lire `onboardingState.loading` (fenêtre ci-dessus).
- **D4 — Gardes de `/onboarding`** (C-36-17-4) : production à l'étape 7 → écran de reprise (plus de renvoi à `/`) ; à
  l'étape ≥ 8 → page terminée. Démo à l'étape ≥ 3 → `/`, inchangé. `/onboarding/restore` à l'étape > 2 renvoie toujours à
  `/onboarding` (`restore/+page.ts:32-33`), qui montre désormais l'écran de l'état au lieu de renvoyer à `/`.
  *Écartée* : renvoi à `/` à l'étape ≥ 8 (la page terminée invisible au rechargement, contraire à #1187, point 2).
- **D5 — L'application reste ouverte à l'étape 7 ; la bannière « Configuration incomplète » s'y affiche et dit de
  terminer avant d'utiliser Kesh** (C-36-17-5, arbitrages de l'orchestrateur sur V1-1 et V2-1) : condition
  `stepCompleted >= 6 && stepCompleted < 8` à `(app)/+layout.svelte:369` ; son lien mène à `/onboarding`, donc à l'écran
  de reprise. La garde `(app)/+layout.ts` ne change pas. **Le texte de la bannière change** (clé existante
  `incomplete-banner-text`, quatre locales, repli égal à `fr-CH`) : « Configuration incomplète : terminez-la avant
  d'utiliser Kesh. » ; le bouton garde `incomplete-banner-cta` (« Terminer la configuration »). L'écran de reprise porte la
  même phrase (D3). *(Le texte « … avant d'ouvrir les réglages de facturation ou de facturer », écrit à la remédiation P1,
  nommait un déclencheur trop étroit : V2-1.)*
  **Écran d'échec sous un refus `VALIDATION_ERROR`** (le seul de la finalisation : `onboarding.rs:1005-1008`, rendu en
  **400** par `errors.rs:1848`, « `build_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", &msg)` » ; reconnu au code,
  non au texte) : une phrase fixe s'affiche **avant** le motif du serveur, et ne le contredit pas (V2-2) : « Si les comptes
  1100 et 3000 figurent dans votre plan comptable, ce refus vient d'un écran de Kesh ouvert avant la fin de la
  configuration. Dans Administration → Facturation, désignez le compte débiteurs et le compte de produit par défaut,
  vérifiez aussi les autres comptes par défaut (créanciers, arrondi, escompte, frais bancaires, pertes), enregistrez,
  puis réessayez. » ; le motif du serveur suit, tel quel. La phrase est conditionnelle : si les comptes manquent vraiment,
  le motif du serveur reste juste et la phrase ne le nie pas.
  **Limite connue** : la finalisation reste refusée dans ce cas tant que l'utilisateur n'a pas fait ce geste. Elle est
  portée par **#1200**, que la **36-19** corrige dans la même release ; la 36-17 ne suppose pas cette correction (ses
  textes restent justes sans elle ; ils se relisent à l'intégration si la 36-19 est fusionnée, Dev Notes).
  *Écartées* : renvoyer l'application à l'assistant à l'étape 7 (une finalisation refusée en **400** `VALIDATION_ERROR`
  enfermerait l'utilisateur hors des écrans qui la réparent) ; corriger la finalisation dans la story (`kesh-db`,
  `kesh-api` : au-delà de cinq modules ; c'est la 36-19) ; nommer les écrans déclencheurs dans la bannière (liste longue,
  qui se périmerait).
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
  **Identifiants** (V1-2) : chaque branche existante 0 à 6 de `+page.svelte` reçoit une enveloppe
  `<div data-testid="onboarding-screen-<n>">` (`n` = l'étape), sans autre changement de son contenu ; les écrans neufs :
  `onboarding-screen-unloaded`, `-finalizing`, `-finalize-error`, `-resume`, `-done`, `-demo-loading`, `-demo-ready`,
  `-fallback`. Correspondance fixée : production 7 → `finalizing`, `finalize-error` ou `resume` selon les drapeaux locaux
  `finalizing` et erreur (précédence D3 ; ces drapeaux ne se lisent qu'à 7) ; production 6 avec `finalizing` vrai → `6` ;
  production ≥ 8 (9 et 10 compris) → `done` ; démo ≥ 3 → `demo-ready` ; démo 2 pendant `seedDemo()` → `demo-loading` ;
  −1 → `fallback` ; non chargé → `unloaded` (D9) ; chargement initial (`loading` et non `loaded`) → `unloaded` en son état
  « Chargement… » (`:320-323` reprise).
  *Écartée* : étendre la garde aux composants ; un détecteur qui ne compterait que les écrans neufs (il ne verrait plus
  rien des étapes 0 à 6).
- **D9 — Un état non lu a son écran** (C-36-17-9, V1-4) : si l'état n'a pu être lu (`loaded` faux hors chargement), la page
  rend `onboarding-screen-unloaded` — « L'état de la configuration n'a pas pu être lu. » et *Réessayer*, qui rappelle
  `onboardingState.fetchState()` (son `catch` dans `+page.svelte`, motif par `errorMessageOf`) — et non l'écran de la
  langue. La garde de `+layout.ts` continue de journaliser sans bloquer.
  *Écartée* : rendre l'étape 0 (l'état d'aujourd'hui) ; bloquer la garde (`/onboarding` deviendrait inaccessible au
  moindre échec réseau).

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
   `ONBOARDING_STEP_ALREADY_COMPLETED`, l'état est relu et son écran s'affiche (étape 6 relue → l'écran bancaire). Un
   refus `VALIDATION_ERROR` de la finalisation porte, **avant** le motif, la phrase de remède de D5 ; un échec réseau ou
   un autre code ne la porte pas. Pendant la finalisation, l'écran de reprise n'apparaît à aucun instant entre l'étape
   bancaire et la finalisation ; pendant une relance par *Réessayer*, « Configuration en cours… » s'affiche (l'erreur est
   effacée avant) ; *Réessayer* est désactivé tant qu'un appel ou le verrou du store est en cours (D3).
5. **AC5 — Rechargement.** `/onboarding` rechargé à l'étape 7 (production) montre l'écran de reprise et *Terminer la
   configuration*, qui finalise et mène à la page terminée ; aucun `POST` n'est envoyé au chargement. Rechargé à
   l'étape ≥ 8, il montre la page terminée et son récapitulatif. La démo à l'étape ≥ 3 est toujours renvoyée à `/`.
6. **AC6 — L'application aux étapes 6 et 7** montre la bannière, au texte neuf de D5 (« Configuration incomplète :
   terminez-la avant d'utiliser Kesh. », quatre locales, repli égal à `fr-CH`), dont le lien mène à l'écran de l'état
   (bancaire à 6, reprise à 7) ; aux étapes < 6 et ≥ 8, la garde et l'absence de bannière sont celles d'avant. L'écran de
   reprise porte la même phrase.
7. **AC7 — Un écran pour chaque état.** Pour tout `stepCompleted` de −1 à 10, `isDemo` vrai ou faux, chargement en cours
   ou non, état chargé — et pour l'état non chargé —, la page rend **exactement un** élément `[data-testid^="onboarding-screen-"]`,
   celui de la correspondance de D8 (branches 0 à 6 comprises) ; **à l'étape 7 (production) et à l'étape 6, la matrice
   porte aussi les drapeaux locaux** — aucun appel, finalisation en attente, finalisation rejetée, relance après rejet ;
   à l'étape 6, étape bancaire en attente — pilotés par les promesses de l'API simulée, chacun avec son écran attendu ;
   l'écran du type d'organisation n'apparaît jamais en démo ; un état non lu ne rend jamais l'écran de la langue (D9).
8. **AC8 — Chemin démo.** Pendant le chargement des données de démonstration, « Chargement des données de
   démonstration… » occupe la zone de l'assistant ; à son succès, `/` s'ouvre ; si la navigation échoue, l'écran « Les
   données de démonstration sont prêtes » et son *Commencer* restent, avec le motif.
9. **AC9 — Textes.** Chaque texte neuf a sa clé `onboarding-*` dans les quatre locales (`fr-CH`, `de-CH`, `it-CH`,
   `en-CH`), son repli français égal à la valeur `fr-CH` ; `parity_between_locales` (`crates/kesh-i18n/src/loader.rs:771`)
   et `npm run lint-i18n-ownership` verts ; les deux clés mortes de D2 retirées (ou leur lecteur restant cité).
10. **AC10 — Manuels.** `user-manual.tex` (§ *Onboarding*, après `:217`) décrit l'indicateur, la page terminée et son
    récapitulatif, *Commencer*, l'échec et *Réessayer*, la reprise au rechargement, l'indicateur de la démo, et la
    **limite connue** de D5 : terminer la configuration avant d'utiliser Kesh — ouvrir le journal, les comptes bancaires,
    une facture, le détail d'une écriture, les réglages de facturation, ou exporter, avant la fin, fait refuser la mise en
    place —, et, faute de quoi, le remède (désigner les comptes débiteurs et de produit, vérifier les autres comptes par
    défaut, enregistrer, réessayer), avec #1200 ; `admin-manual.tex` dit, après `:1461`, que l'assistant se termine sur la
    page « Configuration terminée », et la même limite en une phrase. Si la 36-19 (#1200) est fusionnée dans la release,
    l'intégration retire la limite des deux manuels (Dev Notes).
    PDF régénérés à l'intégration (contrôle aplati, forme E6, de la phrase neuve).
11. **AC11 — Périmètre serveur nul.** `git diff --stat origin/epic/36-0.18.0...HEAD -- crates/ ':!crates/kesh-i18n/locales'`
    (trois points : les seuls changements de la branche, V2-7) ne rend rien ; aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 91).
12. **AC12 — Recette visuelle** : captures, thème clair, base de démonstration jetable, de chaque état touché — en cours,
    échec avec *Réessayer*, reprise, terminée avec compte bancaire, terminée sans compte bancaire, récapitulatif en échec,
    chargement de la démo, démo prête, écran de repli, état non lu, échec avec la phrase de remède de D5, application à
    l'étape 7 avec la bannière au texte neuf — dans
    `/home/gcorbaz/devel/kesh-gate-logs/e36-17/captures/`, jointes au Dev Agent Record et au prompt de la revue.

## Tests à écrire, et la preuve du rouge

Écrits d'abord, sur le code de `0bf3c3b4b`, journal `/home/gcorbaz/devel/kesh-gate-logs/e36-17/rouges.log` ; puis verts
au correctif. Le code se commite avant toute mutation (E5) ; scripts de mutation dans `kesh-gate-logs/e36-17/mut/` ;
restauration par `git restore --worktree -- <fichier>` puis `touch`.

**Vitest** (fichiers à côté de la page, nom sans préfixe `+`, patron de `onboarding-bank-step.test.ts:20-60` : API de
l'assistant, i18n, `$app/navigation`, `svelte-sonner` simulés) :

- `onboarding-screens.test.ts` — **le détecteur d'AC7** : la matrice des états (D8), exactement un écran identifié pour
  chacun, l'identifiant attendu nommé par la table. Deux assertions par cas : l'identifiant attendu, et une zone de
  l'assistant non vide. Au départ, tout rougit sur l'identifiant (aucun n'existe) ; la sonde positive porte sur la seconde
  assertion et sur la démo : (7, production) et (8, production) rougissent sur « zone non vide », (3, démo) sur la présence
  de `onboarding-org-type-*`, et (0, production) **passe** la seconde — preuve que l'assertion « zone non vide » n'est pas
  rouge partout (`rouges.log`). État non chargé : (rejet de `fetchState`) rend aujourd'hui l'écran de la langue
  (`onboarding-screen-0` attendu absent après le correctif).
- `onboarding-end.test.ts` :
  - AC1 : *Configurer plus tard* avec `finalize` en attente (promesse non résolue) → `onboarding-screen-finalizing` ;
    résolue à l'étape 8 → `onboarding-screen-done`, `goto` jamais appelé. Rouge au départ (rien rendu, `goto('/')`
    appelé).
  - AC3 : récapitulatif (société, type, langue, exercice en date suisse, banque et description, pas d'IBAN — la chaîne
    `CH93` absente du rendu) ; sans compte bancaire, la phrase d'ajout ; `fetchCompanyCurrent` en échec → la ligne et le
    motif, *Commencer* présent.
  - AC4 : refus `VALIDATION_ERROR` → phrase de remède de D5 **avant** le motif (ordre des nœuds vérifié), `toastError` non
    appelé ; `TypeError` → « Finalisation échouée — réessayez. », sans phrase de remède ; *Réessayer* → l'écran
    « en cours » pendant la relance (seconde promesse en attente ; `onboarding-screen-finalize-error` absent), puis
    `finalize` appelé une seconde fois → page terminée ; `ONBOARDING_STEP_ALREADY_COMPLETED` → `fetchState` appelé, écran
    de l'état relu (6 → `onboarding-screen-6`).
  - D3 (V1-3, V2-5) : la fenêtre entre la fin de `skipBank` et le début de `finalize` dure moins d'une microtâche ; un test
    qui reprend la main par `await tick()` ne s'y intercale pas. Le test pose donc, **avant** le clic sur *Configurer plus
    tard*, un `MutationObserver` (`childList`, `subtree`) sur le conteneur du rendu, qui enregistre tout nœud ajouté portant
    `data-testid="onboarding-screen-resume"` (un nœud ajouté puis retiré reste dans les enregistrements), puis exige **zéro**
    apparition jusqu'à `onboarding-screen-finalizing` ; `finalize` rejeté → *Réessayer* désactivé jusqu'au tour suivant,
    puis actif. M10 doit y rougir (sinon, la mutation se déclare équivalente pour ce test, avec la raison).
  - D9 : `fetchState` rejeté → `onboarding-screen-unloaded`, *Réessayer* → `fetchState` rappelé → écran de l'état lu.
  - AC5 : état chargé à 7 sans appel → écran de reprise, `finalize` **non** appelé au rendu ; *Terminer la configuration*
    → appelé.
  - AC8 : `seedDemo` en attente → indicateur de la démo ; `goto` rejeté → écran « démo prête » et motif.
- `onboarding-motif.test.ts:235-290` : réécrit — le motif lu à l'écran, `toastError` non appelé (D2). Rouge au départ par
  construction (le toast est l'état d'aujourd'hui).
- `onboarding-catch-guard.test.ts` : plancher monté au nombre recompté (le recompte se cite au Change Log).
- `onboarding-guard.test.ts` (neuf) — la fonction `load` de `routes/onboarding/+layout.ts`. ⚠️ Sous Vitest,
  `browser` (`$app/environment`) vaut faux, et `load` ne fait alors **rien** (`+layout.ts:10`, `:15`) : sans simulacre, le
  cas « production 7 → pas de `redirect` » passerait à vide. Le test simule donc `vi.mock('$app/environment', () => ({
  browser: true }))`, `$lib/app/stores/auth.svelte` (`authState.isAuthenticated`) et l'API de l'assistant (patron
  `onboarding-bank-step.test.ts:30-46`). Cas : production 7 et 8 (et 10) → pas de `redirect` ; démo 3 →
  `redirect(302, '/')` ; non authentifié → `redirect(302, '/login')`. **Sonde positive** : les deux derniers cas sont
  **verts au départ** — preuve que `load` s'exécute bien sous le simulacre ; production 7 et 8 sont **rouges au départ**
  (`redirect` levé, `+layout.ts:30-32`).
- Bannière (AC6) : la condition extraite en fonction pure, `showIncompleteBanner(state)`, dans `features/onboarding/`, lue
  par `(app)/+layout.svelte:369` : 5 → non, 6 → oui, 7 → oui, 8 → non, démo → non ; et `IncompleteBanner` rendu avec le
  texte neuf « Configuration incomplète : terminez-la avant d'utiliser Kesh. » (repli égal à `fr-CH`). Rouge au départ
  sur 7 et sur le texte.

**E2E** (le défaut se voit dans le navigateur : exigé) — `frontend/tests/e2e/onboarding-fin.spec.ts`, préréglage `fresh`,
connexion `changeme`, **assistant joué à l'interface** jusqu'à l'étape bancaire (sélecteurs `id` et `data-testid`, aucun
sélecteur de texte traduit ; champs de coordonnées `#coord-name`, `#coord-street`, `#coord-npa`, `#coord-city`,
`+page.svelte:414-452`), **aucun `page.reload()` ni `page.goto()` entre le clic bancaire et l'assertion de la page
terminée** :

- **E1 — compte saisi** : `onboarding-screen-done` visible, URL `/onboarding`, récapitulatif (société, banque) ; *Commencer*
  → `/`, menu latéral visible ; contrôle par `authedApiContext` que `GET /api/v1/onboarding/state` rend l'étape 8.
- **E2 — compte passé, finalisation retenue puis interrompue**, sans délai fixe : un gestionnaire
  `page.route('**/api/v1/onboarding/finalize', …, { times: 1 })` **retient** la première requête sur une promesse ; le test
  attend `onboarding-screen-finalizing` **visible**, puis libère la promesse et le gestionnaire fait `route.abort()` → écran
  d'échec et *Réessayer* (la requête n'a jamais atteint le serveur : l'état reste 7). `page.unroute(…)`, puis *Réessayer*
  → page terminée, sans compte bancaire, avec la phrase d'ajout. (Le client ne rejoue pas seul un `POST` :
  `api-client.ts:311`, « `/** True si la méthode HTTP est idempotente — éligible au retry exponentiel. */` ».)
- **E3 — reprise** : étapes jouées par l'API jusqu'à `skip-bank` (étape 7, sans finalisation), puis `page.goto('/onboarding')`
  → écran de reprise (et non `/`) ; *Terminer la configuration* → page terminée ; `page.reload()` → page terminée de
  nouveau.
- **E4 — démo** : chemin démo avec `seed-demo` retenu de même par `page.route` (promesse libérée après l'attente de
  `onboarding-screen-demo-loading` visible, puis `route.continue()`) → `/`.

Preuve du rouge de l'E2E : la spec jouée contre le frontend construit au code d'avant le correctif (`git stash` du seul
frontend, ou construction à `0bf3c3b4b`) — E1 rouge à l'attente de `onboarding-screen-done` — journal
`kesh-gate-logs/e36-17/e2e-rouge.log`. `onboarding-bank-account.spec.ts:83` adapté (page terminée, puis *Commencer*).

**Mutations** (journal `kesh-gate-logs/e36-17/mut/*.log`) : **M1** branche de l'étape 7 retirée (AC7, AC1) ; **M2**
`goto('/')` d'office rétabli après la finalisation (AC1 : `goto` non appelé) ; **M3** retour au toast dans
`reportFinalizeError` (AC4) ; **M4** finalisation d'office au rendu de l'étape 7 (AC5 : `finalize` non appelé au rendu) ;
**M5** garde `step >= 7 → '/'` rétablie (AC5, `onboarding-guard.test.ts`) ; **M6** condition de bannière `< 7` rétablie
(AC6) ; **M7** l'écran du type d'organisation sans `!isDemo` (AC7) ; **M8** IBAN ajouté au récapitulatif (AC3) ; **M9**
`fetchState` retiré du cas « déjà finalisé » (AC4) ; **M10** l'écran « en cours » lu sur `onboardingState.loading` au lieu
de `finalizing` (D3, test de la fenêtre) ; **M11** *Réessayer* non désactivé pendant le verrou du store (D3) ; **M12**
l'état non chargé rendu comme l'étape 0 (D9, AC7) ; **M13** l'enveloppe `onboarding-screen-4` retirée (AC7 : le
détecteur voit les branches 0 à 6) ; **M14** l'ancien texte de la bannière (« Configuration incomplète — Terminer la
configuration ») rétabli dans `fr-CH` (AC6) ; **M15** la phrase de remède de D5 retirée de l'écran d'échec (AC4) ;
**M16** la phrase de remède placée après le motif (AC4) ; **M17** *Réessayer* qui pose `finalizing` sans effacer l'erreur
(AC4, AC7 : écran d'échec pendant la relance). Une mutation équivalente se dit.

## Tâches

- [ ] **T0 — Étape 0** : relire le dernier rapport de validation ; recontrôler au code les lignes citées ci-dessus à la tête
  de la branche (elle a pu bouger : 36-7, 36-15 en parallèle), ligne lue citée au Change Log.
- [ ] **T1 — Rouges** : les tests de la section précédente, écrits et journalisés rouges (`rouges.log`), sonde positive du
  détecteur comprise.
- [ ] **T2 — Écrans** (D3, D6, D7, D8, D9) : composants présentationnels sous `frontend/src/lib/features/onboarding/`
  (non chargé, en cours, échec, reprise, terminée avec récapitulatif, démo en chargement, démo prête, repli) ;
  identifiants de D8 ; fonction pure `showIncompleteBanner`.
- [ ] **T3 — Page** (D1, D2, D3, D7, D8, D9) : `+page.svelte` — enveloppes `onboarding-screen-0` à `-6` des branches
  existantes ; branches 7, ≥ 8, démo ≥ 3, non chargé, `{:else}` ; drapeau `finalizing` et précédence de D3 ;
  `navigateHomeIfFinalized` retiré ; `reportFinalizeError` vers l'état d'écran, avec la phrase de remède de D5 ;
  *Réessayer* / *Terminer la configuration* désactivés pendant un appel ; chargement du récapitulatif ; *Réessayer* de
  l'état non lu ; `chooseDemoPath` attend `goto` ; plancher de la garde recompté.
- [ ] **T4 — Gardes et bannière** (D4, D5) : `routes/onboarding/+layout.ts` ; `(app)/+layout.svelte:369` par
  `showIncompleteBanner`.
- [ ] **T5 — Textes** (AC9, AC6) : clés `onboarding-*` neuves aux quatre `messages.ftl` ; `incomplete-banner-text` réécrite
  dans les quatre (`:159`) et son repli (`IncompleteBanner.svelte:14`) ; clés mortes retirées ; replis égaux à `fr-CH`.
- [ ] **T6 — E2E** : `onboarding-fin.spec.ts` (E1 à E4) ; `onboarding-bank-account.spec.ts:83` adapté ; une ligne de
  commentaire à `onboarding-path-b.spec.ts:65`, `:92` et `onboarding.spec.ts:150` (leur fin passe désormais par
  *Commencer* ; KF-029 inchangée) et à `onboarding-path-b.spec.ts:27` (le `fixme` KF-032, #124, reste : la bannière ne
  s'affiche pas à l'étape 8).
- [ ] **T7 — Manuels** (AC10) : `user-manual.tex` § *Onboarding*, limite connue comprise ; `admin-manual.tex` après
  `:1461`. PDF à l'intégration.
- [ ] **T8 — Captures** (AC12), backend lancé pour elles, sur `kesh_e2e_e3617` recréée.
- [ ] **T9 — Mutations** M1 à M17, après commit du code.
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
  bannière « Configuration incomplète » le rappelle dans l'application, en demandant de la terminer avant d'utiliser
  Kesh. Le chargement des données de démonstration affiche lui aussi un indicateur. (refs #1187)

*À l'intégration, l'orchestrateur garde **une seule** des deux phrases suivantes, selon que la 36-19 (#1200) est fusionnée
dans la 0.18.0 ou non ; elles ne se mélangent pas :*

- *Si la 36-19 n'est pas dans la release* — *Limite connue* : si vous ouvrez un écran de Kesh (journal, comptes bancaires,
  factures, réglages de facturation) avant la fin de la configuration, la mise en place est ensuite refusée tant que le
  compte débiteurs et le compte de produit par défaut ne sont pas désignés et enregistrés dans Administration →
  Facturation ; l'écran d'échec dit comment faire. (refs #1200)
- *Si la 36-19 est dans la release* — aucune phrase ici : la correction de #1200 a sa propre note, celle de la 36-19, et
  les textes de la 36-17 qui la mentionnent (phrase de remède, manuels) se relisent à l'intégration.

## Dev Notes

### Hors périmètre, et les issues demandées

- **Limite connue, #1200** (V1-1, arbitrage de l'orchestrateur ; ouverte par l'orchestrateur) — « Assistant : la finalisation est refusée
  (« 1100, 3000 manquants ») si les réglages de facturation ont été ouverts avant elle », `bug`, P2. Lignes :
  `crates/kesh-db/src/repositories/company_invoice_settings.rs:129` (ligne créée sans comptes par
  `get_or_create_default`), `:1250` (`INSERT IGNORE` de la finalisation, sans effet sur une ligne existante),
  `:1280-1306` (relecture jointe aux comptes actifs → `InactiveOrInvalidAccounts`, `:1305`) ;
  `crates/kesh-api/src/routes/onboarding.rs:1008` (motif faux) ; appelants qui créent la ligne avant la finalisation :
  `routes/company_invoice_settings.rs:300`, `:336`, `exports/global.rs:335`, et, côté frontend, tout appel à
  `getInvoiceSettings()` (cinq écrans : journal, comptes bancaires, formulaire et détail de facture, détail d'écriture,
  réglages de facturation — sortie citée sous « Ce qui se lit au code » ; lignes ajoutées en commentaire sur #1200 par
  l'orchestrateur) ; ouverture de l'application avant la finalisation : `frontend/src/routes/(app)/+layout.ts:42`.
  **Corrigée par la story 36-19 dans la même release 0.18.0** (arbitrage de l'orchestrateur) : la finalisation complète
  les comptes vides d'une ligne existante, côté serveur, sans écraser une désignation. **La 36-17 n'en dépend pas.** Si la
  36-19 est fusionnée avant le tag, l'intégration relit : la phrase de remède de l'écran d'échec (D5) — devenue sans
  objet, à retirer ou à garder pour une installation restée sur l'ancien serveur, au choix de l'intégration —, la limite
  des deux manuels (AC10), la note de version (une seule des deux phrases) ; la bannière et l'écran de reprise
  (« terminez-la avant d'utiliser Kesh ») restent justes dans les deux cas.
- `DemoBanner.svelte:28` — navigation non attendue après la remise à zéro de la démo : **#1199** (inventaire, symptôme 2 ;
  `technical-debt`, P3).
- KF-029 (#97) : `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:150` restent rouges (sélecteurs périmés) ; la
  story n'y ajoute qu'un commentaire. `onboarding-path-b.spec.ts:27` reste `fixme` (KF-032, #124, fermée).
- Le nom de la société non modifiable après l'assistant (#1097) : hors epic, sans clé.
- Le serveur : aucune ligne. Si l'étape 0 trouvait une cause côté serveur, la story s'arrête et le dit (périmètre changé).

### Fichiers communs avec les autres stories de la phase 2

| Fichier | Autres stories | Règle |
|---|---|---|
| `crates/kesh-i18n/locales/*/messages.ftl` | 36-15 (vocabulaire « compte analytique », quatre locales), 36-18 | union à l'intégration ; clés `onboarding-*` et `incomplete-banner-text` seulement |
| `docs/manual/fr/user-manual.tex`, `admin-manual.tex` | 36-15, 36-10, 36-9 | union ; § *Onboarding* et `sec:onboarding-config` seulement |
| `frontend/src/routes/(app)/+layout.svelte` | 36-15 (libellés du menu, possibles) | union ; une condition à `:369` |

### Modules touchés (règle de splitting)

1. `frontend/src/routes/onboarding/` (page, garde) ; 2. `frontend/src/lib/features/onboarding/` (écrans, fonction de la
bannière) ; 3. `frontend/src/routes/(app)/+layout.svelte` (une condition) ; 4. `crates/kesh-i18n/locales/` (textes) ;
5. `docs/manual/fr/`. **Cinq** : à la limite, pas au-delà — pas de découpage. Les E2E accompagnent le module 1. Après la
remédiation de la validation P1 : inchangé — les enveloppes des étapes 0 à 6 et l'écran non chargé restent dans
`+page.svelte` (module 1). Le texte neuf de la bannière ajoute un fichier, `frontend/src/lib/shared/components/IncompleteBanner.svelte`
(son seul repli, `:14`, pour rester égal à `fr-CH`) : il est compté avec le module 3 sous le nom de **coquille de
l'application** (`(app)/+layout.svelte` et la bannière qu'il rend, sans autre usage : `git grep -n IncompleteBanner --
frontend/src ':!*.test.ts'` → `(app)/+layout.svelte:10` et `:370` seulement). Si ce regroupement n'est pas admis, le
décompte est de **six** ; le repli peut alors rester l'ancien texte (la clé seule change), au prix d'un repli différent de
`fr-CH` — à arbitrer par l'orchestrateur, la fiche retient le regroupement. *(Regroupement accepté par l'orchestrateur le
2026-11-15 : cinq modules ; C-36-17-10.)*

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
- **Limite connue, assumée** : l'application reste ouverte aux étapes 6 et 7 (D5) ; si un écran qui lit les réglages de
  facturation (journal, comptes bancaires, factures, détail d'écriture, réglages de facturation) ou l'export y est ouvert
  avant la finalisation, celle-ci est refusée (« 1100, 3000 manquants ») jusqu'à ce que les comptes débiteurs et de produit
  soient désignés et enregistrés dans les réglages de facturation, les autres comptes par défaut restant à vérifier. La
  story ne la corrige pas ; elle la dit (bannière, écran de reprise, écran d'échec, manuels) ; #1200 la porte, corrigée par
  la 36-19 dans la même release ; **la 36-17 reste juste sans elle**. Le serveur ne change pas.

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
- 2026-11-15 — **Remédiation de la validation P1** (Opus 5.5 ; rapport `kesh-gate-logs/e36-17/validation-p1.md` : 0/0/2/4).
  Prémisses lues au code avant correction :
  - **V1-1 (MEDIUM), corrigé, prémisse corrigée sur son effet** : la ligne sans comptes est bien créée par
    `get_or_create_default` (`company_invoice_settings.rs:129`, appelée par `routes/company_invoice_settings.rs:300`,
    `:336`, `exports/global.rs:335`) et l'`INSERT IGNORE` de la finalisation (`:1250`) est sans effet ; mais la
    finalisation n'aboutit **pas** « sans comptes » : la relecture jointe aux comptes actifs (`:1280-1306`) rend
    `InactiveOrInvalidAccounts` (`:1305`, « `None => Err(DbError::InactiveOrInvalidAccounts),` »), d'où un refus à motif
    faux (`onboarding.rs:1008`). Arbitrage de l'orchestrateur : D5 gardée ; limite connue écrite (Ce qui se lit au code,
    D5, AC4, AC6, AC10, Ce qui doit rester vrai, note de version) ; bannière vérifiée au code — elle ne dit pas de terminer
    avant de facturer (`fr-CH/messages.ftl:159`) — et réécrite ; issue demandée (Dev Notes).
  - **V1-2 (MEDIUM), corrigé** : `git grep onboarding-screen frontend/src` → vide, confirmé ; enveloppes des étapes 0 à 6,
    table de correspondance (D8), sonde positive redéfinie sur l'assertion « zone non vide ».
  - **V1-3 (LOW), corrigé** : fenêtre confirmée (`onboarding.svelte.ts:144-150`, `finally` de `skipBank` ; `:160`) ;
    drapeau `finalizing`, précédence, boutons désactivés (D3) ; test et mutations M10, M11.
  - **V1-4 (LOW), corrigé** : `+layout.ts:33-38` avale l'erreur, état par défaut `onboarding.svelte.ts:14-21` ; D9.
  - **V1-5 (LOW), corrigé** : `#coord-address` aux `onboarding-path-b.spec.ts:73`, `:106` (et non `:76`, lu à `:73`),
    `onboarding.spec.ts:168` ; `:27` est le `fixme` KF-032 (#124, fermée), `:32` ; E2 et E4 attendent l'indicateur affiché,
    non un délai ; `api-client.ts:311` (pas de rejeu d'un `POST`).
  - **V1-6 (LOW), corrigé** : garde Vitest avec `browser: true` simulé et sonde positive (cas verts au départ) ; limite au
    manuel (AC10).
  Choix C-36-17-3, -5 et -8 annotés, C-36-17-9 ajouté. Modules : cinq, inchangé.
- 2026-11-15 — **Issues ouvertes par l'orchestrateur** : **#1200** (limite connue de V1-1, `bug`, P2) et **#1199**
  (`DemoBanner.svelte:28`, `technical-debt`, P3) ; citées à leurs sites (code lu, D5, AC10, note de version, Dev Notes, ce
  qui doit rester vrai). Les entrées datées ci-dessus, qui disent « issue à ouvrir » et « issue demandée », restent telles
  quelles. Regroupement de `IncompleteBanner.svelte` avec la coquille de l'application accepté : cinq modules
  (C-36-17-10).
- 2026-11-15 — **Remédiation de la validation P2** (Opus 5.5 ; rapport `kesh-gate-logs/e36-17/validation-p2.md` : 0/0/1/6).
  Prémisses lues au code, aucune réfutée :
  - **V2-1 (MEDIUM), corrigé** : `git grep -n "getInvoiceSettings" -- frontend/src ':!*.test.ts'` rend les huit appels de
    cinq écrans (sortie citée) ; `invoices.api.ts:99-100` (`GET /api/v1/company/invoice-settings`) ; la validation passe par
    `get_or_create_default_in_tx` (`invoices.rs:2156`) dans une transaction annulée (`:2158-2163`). Arbitrage de
    l'orchestrateur : bannière et écran de reprise « Configuration incomplète : terminez-la avant d'utiliser Kesh » ;
    #1200 corrigée par la 36-19 dans la même release, la 36-17 n'en dépend pas ; déclencheur juste dans « Ce qui se lit
    au code », D5, AC6, AC10, Dev Notes, ce qui doit rester vrai ; note de version en deux cas exclusifs.
  - **V2-2 (LOW), corrigé** : un seul `AppError::Validation` dans la finalisation (`onboarding.rs:1005-1008`), 400 par
    `errors.rs:1848` ; phrase de remède conditionnelle, placée avant le motif ; test d'ordre et M16.
  - **V2-3 (LOW), corrigé** : `company_invoice_settings.rs:1251-1265` lu ; les autres comptes par défaut nommés au remède,
    au code lu et au manuel.
  - **V2-4 (LOW), corrigé** : ordre du geste (effacer l'erreur, poser `finalizing`, appeler) et lecture des drapeaux à
    l'étape 7 seulement (D3, D8) ; matrice d'AC7 étendue aux drapeaux ; M17.
  - **V2-5 (LOW), corrigé** : test de la fenêtre par `MutationObserver` posé avant le clic, zéro apparition exigée.
  - **V2-6 (LOW), corrigé** : 400 et non 422 (D5) ; `SetupForm.svelte:27` ajouté à la sortie de l'inventaire.
  - **V2-7 (LOW), corrigé** : AC11 en `origin/epic/36-0.18.0...HEAD` (la base a avancé : `9172e9fe8`, `ca5d02ed0`,
    `e849b605d`, `c7ca079d2`).
  C-36-17-3, -5 annotés. Modules : cinq, inchangé (aucun fichier neuf hors des cinq).
