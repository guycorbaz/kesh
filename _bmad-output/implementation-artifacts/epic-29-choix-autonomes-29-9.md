# Epic 29 — registre des choix autonomes de la Story 29-9

*Registre propre à la story, à verser **par union** au registre de l'epic (`epic-29-choix-autonomes.md`). Fiche :
`29-9-trois-petits-correctifs.md`. Issues : #649, #655, #646. Relevés sur `fcd2ae75`.*

---

### C-29-9-1 — Une story pour trois correctifs, sans découpage

- **Contexte** : story ajoutée à l'Epic 29 sur décision de Guy (2026-10-14) pour que le tag v0.13.4 compte moins
  d'issues ouvertes que celui de la v0.13.3. Trois défauts P3 indépendants : la liste de la réconciliation tronquée à
  100 (#649), une bannière d'erreur périmée (#655), une suppression sans confirmation (#646).
- **Retenu** : une seule story, sur le précédent de la 28-8 (cinq correctifs). Modules au sens de la règle : le
  frontend (un paquet npm, trois zones : réconciliation, import bancaire, contacts), `kesh-api` (un test et deux
  doc-comments, aucun code de production), la documentation, les catalogues i18n — **quatre**, sous la limite de cinq.
  Les correctifs ne se lisent ni ne s'appellent l'un l'autre ; aucun ne change une règle métier ; la revue se fait
  correctif par correctif.
- **Écartée** : trois stories (trois cycles de CI et trois rebases sur le même composant que la 29-3, la 29-6 et la
  29-8, pour le même diff).
- **Réversible** : oui — un correctif fautif sort du périmètre (son issue reste ouverte) plutôt que de retenir les deux
  autres.
- *Annotation du 2026-10-14 (validation P2, R1 = F1)* : la réécriture du commentaire de module de `kesh-db`
  (C-29-9-16) ajoute un cinquième module — un commentaire, aucun code ; cinq reste sous le seuil (« plus de 5 »).

### C-29-9-2 — #649 : un seul palier, « jusqu'à 500 », et non un « Charger plus » par cent

- **Contexte** : l'API accepte déjà `limit` jusqu'à `MAX_PROPOSALS_LIMIT = 500` (`routes/reconciliation.rs:67`) ; le
  frontend envoie toujours 100 (`reconciliation.api.ts:18`). Trois gestes possibles : un bouton qui ajoute 100 à
  chaque clic, un bouton qui passe d'un coup au plafond, une vraie pagination serveur.
- **Retenu** : un bouton **Afficher aussi les plus anciennes (jusqu'à 500 en tout)**, qui passe `limit` à 500. Deux
  états seulement (100, puis 500), donc deux branches d'écran et deux tests. Le coût d'un rechargement est
  proportionnel au nombre de transactions **réellement** en attente (une requête de candidates par transaction rendue),
  pas au `limit` : un compte qui en a 150 en charge 150, comme il en chargerait 150 avec deux clics « +100 ».
- **Écartées** : « +100 par clic » (quatre clics pour atteindre le plafond, un état par palier, sans gain de coût) ;
  pagination serveur par décalage ou par curseur (signature de `find_pending_transactions_for_account` changée, donc
  `kesh-db` touché et gate complet en boucle ; un décalage se décale à chaque transaction acceptée, ce qui exige de
  recharger toutes les pages chargées — le périmètre d'une fonctionnalité, pas d'un P3).
- **Réversible** : oui (le palier est une constante ; une pagination serveur peut venir ensuite sans défaire l'écran).

### C-29-9-3 — #649 : au-delà de 500, un avis qui dit comment faire apparaître la suite, sans conseiller le rejet

- **Contexte** : au plafond, `hasMore` peut rester vrai (plus de 500 transactions en attente sur un compte). Le
  plafond est une défense anti-DoS (M9 de la 8-4) qu'on ne lève pas.
- **Retenu** : au plafond, un avis **sans bouton** : les 500 plus récentes sont affichées, le maximum de la page ; les
  plus anciennes apparaîtront à mesure que celles-ci seront acceptées, affectées manuellement ou éclatées. C'est le
  moyen d'accès, vrai au code (la liste est triée `booking_date DESC, id DESC` et ne rend que les `pending` non
  rejetées), et déjà écrit au manuel (`user-manual.tex:2057`) et à `api-external.md:594`. **Le rejet n'est pas
  nommé** : une transaction rejetée reste `pending` et bloque la clôture du compte bancaire (fiche 29-6, #670) — le
  conseiller pour « faire de la place » créerait le défaut que la 29-8 répare.
- **Écartée** : lever ou relever le plafond (le coût d'un rechargement de 1000 transactions à une requête chacune est
  ce que M9 interdit) ; une issue pour la pagination au-delà de 500 — **laissée à l'orchestrateur** (rapport) : l'écran
  dit vrai et donne un moyen, ce que #649 demande ; une pagination serait une amélioration, pas un défaut.
- **Réversible** : oui.

### C-29-9-4 — #649 : le plafond se reflète par une constante du frontend, figée par un test Rust

- **Contexte** : le frontend doit savoir qu'il est au plafond, sans quoi il afficherait un bouton sans effet (le
  serveur ramène silencieusement `limit` à 500).
- **Retenu** : `PROPOSALS_DEFAULT_LIMIT = 100` et `PROPOSALS_MAX_LIMIT = 500` exportées de `reconciliation.api.ts`
  (le paramètre par défaut de `getProposals` lit la première — DRY), doc-comment qui renvoie à
  `DEFAULT_PROPOSALS_LIMIT` / `MAX_PROPOSALS_LIMIT` ; côté Rust, doc-comments réciproques et **un test d'intégration**
  qui fige le plafond (`limit=1000` sur 501 transactions → 500 rendues, `hasMore` vrai) et le second état de l'écran
  (`limit=500` sur 150 → 150, `hasMore` faux). *Annotation du 2026-10-14 (validation P1, F8)* : le second test porte
  sur **exactement 500** transactions (`limit=500` → 500, `hasMore` faux), pour figer la frontière que 150 laissait
  libre (`>=` pour `>`) ; il s'appelle `get_proposals_limit_500_exact_has_no_more`.
- **Écartées** : un champ `limit` effectif dans la réponse (change le contrat de l'API pour un P3) ; déduire le
  plafond de la réponse (`proposals.length < limit demandé`) — juste, mais il faut un clic inutile pour le découvrir.
- **Réversible** : oui.

### C-29-9-5 — #649 : « Afficher aussi les plus anciennes » vide la sélection, comme tout rechargement

- **Contexte** : `load()` remet `selected` à vide (`ReconciliationProposals.svelte:109`), et la fiche 29-6 s'appuie
  sur cet invariant (« `proposals` ne change qu'au `load()`… une transaction ne peut pas perdre sa proposition en
  restant cochée », état (a)).
- **Retenu** : le bouton recharge par le même `load()` ; la sélection est vidée, et le manuel le dit.
- **Écartée** : conserver l'intersection de la sélection (rompt l'invariant (a) de la 29-6, pour un geste rare).
- **Réversible** : oui.

### C-29-9-6 — #649 : un seul déclencheur du rechargement, l'`$effect`

- **Contexte** : `load()` est lancé par `$effect(() => { void load(); })` (`:118-120`) ; il lit `bankAccountId`
  avant son premier `await`, donc l'effet le suit. Lire `limit` au même endroit le fera suivre aussi : un bouton qui
  poserait `limit` **puis** appellerait `load()` lancerait **deux** requêtes.
- **Retenu** : le bouton ne fait que `limit = PROPOSALS_MAX_LIMIT` ; l'effet recharge. Commentaire au site ; test qui
  exige **exactement** deux appels de `getProposals` (montage, clic). Le `{#key selectedId}` de la page
  (`routes/(app)/reconciliation/+page.svelte`) remonte le composant à chaque changement de compte : `limit` repart à
  100 — à écrire dans le commentaire, comme C-15-6-28 l'a fait pour `bankLedgerAccountId`.
- **Écartée** : `untrack` + appel explicite (deux mécanismes pour un).
- **Réversible** : oui.

### C-29-9-7 — #655, premier cas : un numéro de séquence dans `load()` du détail d'import

- **Contexte** : `bank-import/[id]/+page.svelte:47-70` n'a pas de garde : la réponse la plus lente l'emporte, échec
  comme succès.
- **Retenu** : le patron déjà en place dans `ReconciliationProposals.svelte:99-117` (`loadGen`) : `const mySeq =
  ++loadSeq` ; après l'`await`, `detail`, `errorMessage` et `loading` ne s'écrivent que si `mySeq === loadSeq`. La
  remise à `null` en tête (28-5b) reste.
  *Annotation du 2026-10-14 (validation P1, F7 et R8)* : `loading` sort de la garde — voir C-29-9-14 ; `++loadSeq`
  est la première instruction de `load()`, avant la branche « Identifiant invalide » (R7).
- **Écartée** : `AbortController` (l'`apiClient` remplace le `signal` de l'appelant par le sien, `api-client.ts:258-260` ; plus large pour le même effet).
- **Réversible** : oui.

### C-29-9-8 — #655, second cas : prémisse rectifiée — défaut latent, corrigé quand même

- **Contexte** : `bank-import/profiles/[id]/+page.svelte` ne remet jamais `errorMessage` à `null` (exact) ; mais son
  `$effect` ne se rejoue que si `page.params.id` change **sans quitter la route**, et aucun lien de l'application ne
  mène d'un profil à un autre (`git grep "bank-import/profiles/"` : la liste, séparée, et `new` → `goto` vers
  `[id]`, une autre route). Le symptôme n'est **pas atteignable** depuis l'interface au sol.
- **Retenu** : corriger quand même, à l'identique du premier cas, parce que c'est mécanique et que la route le
  permettra dès qu'un lien direct apparaîtra : en tête de l'effet `errorMessage = null`, `isLoading = true`, numéro de
  séquence. `isLoading = true` démonte le formulaire pendant la relecture, si bien que `BankProfileForm` repart du
  nouveau profil et non de l'ancien. Test par un `page` réactif (fixture `.svelte.ts`). L'orchestrateur commente #655
  (prémisse « même symptôme » → « même défaut, latent »).
- **Écartée** : sortir le second cas du périmètre (l'issue resterait ouverte pour trois lignes).
- **Réversible** : oui.
- *Annotation du 2026-10-14 (validation P1, F4)* : la prémisse « son `$effect` ne se rejoue que si `page.params.id`
  change » est **fausse au code** — l'effet dépend du signal `page.params`, que SvelteKit réassigne (objet neuf) à
  chaque navigation dans la route, même à `id` égal (`@sveltejs/kit/src/runtime/client/state.svelte.js:37`,
  `client.js:1888`). Telle quelle, la correction retenue ci-dessus (`isLoading = true` en tête) aurait démonté le
  formulaire et perdu une saisie à chaque navigation de ce genre. Le « Retenu » ci-dessus est conservé pour la trace ;
  il est **remplacé par C-29-9-13** pour la forme de l'effet. Le constat « défaut latent » tient (aucun lien au sol).
  *Précision du 2026-10-14 (validation P2, F5)* : « à chaque navigation » est trop large — à chaque navigation où
  `page_changed` est vrai (`client.js:763-772`), ce qui comprend une navigation à `id` égal qui change l'URL ou les
  données ; la conclusion ne change pas.

### C-29-9-9 — #655 : la classe n'est pas revendiquée fermée

- **Contexte** : d'autres pages peuvent relire sans séquence ou garder une erreur périmée.
- **Retenu** : la story ferme les **deux sites** que nomme #655, et la fiche ne prétend rien de plus (aucun critère ne
  ferme une classe, donc pas d'inventaire clos à tenir). Le Change Log le dira.
- **Écartée** : un inventaire des `load()` de toute l'application (hors du périmètre minimal ; une issue si un site
  atteignable est trouvé en chemin).
- **Réversible** : oui.

### C-29-9-10 — #646 : `window.confirm`, qui nomme la personne

- **Contexte** : le gestionnaire des personnes vit **dans** le dialogue d'édition d'un contact
  (`routes/(app)/contacts/+page.svelte:1099`, à l'intérieur de `Dialog.Root` `:811`). L'issue propose le
  `window.confirm` de `BankProfileList.svelte:30` / `RulesList.svelte:73` ou un dialogue.
- **Retenu** : `window.confirm(i18nMsg('contact-persons-confirm-delete', …, { name }))`, `name` = prénom et nom ; refus
  → rien n'est envoyé. Même patron que six fichiers du dépôt (sept appels, `git grep -n "confirm(" -- '*.svelte'`) ; pas de dialogue imbriqué dans un dialogue bits-ui (piège
  de focus et d'empilement pour un P3). Le texte dit qu'aucun écran ne la rétablit — vrai : la suppression archive
  (`active = FALSE`, `contact_persons.rs:158`, audit `contact_person.archived`) et aucune route ne désarchive.
- **Conséquence assumée** : un dialogue natif n'apparaît pas dans une capture Playwright ; la recette visuelle de ce
  correctif se fait sans capture, et le rapport le déclare.
- **Écartée** : un `Dialog` imbriqué (captures possibles, mais nouveau composant et focus imbriqué).
- **Réversible** : oui.

### C-29-9-11 — Ordre : en dernier de la série de la réconciliation, dans la v0.13.4

- **Contexte** : #649 touche `ReconciliationProposals.svelte`, que modifient la 29-3 (paragraphe de priorité avant la
  table), la 29-6 (case, *Accepter* grisé, indication dans la rangée des boutons) et la 29-8 (troisième bouton de
  ligne, modale), et le passage du manuel `user-manual.tex:2045-2057` que les trois réécrivent.
- **Retenu** : la 29-9 se rebase après elles et se merge en dernier ; son ajout à l'écran se place **après la table**
  (`</table>`, `:401`), zone qu'aucune des trois ne touche. Elle part dans la v0.13.4.
- **Réversible** : oui.
- *Annotation du 2026-10-14 (validation P2, F8)* : « zone qu'aucune des trois ne touche » n'est pas établi pour la
  29-8, qui ajoute une `LinkSettlementModal` à ce composant sans en prescrire l'emplacement (fiche 29-8 `:370`) — son
  emplacement naturel est après `</table>`, à côté des deux autres modales. Règle retenue (AC1) : au rebase, l'avis et
  le bouton se placent **immédiatement** après `</table>`, avant toute instance de modale, la sienne comprise.

### C-29-9-12 — Tests de bout en bout : deux specs neuves, aucun pour #655

- **Contexte** : #649 et #646 changent un parcours ; #655 ne change aucun parcours, et la course qu'il corrige n'est pas
  reproductible de façon déterministe à travers un vrai serveur.
- **Retenu** : `reconciliation-show-older.spec.ts` (import CAMT synthétique de 101 transactions, bouton, 101e visible)
  — *corrigé le 2026-10-14 (validation P1, R5)* : la fiche dit, et c'est ce qui vaut, **501** transactions (paliers 100
  puis 500, avis plafonné), la **150e** absente avant le clic et présente après ; entrées uniques (F11) ;
  et `contact-persons-delete.spec.ts` (refus → la personne reste ; acceptation → elle part, et le serveur ne la rend
  plus). #655 est couvert par Vitest, promesses résolues dans l'ordre inverse.
