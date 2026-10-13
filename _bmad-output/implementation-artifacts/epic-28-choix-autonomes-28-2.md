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
- **Révisé en validation P1** : classe C passée à la valeur brute (C-28-2-19) ; classe D suivie par #644 (C-28-2-22).

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
- **Révisé en validation P1** : l'issue est ouverte (#644, par l'orchestrateur) ; voir C-28-2-22.

### C-28-2-11 — Un test par fichier de la classe B, la garde pour le reste

- **Contexte** : `settings/api-keys` et `reports` n'ont pas de test de page.
- **Retenu** : un test par fichier B sur un rejet non-`ApiError` ; créer `api-keys-page.test.ts` ; pour `reports`,
  un cas dans un test existant s'il y en a un, sinon la garde AC6 seule, écrit au Dev Agent Record.
- **Écartée** : un test par site (vingt montages pour une ligne chacun).
- **Réversible** : oui.
- **Révisé en validation P1** : plus d'exception pour `reports` — voir C-28-2-21.

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

---

Choix faits à la remédiation de la validation P1 (2026-10-13, Opus 5.5, autonomie). Rapports hors dépôt :
`kesh-gate-logs/28-2-validate-p1-{R,F}.md`.

### C-28-2-15 — Le manuel cite le message réseau ; il reste exact et ne change pas (F1 = R1)

- **Contexte** : `user-manual.tex:1871-1873` cite « Impossible de contacter le serveur… » (import CSV, fichier
  modifié après l'aperçu), citation coupée sur deux lignes — d'où le zéro du `grep` mono-ligne de la fiche.
- **Retenu** : AC11 réécrit ; le passage reste exact en français (repli et texte `fr-CH` identiques, garde
  `i18n-repli-divergent-actif`) ; les manuels `de/en/it` n'ont pas de `.tex` ; manuel et PDF inchangés ; contrôle
  sur texte aplati, `.tex` et PDF, sorties attendues écrites.
- **Écartées** : ajouter « dans la langue de l'installation » au passage (rien ne change pour le lecteur du manuel
  français) ; supprimer la citation (elle aide l'utilisateur à reconnaître le cas).
- **Réversible** : oui.

### C-28-2-16 — Portée des liaisons dans la garde, et échec sur portée incalculable (F2)

- **Contexte** : l'AC6 disait « recueille les liaisons » sans borner leur portée ; `corpsDeFonction` rend `null` sur
  des accolades déséquilibrées, et un `null` sauté en silence est le précédent de la 23-1a.
- **Retenu** : portée = bloc (`corpsDeFonction`) pour `catch` et `.catch` à bloc ; pour un `.catch` à
  corps-expression, de `=>` à la parenthèse qui ferme `.catch(`, même saut des littéraux — par une fonction
  `blocEquilibre(source, ouvrante, ouvre, ferme)` exportée d'`i18n-literal-reader.js`, dont `corpsDeFonction`
  devient un appel (les outils de saut des littéraux y sont privés) ; une portée `null` fait échouer la garde en
  nommant la ligne ; auto-tests accolades dans chaîne / gabarit / regex.
- **Écartées** : chercher les règles dans tout le fichier (faux positifs sur les `err` d'autres portées) ; sauter
  une portée incalculable avec un avertissement (garde muette) ; recopier la boucle dans la garde, ou exporter les
  trois outils privés du lecteur (copie de logique, ou surface publique élargie pour un seul appelant).
- **Réversible** : oui.

### C-28-2-17 — Planchers anti-muet par forme de liaison (F3)

- **Contexte** : un plancher global de 160 laissait passer la perte de toutes les liaisons `.catch` (5) ou typées (4).
- **Retenu** : fichiers ≥ 310 (317), `catch` non typé ≥ 150 (159), typé ≥ 4 (4), `.catch` lié ≥ 5 (5), total ≥ 160
  (168) ; les petites formes au plancher exact ; un plancher qui rougit se baisse avec sa raison écrite.
- **Écartée** : bornes exactes partout (rougiraient à chaque `catch` ajouté ou retiré par une story ordinaire).
- **Réversible** : oui.

### C-28-2-18 — G3 interdit `instanceof Error` sous toute forme, dans tout le fichier (F8)

- **Contexte** : G3 ne visait que la forme ternaire ; `if (e instanceof Error) msg = e.message; else msg = repli;`
  n'était vue par aucune règle ; et la forme peut vivre hors d'une portée de liaison (`.reason` d'`allSettled`,
  fonction intermédiaire).
- **Retenu** : toute occurrence hors commentaires et hors appel `console`, dans tout le fichier ; à la fin de la
  story il n'en reste aucune (les 9 lignes de code disparaissent avec A, B et C).
- **Écartée** : garder la seule forme ternaire (laisse passer la forme `if`, qui affiche le même texte technique).
- **Réversible** : oui.

### C-28-2-19 — Classe C : les journalisations passent la valeur brute (F5 = R8)

- **Contexte** : `hooks.client.ts:27`, `auth.svelte.ts:301`, `api-client.ts:162` journalisent
  `error instanceof Error ? error.message : String(error)` : une `ApiError` y devient « [object Object] ».
- **Retenu** : `console.*('[…]:', error)` — une ligne par site ; la console affiche l'objet (`code`, `message`,
  `status`). L'exemption `console` de la garde reste (auto-tests). Classe C : 3 → 0.
- **Écartées** : laisser en l'état (diagnostic perdu, alors que `errorMessageOf` prend soin de le conserver) ;
  un formateur de journal dédié (code neuf pour trois lignes).
- **Réversible** : oui.

### C-28-2-20 — `handleDelete` de `BankProfileList.svelte` reçoit un `try/catch` (R10)

- **Contexte** : `await deleteBankProfile(id)` sans `try/catch` : un refus devient un rejet non géré, sans message ;
  même fichier qu'A3.
- **Retenu** : `try { delete; load } catch (err) { notifyError(errorMessageOf(err)); }` — notification, la liste
  restant affichée ; test et mutation M23.
- **Écartées** : `errorMessage = …` (remplacerait toute la liste) ; l'angle mort seul (le fichier est touché, le
  correctif tient en trois lignes).
