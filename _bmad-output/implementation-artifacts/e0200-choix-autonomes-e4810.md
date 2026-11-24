# Registre des choix autonomes — Story 48-10 (clé `e4810`)

Développement en autonomie (Opus 5.5), release 0.20.0, Epic 48. Fiche :
`48-10-montants-ambigus-reglages-filtres.md`. Chaque choix : contexte, option retenue, autres options, réversibilité.

## C-e4810-1 — Remise à zéro de `formError` à la saisie du prix : comparée à tous les messages du prix

- **Contexte** : D1 écrit `if (formError === priceMessage) formError = ''` dans le gestionnaire `oninput` du prix. En
  Svelte 5, `oninput` est un événement **délégué** (écouté à la racine), alors que `bind:value` pose un écouteur direct sur
  l'élément : `formPrice` est déjà mis à jour quand `oninput` s'exécute, et `priceMessage` (dérivé) vaut celui de la
  **nouvelle** saisie. Après Entrée sur « 1.250 » puis correction en « 1'250 », `priceMessage` vaut `''`, la comparaison
  échoue, et le message ambigu périmé reste en rouge au pied — exactement le défaut que l'AC12 (V2-3) et M19 visent.
- **Retenu** : `onPriceInput()` efface `formError` s'il égale **l'un des trois** messages du prix (`ambiguous`,
  `negative`, `invalid`, calculés par `priceMessageFor`). Même intention que la fiche (le refus du serveur et le message
  des taux de TVA restent), sans dépendre de l'ordre des écouteurs.
- **Écartées** : la formule littérale de la fiche (rouge au test d'AC12) ; un drapeau `formErrorFromPrice` posé à chaque
  site qui écrit `formError` (cinq sites, oubli facile).
- **Réversibilité** : locale à `products/+page.svelte`.

## C-e4810-2 — Un seul paragraphe pour le refus d'une borne du filtre

- **Contexte** : D4 veut sous la borne « `amountEntryMessage(problem, 4)` suivi de
  `journal-entries-filter-amount-not-applied` ».
- **Retenu** : un seul `<p id="filter-amount-min-problem" role="alert">` qui porte les deux phrases, séparées d'une
  espace ; `aria-describedby` nomme cet `id`.
- **Écartées** : deux paragraphes (deux `id` à nommer, deux annonces `alert`).
- **Réversibilité** : locale à `journal-entries/+page.svelte`.

## C-e4810-3 — Le minimum : `aria-describedby` nomme le refus **et** l'aide

- **Contexte** : D3 prend pour modèle l'écart admis (`groupMax`), dont `aria-describedby` nomme le refus et l'aide
  (revue P1 de la 43-7, B1-4).
- **Retenu** : l'aide du minimum reçoit un `id` (`{uid}-minimum-hint`) ; `aria-describedby` vaut
  `"{uid}-minimum-error {uid}-minimum-hint"` en erreur, `"{uid}-minimum-hint"` sinon. `aria-invalid` est toujours présent
  (`"false"` hors erreur), comme le patron.
- **Écartées** : ne nommer que le refus (l'aide, qui dit « vide = aucun seuil », ne serait plus lue en erreur).
- **Réversibilité** : locale à `settings/invoicing/+page.svelte`.

## C-e4810-4 — Captures de l'E2E préfixées `e4810-`

- **Contexte** : l'aide `capture` de `montants-ambigus.spec.ts` préfixait toujours `e3618a-`.
- **Retenu** : un paramètre `prefixe` (`'e3618a'` par défaut, `'e4810'` pour les tests neufs) ; les captures de la 36-18a
  gardent leur nom.
- **Réversibilité** : triviale.

## C-e4810-5 — Les tests de page de la 48-10 démontent les rendus précédents

- **Contexte** : `@testing-library/svelte` ne nettoie pas seul sans `globals` ; dans `products-page.test.ts`, les dialogues
  ouverts par les blocs précédents (48-15) restent montés et portent eux aussi un `#form-price`.
- **Retenu** : `beforeEach(() => cleanup())` dans le bloc de la 48-10 ; `afterEach(cleanup)` dans le fichier neuf
  `dunning-page.test.ts`. Les blocs existants ne changent pas.
- **Réversibilité** : triviale.

## C-e4810-6 — Revue P1 (B4, B5) : une lecture « au centime » pour les frais et le minimum, dans un module neuf

- **Contexte** : le message illisible des frais et du minimum dit « 2 décimales au plus », mais la lecture du journal en
  admettait quatre ; « 0.125 » ou « 10.1250 » partaient et le serveur les refusait dans un autre bloc. Le minimum écrivait
  aussi trois fois le même prédicat.
- **Retenu** : `features/journal-entries/amount-entry-centimes.ts` (`readCentimesEntry`, `centimesEntryProblem`) : la
  lecture du journal, puis « deux décimales significatives au plus » (illisible sinon) ; la charge est ramenée au centime
  quand la saisie écrit des zéros au-delà (« 10.0100 » → `10.01`, « 5.0000 » → `5.00`), parce que le serveur des frais
  compte les décimales écrites (`scale_within(fee, 2)` sans `normalize`). Aucune forme acceptée par le serveur n'est
  refusée ; un minimum rouvert à `5.0000` repart `5.00`, même valeur. Un seul prédicat (`minimumProblem`) commande le
  message, le bouton et la garde de `save()`.
- **Écartées** : modifier `balance.ts` (protégé par D6 et AC7) ; garder la lecture à quatre décimales et dire l'écart en
  dette (le message resterait faux).
- **Réversibilité** : locale à deux écrans et un module.

## C-e4810-7 — Revue P1 (B1, E1) : la saisie du prix efface tout message du pied sauf ceux du nom et des taux de TVA

- **Contexte** : C-e4810-1 n'effaçait que les trois refus du prix sous le champ ; « Le prix est obligatoire » et un refus du
  serveur sur le prix (plafond, échelle) restaient périmés au pied.
- **Retenu** : liste fermée des messages qui ne viennent pas du prix (nom vide, nom trop long, doublon de nom, taux de TVA
  en chargement ou en échec) ; tout autre message s'efface à la saisie du prix. Un refus du serveur d'une autre cause
  s'efface aussi : la soumission suivante le redit s'il tient encore.
- **Écartées** : mémoriser l'origine de chaque `formError` (cinq sites d'écriture) ; une issue P4 (le correctif est court).
- **Réversibilité** : locale à `products/+page.svelte`.
- **Prémisse de C-e4810-1, constatée** (revue P1, A4) : la mutation `M19-litteral` (formule littérale de D1) rougit le
  test d'AC12 « Entrée sur 1.250, puis correction » (`kesh-gate-logs/e4810/mut/M19-litteral.log`).