- **Réversible** : oui.

### C-29-9-13 — #655, second cas : l'effet du profil dépend d'un `$derived` de l'identifiant (validation P1, F4)

- **Contexte** : `page.params` est un `$state.raw` réassigné par `Object.assign(page, …)` à chaque navigation où
  `page_changed` est vrai — URL, erreur, formulaire ou données changés (`@sveltejs/kit` 2.55.0, `state.svelte.js:37`,
  `client.js:763-772`, `:1888` ; sinon `clone_page(page)`, `:707`, garde la référence — précisé en validation P2, F5) ; un effet qui lit `page.params.id` se rejoue donc à
  `id` égal. `BankProfileForm` initialise ses champs **une fois** depuis `existing` (`BankProfileForm.svelte:46-60`) :
  à id changé, il doit être remonté ; à id égal, il ne doit **jamais** l'être (saisie en cours). Décision de
  l'orchestrateur : le geste le plus étroit, sans démontage ni perte de saisie au même id.
- **Retenu** : `const rawId = $derived(page.params.id)` ; l'effet ne lit que `rawId`. Un `$derived` compare par `===`
  (`svelte/src/internal/client/reactivity/equality.js:4-6`, Svelte 5.55.1) : à id égal, il garde sa valeur et l'effet
  ne se rejoue pas. La **chaîne** est dérivée, non le nombre (`NaN !== NaN` relancerait l'effet à chaque
  réassignation de `page.params` sur un id invalide). En tête de l'effet — donc d'un chargement réel — `++loadSeq` (avant la branche « ID
  invalide »), `errorMessage = null`, `isLoading = true` ; garde de séquence sur `profile`, `errorMessage`,
  `isLoading`. `isLoading = true` est **gardé** : établi au code qu'il ne s'exécute qu'à id changé, où il est
  nécessaire. Tests V13 (id égal : un seul appel, saisie intacte), V14, V15, V16 ; fixture qui **réassigne** `params`.
