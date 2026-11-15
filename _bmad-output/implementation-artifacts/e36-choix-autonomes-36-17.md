# Epic 36 — registre des choix autonomes de la story 36-17 (#1187)

Registre propre à la story 36-17, **versé par union**. Fiche : `36-17-assistant-fin-configuration.md`. Code lu à
`0bf3c3b4b` (tête de `epic/36-0.18.0`, kickoff de la 0.18.0). Ces choix ne sont pas couverts par C-e360 ; ils précisent
C-e360-18 (périmètre de #1187), qui fait foi. Option prudente et réversible à chaque fois (règle de l'epic).

### C-36-17-1 — Le chemin production s'arrête sur « Configuration terminée » ; *Commencer* recharge l'application (D1)

- **Contexte** : `+page.svelte:255-266` navigue d'office par `goto('/')` (`:257`), sans `await` ni traitement d'échec ;
  #1187 demande une page « Configuration terminée » avec *Commencer*.
- **Retenu** : aucune navigation d'office après la finalisation ; l'écran de l'étape 8 est la page terminée ; *Commencer*
  est un lien `href="/"` portant `data-sveltekit-reload` (chargement complet, l'état de l'assistant se relit).
- **Écartées** : `await goto('/')` dans un `try` (une navigation cliente qui échoue demande encore un écran d'erreur) ;
  navigation d'office après un délai (l'utilisateur ne lit pas le récapitulatif).
- **Réversible** : oui (un lien).

### C-36-17-2 — L'échec de la finalisation s'affiche à l'écran, sans toast ; « déjà finalisé » relit l'état (D2)

- **Contexte** : `reportFinalizeError` (`+page.svelte:239-250`) ne fait qu'un `toast.error`, passager, sur une page vide.
- **Retenu** : le motif (`errorMessageOf(err, onboarding-finalize-error)`) s'affiche dans l'écran de l'étape 7, région
  `role="alert"`, avec *Réessayer* ; plus de toast pour la finalisation. Sur `ONBOARDING_STEP_ALREADY_COMPLETED`, la page
  relit l'état (`onboardingState.fetchState()`) et montre l'écran de l'état relu ; si la relecture échoue, son motif
  s'affiche de même avec *Réessayer*.
- **Écartées** : toast et écran (message doublé) ; toast seul (l'état d'aujourd'hui).
- **Réversible** : oui.

### C-36-17-3 — Au rechargement à l'étape 7, un écran de reprise, sans finalisation d'office (D3)

- **Contexte** : `routes/onboarding/+layout.ts:30-32` renvoie à `/` dès l'étape 7, finalisation faite ou non : une
  installation non finalisée entre dans l'application sans réglages de facturation, taux de TVA ni exercice
  (`onboarding.rs:985`, `:1018`, `:1046`), et rien ne le signale.
- **Retenu** : à l'étape 7 hors finalisation en cours, l'écran dit que la configuration n'est pas terminée, avec
  *Terminer la configuration* (même geste que *Réessayer*). Aucun `POST` au chargement de la page.
- **Écartées** : finaliser d'office au chargement (une écriture sans geste de l'utilisateur) ; garder le renvoi à `/`
  (la finalisation inachevée reste ignorée, contraire à #1187, point 4).
- **Réversible** : oui.
- *Complété le 2026-11-15 à la remédiation de la validation P1 (V1-3)* : l'état « en cours » se lit sur un drapeau local
  `finalizing`, posé avant l'appel de l'étape bancaire, et non sur `onboardingState.loading` (fenêtre entre le `finally`
  de `skipBank`, `onboarding.svelte.ts:144-150`, et `:160`) ; précédence échec > en cours > repris ; boutons désactivés
  pendant un appel ou le verrou d'un tour du store (`:166-168`).
- *Complété le 2026-11-15 à la remédiation de la validation P2 (V2-4, V2-1)* : *Réessayer* et *Terminer la configuration*
  effacent l'erreur, puis posent `finalizing`, puis appellent la finalisation ; `finalizing` et l'erreur ne se lisent qu'à
  l'étape 7. L'écran de reprise dit « Configuration incomplète : terminez-la avant d'utiliser Kesh. » (même clé que la
  bannière).

### C-36-17-4 — La garde de `/onboarding` ne renvoie plus le chemin production à `/` ; celle du chemin démo reste (D4)

- **Retenu** : chemin production, étape 7 → écran de reprise ; étape ≥ 8 → page terminée (#1187, point 2 : elle
  s'affiche au rechargement). Chemin démo, étape ≥ 3 → `/`, inchangé. Les étapes au-delà de 8 (préréglage de test
  `post-onboarding`, `test_fixtures.rs:508`, étape 10) valent « terminé ».
- **Écartée** : garder le renvoi à l'étape ≥ 8 (la page terminée invisible au rechargement).
- **Réversible** : oui (deux conditions).

### C-36-17-5 — L'application reste ouverte à l'étape 7 ; la bannière « Configuration incomplète » s'y affiche (D5)

- **Contexte** : `(app)/+layout.ts:42` laisse entrer dès l'étape 6 ; `(app)/+layout.svelte:369` n'affiche la bannière
  (lien vers `/onboarding`) qu'à l'étape 6.
- **Retenu** : la bannière s'affiche aux étapes 6 et 7 (`< 8`) ; la garde de `(app)` est inchangée.
- **Écartée** : renvoyer l'application à `/onboarding` à l'étape 7 — une installation dont la finalisation est refusée
  (422, comptes 1100/3000 absents, `onboarding.rs:991`) perdrait l'accès aux écrans qui permettent de la réparer.
- **Réversible** : oui.
- *Complété le 2026-11-15 à la remédiation de la validation P1 (V1-1 ; arbitrage de l'orchestrateur : D5 gardée)* :
  ouvrir les réglages de facturation avant la finalisation crée leur ligne sans comptes
  (`company_invoice_settings.rs:129`), et la finalisation est ensuite refusée à motif faux (`:1280-1306`, `:1305` ;
  `onboarding.rs:1008`). **Limite connue**, portée par **#1200** ; la bannière (`incomplete-banner-text`, quatre locales) dit
  désormais de terminer avant d'ouvrir les réglages de facturation ou de facturer, l'écran de reprise aussi, et l'écran
  d'échec ajoute le remède sous un refus `VALIDATION_ERROR`. Écartée en plus : corriger la finalisation ici (`kesh-db`,
  `kesh-api`, au-delà de cinq modules).
- *Complété le 2026-11-15 à la remédiation de la validation P2 (V2-1, V2-2, V2-3, V2-6 ; arbitrage de l'orchestrateur)* :
  le déclencheur est **tout** `GET /api/v1/company/invoice-settings`, lancé par `getInvoiceSettings()` depuis cinq écrans
  (journal, comptes bancaires, formulaire et détail de facture, détail d'écriture, réglages de facturation) et par
  l'export — non les seuls réglages. La bannière et l'écran de reprise disent donc « Configuration incomplète :
  terminez-la avant d'utiliser Kesh. » (remplace le texte du complément précédent). La phrase de remède est conditionnelle
  (« Si les comptes 1100 et 3000 figurent dans votre plan comptable… »), placée **avant** le motif du serveur, et nomme les
  autres comptes par défaut à vérifier. Le refus est un **400** `VALIDATION_ERROR` (`errors.rs:1848`), non un 422 comme
  l'écrivait la ligne « Écartée » ci-dessus. #1200 est corrigée par la **36-19** dans la même release ; ce choix ne
  suppose pas cette correction.
- *Annoté le 2026-11-15 à l'étape 0 du développement (V3-2, V3-5 de la validation P3)* : « cinq écrans » ci-dessus en
  énumère six — cinq pages et le formulaire de facture (`InvoiceForm.svelte`) ; outre la facture, l'avoir
  (`credit_notes.rs:434`) et la facture fournisseur (`supplier_invoices.rs:1651`) passent par
  `get_or_create_default_in_tx` dans une transaction annulée par leur refus : sans effet durable non plus.

### C-36-17-6 — Le récapitulatif se lit par les routes existantes ; son échec ne bloque pas la page (D6)

- **Retenu** : `GET /api/v1/companies/current` (nom, type d'organisation, langue comptable, comptes bancaires) et
  `GET /api/v1/fiscal-years`, toutes deux ouvertes à tout utilisateur authentifié (`lib.rs:1160-1163`, `:1210-1213`) ;
  « reste à faire » : les comptes de TVA pour un assujetti (phrase fixe, `user-manual.tex:219`) et, sans compte bancaire,
  son ajout. Lecture en échec : une ligne le dit (motif par `errorMessageOf`), le titre et *Commencer* restent.
- **Écartée** : une route neuve de récapitulatif (le serveur ne change pas : frontend seul, C-e360-18).
- **Réversible** : oui.

### C-36-17-7 — Chemin démo : indicateur pendant le chargement, navigation attendue, écran de repli (D7)

- **Retenu** : pendant `seedDemo()`, « Chargement des données de démonstration… » sur toute la page ; puis
  `await goto('/')` dans son propre `try` ; l'état démo à l'étape ≥ 3 a son écran (« Les données de démonstration sont
  prêtes », *Commencer* en lien rechargeant) : l'écran du type d'organisation (`+page.svelte:367`, qui ne regarde pas
  `isDemo`) n'apparaît plus en démo.
- **Écartée** : une page terminée obligatoire aussi en démo (un clic de plus, sans récapitulatif à donner).
- **Réversible** : oui.

### C-36-17-8 — Écrans présentationnels dans `features/onboarding` ; tout `catch` reste dans `+page.svelte` (D8)

- **Contexte** : la garde `onboarding-catch-guard.test.ts` ne lit que `+page.svelte` (`:27`, plancher `:28`).
- **Retenu** : les écrans neufs sont des composants sans `catch` ni appel d'API ; le chargement du récapitulatif, la
  finalisation et la navigation démo, avec leurs `catch`, vivent dans `+page.svelte` ; le plancher monte au nombre
  recompté.
- **Écartée** : étendre la garde aux composants (portée de la garde changée pour une story d'écran).
- **Réversible** : oui.
- *Complété le 2026-11-15 à la remédiation de la validation P1 (V1-2)* : les branches existantes 0 à 6 reçoivent une
  enveloppe `data-testid="onboarding-screen-<n>"`, et une table fixe l'écran de chaque état (fiche, D8), pour que le
  détecteur d'AC7 voie aussi les écrans existants.

### C-36-17-9 — Un état non lu a son écran, avec *Réessayer* (D9)

- **Contexte** (V1-4) : la garde de `/onboarding` avale l'échec de `fetchState()` (`routes/onboarding/+layout.ts:33-38`) ;
  l'état par défaut du module (`onboarding.svelte.ts:14-21`, étape 0) fait rendre l'écran de la langue.
- **Retenu** : `loaded` faux hors chargement → écran « L'état de la configuration n'a pas pu être lu. » et *Réessayer*
  (`fetchState()`, `catch` dans `+page.svelte`).
- **Écartées** : l'écran de l'étape 0 (aujourd'hui) ; une garde qui bloque `/onboarding` à l'échec.
- **Réversible** : oui.

### C-36-17-10 — `IncompleteBanner.svelte` compte avec la coquille de l'application (décompte des modules)

- **Contexte** : le texte neuf de la bannière (D5) change le repli de `frontend/src/lib/shared/components/IncompleteBanner.svelte:14`,
  pour rester égal à `fr-CH` ; le composant n'est rendu que par `(app)/+layout.svelte` (`:10`, `:370`).
- **Retenu** (accepté par l'orchestrateur le 2026-11-15) : il compte avec `(app)/+layout.svelte` dans le module 3,
  « coquille de l'application » ; la story touche **cinq** modules.
- **Écartée** : changer la clé seule et garder l'ancien repli (repli différent de `fr-CH`).
- **Réversible** : oui.

### C-36-17-11 — L'écran de reprise reçoit la phrase et le bouton de la bannière en props (développement)

- **Contexte** : D3 veut « même phrase et même clé que la bannière » (`incomplete-banner-text`, `incomplete-banner-cta`) ; mais
  `npm run lint-i18n-ownership` refuse, dans `src/lib/features/onboarding/`, une clé d'un autre espace
  (`front-lint-i18n-1.log` : « uses key "incomplete-banner-text" (incomplete namespace) from different feature »).
- **Retenu** : `+page.svelte` (hors de `features/`) résout les deux clés et les passe à `OnboardingFinalizeScreen` (props
  `resumeText`, `resumeCta`) : même clé, même phrase, lint vert.
- **Écartées** : deux clés `onboarding-*` à la même valeur (deux textes à tenir égaux) ; une exception au lint (dette #30).
- **Réversible** : oui.

### C-36-17-12 — Identifiants de test ajoutés aux boutons des étapes 0 à 6 (développement)

- **Contexte** : la spec E2E joue l'assistant à l'interface sans sélecteur de texte (fiche, E2E ; V3-6) ; les boutons de la
  langue, du mode, du chemin, de la langue comptable, des coordonnées et de l'étape bancaire n'avaient pas d'identifiant.
- **Retenu** : `data-testid` ajoutés (`onboarding-language-<code>`, `onboarding-mode-guided|expert`, `onboarding-path-demo|production`,
  `onboarding-accounting-lang-<code>`, `onboarding-coordinates-submit`, `onboarding-skip-bank`, `onboarding-bank-save`), sans autre
  changement du contenu de ces écrans (D8).
- **Écartée** : sélecteurs `button:has-text(...)` (texte traduit).
- **Réversible** : oui.

### C-36-17-13 — L'état non lu se teste dans son propre fichier (développement)

- **Contexte** : `_loaded` est un état de module qui ne repasse jamais à faux (`onboarding.svelte.ts:21`, `:48`) ; la matrice
  d'AC7 (`onboarding-screens.test.ts`) lit l'état avant chaque cas.
- **Retenu** : les cas « non chargé » d'AC7 et le *Réessayer* de D9 vivent dans `onboarding-unloaded.test.ts`, en **un seul**
  `it` ordonné (Vitest isole les modules par fichier) ; la fiche les plaçait dans `onboarding-screens.test.ts` et
  `onboarding-end.test.ts`.
- **Écartées** : `vi.resetModules()` et imports dynamiques (deux copies du moteur Svelte entre la page et le rendu de
  Testing Library) ; une remise à zéro du store réservée aux tests (code de production pour un test).
- **Réversible** : oui.

### C-36-17-14 — Allemand : « Lassen Sie diese Seite geöffnet » (développement)

- **Contexte** : la garde `la_cloture_d_exercice_ne_parle_pas_comme_un_panneau` (`crates/kesh-i18n/src/loader.rs:1368`) réserve
  « schliessen » nu aux panneaux ; « Schliessen Sie diese Seite nicht » l'a fait rougir (`nextest-cible-i18n-1.log`).
- **Retenu** : « Lassen Sie diese Seite geöffnet. » ; la bannière garde la forme séparable admise
  (« Schliessen Sie die Einrichtung ab, … »).
- **Réversible** : oui.