- **Réversible** : oui.

### C-28-2-21 — Tests de page neufs pour `reports` et `api-keys`, sans exception (R7)

- **Contexte** : l'AC9 exigeait un test par fichier B puis exceptait `reports` ; ni `reports` ni `api-keys` n'ont de
  test de page (`reports.api.test.ts` teste l'API).
- **Retenu** : `api-keys-page.test.ts` (B6) et `reports-page.test.ts` (B15, branche `REPORT_PERIOD_…` préservée) ;
  mutations M21, M22.
- **Écartée** : la garde AC6 seule pour `reports` (elle voit la forme, pas le rendu).
- **Réversible** : oui.

### C-28-2-22 — Classe D hors périmètre, suivie par #644 (F4, R2, R3)

- **Contexte** : décision de l'orchestrateur après la validation P1 — la correction n'est pas mécanique (silences
  voulus, `invoices/[id]/+page.svelte:186` et `:483`), et l'intégrer porterait la story de 6 à au moins 10 modules.
  Il a ouvert l'issue **#644** (bug, P3).
- **Retenu** : la fiche cite la classe D comme angle mort assumé, avec renvoi `refs #644` (PR comprise), et les deux
  silences voulus ; **aucune** autre issue de suivi n'est demandée.
- **Écartée** : intégrer la classe D (règle G8, ~25 fichiers, dérogation au découpage intenable).
- **Réversible** : oui.

### C-28-2-23 — La traduction vaut une fois le catalogue chargé ; CHANGELOG et AC10 le disent (F6 ≈ R5)

- **Contexte** : le catalogue est derrière l'authentification et chargé à l'`onMount` racine si une session est
  ouverte : `/login`, un rechargement serveur injoignable et la navigation après connexion restent en français ;
  l'E2E en `fr-CH` ne distingue pas un texte traduit d'un littéral.
- **Retenu** : angle mort n° 5 ; phrase du CHANGELOG restreinte ; l'AC10 écrit que le scénario 2 ne prouve pas la
  traduction (les tests unitaires au catalogue allemand le font) ; l'orchestrateur vérifie si une issue couvre la
  limite préexistante du chargement du catalogue.
- **Écartées** : charger le catalogue après la connexion (hors sujet, touche l'authentification) ; une instance E2E
  en `de-CH` (montage neuf pour un contrôle que l'unitaire fait déjà).
- **Réversible** : oui.

### C-28-2-24 — Une `ApiError` dont le `message` n'est pas une chaîne rend le générique (F7)

- **Contexte** : `isApiError` ne vérifie que `code` et `status` ; un `message` `undefined` ou numérique passerait.
- **Retenu** : ligne explicite au tableau de l'AC1 et cas de test ; `isApiError` lui-même n'est pas durci (110
  importeurs, comportement inchangé hors de l'utilitaire).
- **Écartée** : durcir `isApiError` (changerait l'aiguillage de sites que la story ne touche pas).
- **Réversible** : oui.