- **Écartées** : `{#key rawId}` autour du formulaire (démonte aussi, mais c'est l'effet qu'il faut borner, et deux
  mécanismes pour un) ; `untrack(() => page.params)` (n'exprime pas « l'id seul ») ; ne pas poser `isLoading = true`
  (le profil 6 s'afficherait avec les champs du 5).
- **Réversible** : oui.

### C-29-9-14 — #655, premier cas : `loading` du détail d'import n'est pas gardé (validation P1, F7 et R8)

- **Contexte** : la fiche initiale gardait `loading` dans le `finally`. Or `load()` ne remet jamais `loading` à
  `true`, et les deux déclencheurs de relecture ne sont rendus que dans `{:else if detail}` (`[id]/+page.svelte:96`),
  donc après la première lecture : un `finally` périmé écrit `false` sur `false`.
- **Retenu** : garde de séquence sur `detail` et `errorMessage` seulement ; `loading = false` reste nu dans le
  `finally`, avec un commentaire qui dit pourquoi et qu'il faudra le garder le jour où `load()` le remettra à `true`.
- **Écartée** : garder `loading` (garde intuable : aucun test ne peut la rougir) ; ajouter `loading = true` à `load()`
  pour la rendre observable (démonterait les lignes à chaque relecture — un changement d'écran hors de #655).
- **Réversible** : oui.

### C-29-9-15 — #649 : le saut de défilement au clic est déclaré, non corrigé (validation P1, F9)

- **Contexte** : `{#if loading}` (`ReconciliationProposals.svelte:266-269`) remplace la liste par le message de
  chargement pendant tout rechargement ; au clic sur *Afficher aussi les plus anciennes*, en bas de page, la page
  remonte.
- **Retenu** : déclarer (AC2, Dev Agent Record) ; `load()` inchangé. Le même saut existe déjà après *Accepter* et
  *Rejeter*.
- **Écartée** : ne passer `loading` à `true` que si la liste est vide — la liste resterait cliquable pendant un
  rechargement pour **tous** les déclencheurs (dont un lot en vol), un changement de comportement hors du P3.
- **Réversible** : oui.

### C-29-9-16 — #649 : le commentaire de module de `kesh-db` n'est pas réécrit (validation P1, R3/F3)

- **Contexte** : `crates/kesh-db/src/repositories/reconciliation.rs:18-19` dit « paginées (limit hardcodé 100 au
  caller, cf. L24) », trouvé par le grep élargi.
- **Retenu** : laissé tel quel, cité au Dev Agent Record : le défaut de 100 est bien fixé par la route appelante, et
  la story ne change aucun code de production Rust — le commentaire reste vrai. Le préciser ferait entrer un
  repository `kesh-db` dans le diff pour un commentaire, et avec lui l'exception *kesh-db* (gate complet à chaque
  patch de la boucle de revue).
- **Écartée** : le réécrire (« défaut 100, plafond 500, fixés par la route »).
- **Réversible** : oui.
- *Annotation du 2026-10-14 (validation P2, R1 = F1) — **décision de l'orchestrateur : le « Retenu » ci-dessus est
  renversé**, conservé pour la trace.* Les deux raisons étaient fausses. (1) « Le commentaire reste vrai » : il est
  faux **depuis la Story 8-4** (`22fd6d4a`, le commit qui l'a écrit) — l'appelant ne code pas 100 en dur, la route
  prend `?limit=` de 1 à 500, 100 par défaut (`routes/reconciliation.rs:598-602`), et passe `limit + 1` (`:622-626`) ;
  après la story, l'écran demande lui-même 500. Un site du symptôme de #649 était donc trié sur un motif faux.
  (2) « L'exception *kesh-db* : gate complet à chaque patch de la boucle » : surestimée — CLAUDE.md dit « dès qu'un
  **patch** touche … un repository » ; elle vaut pour **le commit qui touche ce fichier**, celui de T1, et pour un
  patch de revue qui le retoucherait, pas pour toute la boucle. **Retenu désormais** : le commentaire est **réécrit en
  T1** (« `limit` fourni par la route : défaut 100, plafond 500, sentinelle `limit + 1` pour `hasMore` »), le renvoi
  `L24` disparaît ; gate complet au commit de T1 ; aucun code de production Rust ne change. **Écartée** désormais : le
  laisser comme angle mort assumé (un commentaire faux qu'on sait faux, pour éviter un gate).

### C-29-9-17 — #655, second cas : `onUpdated` gardé, sur l'identifiant courant et non sur une fermeture (validation P2, F2)

- **Contexte** : `bank-import/profiles/[id]/+page.svelte:33-35`, `onUpdated(updated)` écrit `profile` sans condition.
  Une sauvegarde du profil 5 qui se termine après une navigation vers 6 (et après la lecture de 6) écrit `profile = p5` ;
  le formulaire monté garde les champs de 6 (initialisés une fois, `BankProfileForm.svelte:46-60`) mais `existing` = p5,
  et la sauvegarde suivante écrit les valeurs de 6 dans le profil 5. Latent, comme le reste du second cas (aucun lien
  d'un profil à l'autre au sol). **Décision de l'orchestrateur** : `onUpdated` reçoit la même garde de séquence que le
  chargement — une sauvegarde qui finit après une navigation vers un autre profil n'écrit plus `profile`.
- **Retenu** : la garde se pose **dans** `onUpdated`, évaluée à l'appel : `if (updated.id !== Number(rawId)) return;`
  puis `profile = updated`. C'est la forme que prend la décision de l'orchestrateur pour une écriture que la page ne
  lance pas elle-même : la page ignore toute écriture qui ne relève plus du profil affiché, comme `mySeq === loadSeq`
  ignore une lecture périmée. Test V17, mutation M19.
- **Écartées** : un numéro de séquence capturé dans une fermeture passée au formulaire (`onSuccess={onUpdatedFor(seq)}`)
  — **ne tient pas** : Svelte 5.55.1 compile `let { existing, onSuccess } = $props()` suivi d'un appel après `await` en
  `$$props.onSuccess?.($$props.existing)` (compilation d'un composant d'essai, 2026-10-14, hors dépôt), si bien que le
  formulaire démonté de 5 appellerait le gestionnaire **courant**, celui de 6, et que la garde passerait ; modifier
  `BankProfileForm` pour qu'il capture l'identifiant au départ de la sauvegarde (hors des deux sites de #655) ; sortir
  le site du périmètre (C-29-9-9) — l'orchestrateur a tranché pour la garde.
- **Conséquence assumée** : retour de 6 à 5 avant la fin de la sauvegarde de 5 → la réponse est acceptée ; c'est juste
  (même profil, valeurs enregistrées), et une garde strictement séquentielle l'aurait refusée sans gain.
- **Réversible** : oui.

### C-29-9-18 — #655, second cas : la dépendance synchrone de `getBankProfile` s'écrit au site, sans `untrack` (validation P2, F6)

- **Contexte** : « l'effet ne dépend que de l'identifiant » (C-29-9-13) suppose aussi que la partie synchrone de
  `getBankProfile(id)` ne lit aucun signal. Vrai au sol (`request` → `buildHeaders` → `fetchWithRetry` →
  `fetchWithTimeout`, `api-client.ts:181-199`, `:267-280`, `:329-336`, `:392-399`) ; V13 mocke l'API et ne le verrait
  pas changer.
- **Retenu** : un commentaire au site qui écrit cette dépendance et nomme le remède (`untrack(() => getBankProfile(id))`)
  pour le jour où une lecture réactive entrerait dans `request`.
- **Écartée** : `untrack` dans le code dès maintenant — garde qu'aucun test ne peut rougir au sol (même règle que
  C-29-9-14 pour `loading`).
- **Réversible** : oui.
