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
