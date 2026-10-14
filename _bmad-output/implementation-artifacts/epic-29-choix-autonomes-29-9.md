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
  (`limit=500` sur 150 → 150, `hasMore` faux).
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

### C-29-9-12 — Tests de bout en bout : deux specs neuves, aucun pour #655

- **Contexte** : #649 et #646 changent un parcours ; #655 ne change aucun parcours, et la course qu'il corrige n'est pas
  reproductible de façon déterministe à travers un vrai serveur.
- **Retenu** : `reconciliation-show-older.spec.ts` (import CAMT synthétique de 101 transactions, bouton, 101e visible)
  et `contact-persons-delete.spec.ts` (refus → la personne reste ; acceptation → elle part, et le serveur ne la rend
  plus). #655 est couvert par Vitest, promesses résolues dans l'ordre inverse.
- **Réversible** : oui.
