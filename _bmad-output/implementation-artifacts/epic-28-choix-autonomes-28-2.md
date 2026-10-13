# Epic 28 — registre des choix autonomes, Story 28-2

*Registre tenu par union (jamais par écrasement) ; à verser au registre de l'epic à la fusion. Chaque choix :
contexte, option retenue, alternatives, réversibilité.*

---

## Story 28-2 — Une erreur d'API ne s'affiche plus « [object Object] » (#520)

Fiche : `28-2-erreurs-object-object.md`. Choix faits à la création de la fiche (2026-10-13), en autonomie, sur
`6f971d76` (rétrospective de l'Epic 27).

### C-28-2-1 — Périmètre : les classes A et B sont traitées, C légitime, D assumée, E conforme

- **Contexte** : l'issue vise le motif `e instanceof Error ? e.message : String(e)`. L'inventaire au sol (fiche,
  § Inventaire) trouve 20 conversions affichées d'une valeur attrapée : 4 qui rendent « [object Object] » pour une
  `ApiError` (A), 16 qui la traitent mais gardent un repli `String` (B) ; 3 conversions dans un `console.*` (C) ;
  ≈ 23 `catch` qui n'affichent rien hors `ApiError` (D) ; de nombreux replis `i18nMsg` génériques (E).
- **Retenu** : A et B résolus par un utilitaire unique ; C légitimes par critère (journaliser n'est pas afficher) ;
  D angle mort assumé, issue de suivi demandée (C-28-2-10) ; E laissés tels quels (traduits, jamais « [object
  Object] »).
- **Écartées** : A seul (laisserait « TypeError: … » et « [object Object] » d'un objet quelconque sur 16 sites, et
  la garde ne pourrait pas interdire `String(err)`) ; A + B + D + E (refonte d'une centaine de sites, hors registre
  de la demande, dans sept modules de plus).
- **Réversible** : oui.

### C-28-2-2 — L'utilitaire dans un module neuf ; `isApiError` extrait et réexporté

- **Contexte** : `errorMessageOf` vit dans `api-client.ts` (15-5b). Le traduire exige `i18nMsg`, or
  `i18n.svelte.ts` importe `apiClient`. Et 18 fichiers de test simulent `$lib/shared/utils/api-client` par une
  fabrique qui ne fournit presque jamais `isApiError` ni `errorMessageOf`.
- **Retenu** : `lib/shared/utils/error-message.ts` (l'utilitaire) importe `isApiError` de
  `lib/shared/utils/api-error.ts` (neuf, sans dépendance), que `api-client.ts` réexporte. Les quatre importeurs
  d'`errorMessageOf` importent du module neuf ; `api-client.ts` ne le réexporte pas.
- **Écartées** : le laisser dans `api-client.ts` (cycle d'import et dépendance aux simulations du client) ;
  déplacer `isApiError` sans réexport (~110 importeurs à modifier).
- **Réversible** : oui.

### C-28-2-3 — Une `Error` ou une valeur inconnue rend le générique traduit, et se journalise

- **Contexte** : le message d'une `Error` du navigateur ou du code est technique et non traduit (« Failed to
  fetch », « No candidate for txId=5 », `ReconciliationProposals.svelte:206/:224`). Une seule `Error` à message
  destiné à l'utilisateur existe (`admin-backup.api.ts:39`), et son écran ne passe pas par l'utilitaire.
- **Retenu** : `i18nMsg('error-unexpected', 'Erreur inattendue.')` + `console.error('[errorMessageOf]', err)`.
- **Écartées** : afficher `err.message` d'une `Error` (texte technique, langue imprévisible) ; ne rien journaliser
  (perte du diagnostic).
- **Réversible** : oui.

### C-28-2-4 — Les messages fabriqués par le client sont traduits à la source

- **Contexte** : `NETWORK_ERROR`, `TIMEOUT`, `UNKNOWN_ERROR` portent des messages français en dur, lus par 177
  lectures de `.message` du frontend. Les traduire dans l'utilitaire seul n'aurait servi que ses vingt sites.
- **Retenu** : `i18nMsg` dans `toFetchApiError` et dans une fonction `serverStatusMessage(status)` qui factorise les
  trois `Erreur serveur (${res.status})` ; replis = textes actuels, à l'identique. Le cycle `api-client.ts` ↔
  `i18n.svelte.ts` est bénin (usages à l'appel seulement), et un test le prouve.
- **Écartées** : traduction dans l'utilitaire seul ; un module de catalogue séparé pour casser le cycle (refonte
  d'`i18n.svelte.ts` sans gain mesuré).
- **Réversible** : oui.

### C-28-2-5 — `SETUP_REQUIRED` et `UNAUTHENTICATED` restent en français

- **Contexte** : ces deux messages (`api-client.ts:407`, `:427`, `:504`, `:521`) précèdent une redirection.
- **Retenu** : hors périmètre, écrits en angle mort.
- **Écartée** : les traduire aussi (deux clés pour des textes qu'aucun écran connu n'affiche).
- **Réversible** : oui.

### C-28-2-6 — Les préfixes français en dur d'A3 et A4 disparaissent

- **Contexte** : « Erreur chargement profils : » et « Profil introuvable : » sont en dur, et le second est faux
  pour une panne réseau.
- **Retenu** : `errorMessageOf(err)` seul.
- **Écartée** : deux clés à variable `{ $message }` (le contexte est déjà donné par l'écran).
- **Réversible** : oui.

### C-28-2-7 — La garde est un test Vitest qui lit les sources, non une règle ESLint

- **Contexte** : le dépôt tient ses gardes de sources en Vitest (`lib/shared/i18n-*.test.ts`), exécutées par
  `npm run test:unit` en CI ; aucune règle ESLint maison n'existe.
- **Retenu** : `lib/shared/erreurs-en-texte.test.ts`, règles G1-G7, exemption `console` par critère, planchers
  anti-muet, auto-tests du détecteur.
- **Écartée** : un plugin ESLint (outillage neuf, sans précédent au dépôt).
- **Réversible** : oui.

### C-28-2-8 — E2E par interception `page.route`

- **Contexte** : provoquer un vrai refus du verrou des livres verrouillerait la société de `kesh_e2e`, partagée par
  toutes les specs (pollution d'état, cf. le huitième échec instable).
- **Retenu** : interception de la requête (réponse 400 JSON ; abandon réseau pour le second scénario), qui exerce
  le vrai `parseErrorResponse` / `toFetchApiError` du navigateur.
- **Écartée** : refus réel suivi d'un déverrouillage (deux écritures, un nettoyage qui peut échouer).
- **Réversible** : oui.

### C-28-2-9 — Dérogation à la règle de découpage préventif (six modules)

- **Contexte** : `shared/utils`, `settings`, `bank-import`, `bank-accounts`, `reconciliation`, `reports`.
- **Retenu** : une seule story — la logique est dans `shared/utils`, le reste est un remplacement d'une ligne par
  site ; scinder laisserait la garde rouge sur la sous-story pilote. Revue site par site exigée au prompt de revue.
  **Signal déclaré au Project Lead.**
- **Écartée** : 28-2a (utilitaire + garde) / 28-2b (vingt sites).
- **Réversible** : oui (découper avant `dev-story` si Guy le décide).

### C-28-2-10 — La classe D devient une issue de suivi, P3

- **Contexte** : ≈ 23 sites dans sept modules (`credit-notes`, `invoices` ×5 écrans, `payment-batches`,
  `supplier-invoices`) n'affichent rien quand la valeur attrapée n'est pas une `ApiError` ; cas rare (corps de
  succès non JSON, erreur de programmation).
- **Retenu** : inventoriée et comptée dans la fiche ; issue demandée à l'orchestrateur, P3.
- **Écartée** : la traiter ici (découpage préventif, et défaut distinct de #520).
- **Réversible** : oui.

### C-28-2-11 — Un test par fichier de la classe B, la garde pour le reste

- **Contexte** : `settings/api-keys` et `reports` n'ont pas de test de page.
- **Retenu** : un test par fichier B sur un rejet non-`ApiError` ; créer `api-keys-page.test.ts` ; pour `reports`,
  un cas dans un test existant s'il y en a un, sinon la garde AC6 seule, écrit au Dev Agent Record.
- **Écartée** : un test par site (vingt montages pour une ligne chacun).
- **Réversible** : oui.

### C-28-2-12 — La section `[0.13.3] — Non publié` du CHANGELOG est créée par cette branche si absente

- **Retenu** : union avec 28-1 et 28-3 à la fusion.
- **Réversible** : oui.

### C-28-2-13 — Les littéraux français en dur voisins sont signalés, non traités

- **Contexte** : `'ID invalide'`, le message réseau de `routes/login/+page.svelte:54`, les préfixes d'`InvoiceForm`.
- **Retenu** : angle mort écrit ; l'orchestrateur vérifie si une issue i18n les couvre.
- **Réversible** : oui.

### C-28-2-14 — Recette visuelle : quatre captures hors dépôt

- **Retenu** : section `settings-books-lock`, thèmes clair et sombre, avant et après, dans
  `/home/gcorbaz/devel/kesh-gate-logs/28-2-captures/`, citées au Dev Agent Record et au prompt de revue (action B2
  de la rétrospective de l'Epic 27).
- **Écartée** : captures versionnées (poids du dépôt, aucune garde ne les relit).
- **Réversible** : oui.
