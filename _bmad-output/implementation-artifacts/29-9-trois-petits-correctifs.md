# Story 29.9 : Trois petits correctifs — la suite de la liste de réconciliation, une bannière périmée, une suppression sans confirmation

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `fcd2ae75` (`main`,
kickoff de l'Epic 29), branche `story/29-9`, worktree `/home/gcorbaz/devel/kesh-29-9`. Validation P1 (Sonnet,
lentilles R et F : 0/0/4/5 et 0/0/5/6) remédiée le 2026-10-14 ; validation P2 (Opus, lentilles R et F : 0/0/1/8
et 0/0/2/7) remédiée le 2026-10-14 ; passe ciblée P3 à lancer (prompt `29-9-validate-prompt-p3-ciblee.md`). Choix
consignés au registre `epic-29-choix-autonomes-29-9.md`, **C-29-9-1 à C-29-9-18**.

10 AC, T0-T9. Trois issues, chacune **entièrement** réglée ou sortie du périmètre : [#649], [#655], [#646] — toutes
`bug` et `P3`.

⛔ **Consigne pour l'orchestrateur — la PR, et elle seule, porte `closes #649 #655 #646`** (corps de la PR ; le dépôt
merge en squash, et le squash recopie le corps des commits). Les commits de la branche portent **`refs #N`**, jamais
`closes`/`fixes`, même en prose ou au futur (B4 de la rétrospective 28). Avant la PR : `git log main..HEAD --format=%B
| grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?)\b"` doit être vide. Après merge, `gh issue view <N> --json state`
pour les trois.

## Story

**As a** Guy, qui veut que chaque release ait moins d'issues ouvertes que la précédente,
**I want** fermer trois défauts déjà diagnostiqués, chacun à son minimum,
**so that** le décompte baisse sans risque : la réconciliation dit qu'il reste des transactions et permet de les voir,
une page rechargée avec succès ne reste plus masquée par une erreur ancienne, et une personne de contact ne s'efface
plus d'un clic malheureux.

**Ajoutée à l'Epic 29 le 2026-10-14 sur décision de Guy**, pour tenir la règle de release de la v0.13.4 (moins
d'issues ouvertes qu'au tag v0.13.3 : 138).

**Effet sur le décompte** (fichier d'epic, § *Le décompte*) : les trois portent `bug` → **−3 issues, −3 bugs**.

## Le constat, au sol (`fcd2ae75`)

| Issue | Constat | Preuve relevée |
|---|---|---|
| [#649] | `GET /reconciliation/proposals` rend au plus `limit` transactions (défaut 100, plafond 500) et `hasMore` ; le frontend envoie toujours 100 et **ne lit pas** `hasMore` | `crates/kesh-api/src/routes/reconciliation.rs:63` (`DEFAULT_PROPOSALS_LIMIT = 100`), `:67` (`MAX_PROPOSALS_LIMIT = 500`), `:598-602` (filtre `> 0`, `min(MAX)`), `:618-631` (`limit + 1`, `has_more`, `truncate`) ; `:84-90` doc-comment de `has_more` : « v0.1 : structure présente mais pas de UI dédiée » ; `frontend/src/lib/features/reconciliation/reconciliation.api.ts:16-26` (`limit = 100`) ; `ReconciliationProposals.svelte:106` (`getProposals(bankAccountId)`) ; `git grep -n hasMore -- frontend/src ':!*.test.ts'` → seul `reconciliation.types.ts:55` |
| [#649] (tri) | Les plus récentes d'abord ; seules les `pending` non rejetées | `crates/kesh-db/src/repositories/reconciliation.rs:288-311` (`status = 'pending' AND auto_match_rejected_at IS NULL ORDER BY booking_date DESC, id DESC LIMIT ?`) |
| [#649] (textes) | La limite est déjà **écrite** à quatre endroits, qui resteront vrais ou se réécrivent | `docs/manual/fr/user-manual.tex:2045` (« les 100 plus récentes au plus »), `:2057` (« qui ne montre que les 100 plus récentes en attente : une transaction plus ancienne n'y reparaît qu'une fois les plus récentes traitées ») ; `docs/api-external.md:594` ; `reconciliation-unreject-success` ×4 (`fr-CH/messages.ftl:867`, `de/en/it-CH:819`) ; `CHANGELOG.md` § `[0.13.3]` (publié : **ne se touche pas**) |
| [#649] (test) | Le défaut de 100 et `hasMore = true` sont testés ; le **plafond** de 500 ne l'est pas | `crates/kesh-api/tests/reconciliation_e2e.rs:2065-2116` (`get_proposals_paginates_at_100_default`, 150 transactions) ; `grep -n "limit=" crates/kesh-api/tests/reconciliation_e2e.rs` → aucun appel avec `limit` |
| [#655] cas 1 | `load()` du détail d'import n'a **aucune** garde de séquence : la réponse la plus lente l'emporte | `frontend/src/routes/(app)/bank-import/[id]/+page.svelte:47-70` ; relectures lancées par `UnrejectTransactionAction` (`:160`) et `CancelReconciliationDialog` (`:176`) ; le patron de garde existe à côté : `ReconciliationProposals.svelte:99-117` (`loadGen`) |
| [#655] cas 2 | `bank-import/profiles/[id]/+page.svelte` ne remet jamais `errorMessage` à `null` ni `isLoading` à `true` ; aucune garde de séquence | même fichier, l. 11 (déclaration), 16 et 28 (écritures), 44 (lecture) ; l'`$effect` `:13-31` |
| [#655] cas 2 (dépendance de l'effet) | L'effet lit `page.params.id` ; or SvelteKit **réassigne** `page.params` (objet neuf) à chaque navigation où `page_changed` est vrai — URL, erreur, formulaire ou données changés —, donc même quand l'`id` ne change pas (une *query string*, des données relues) ; sinon `props.page = clone_page(page)` garde la référence (validation P2, F5) | `node_modules/@sveltejs/kit/src/runtime/client/state.svelte.js:37` (`params = $state.raw({})`), `client.js:763-768` (`page_changed`), `:770-772` (`props.page` neuf, `params` compris), `:707` (`clone_page(page)` sinon), `:1888` (`Object.assign(page, navigation_result.props.page)`), `@sveltejs/kit` 2.55.0 ; `$derived` compare par `===` (`svelte/src/internal/client/reactivity/equality.js:4-6`, Svelte 5.55.1) ; `BankProfileForm.svelte:46-60` initialise son état **une fois** depuis `existing` (validation P1, F4) |
| [#646] | *Supprimer* une personne de contact appelle `deleteContactPerson` sans confirmation | `frontend/src/lib/features/contacts/ContactPersonsManager.svelte:76-83` (`removePerson`), bouton `:112-119` (`contact-person-delete-{id}`) ; le composant est rendu dans le dialogue d'édition (`routes/(app)/contacts/+page.svelte:1097-1100`, dans `Dialog.Root` `:811`) |
| [#646] (sémantique) | « Supprimer » **archive** : `active = FALSE`, trace `contact_person.archived` ; aucune route ne désarchive | `crates/kesh-db/src/repositories/contact_persons.rs:158` ; `crates/kesh-api/src/routes/contact_persons.rs:295` ; routes `crates/kesh-api/src/lib.rs:426-431`, `:794` (création, modification, suppression, liste) |
| [#646] (tests) | Le composant n'a **aucun** test Vitest ni E2E ; l'inventaire des boutons rouges écrit « sans confirmation (#646, #648) » | `git grep -ln "ContactPersonsManager\|contact-person-delete" -- frontend` → le composant, la page, `i18n-harvest.test.ts:35`, `boutons-rouges-inventory.test.ts:231-234` |
| [#646] (manuel) | Le manuel décrit l'ajout de personnes, pas leur suppression | `docs/manual/fr/user-manual.tex:869-874` (`keshnote`) |

### Les prémisses des issues

- **#649 — exacte.** Précision : l'API sait déjà rendre jusqu'à 500 transactions ; seul l'écran est en défaut. Le
  geste minimal est donc **frontend** (plus un test Rust qui fige le contrat, C-29-9-4).
- **#655, cas 1 — exacte.**
- **#655, cas 2 — exacte sur le code, rectifiée sur l'effet** (C-29-9-8). L'issue dit « même symptôme » ; or l'`$effect`
  de la page des profils ne se rejoue que si `page.params.id` change **sans quitter la route**, et aucun lien de
  l'application ne mène d'un profil à un autre (`git grep -n "bank-import/profiles/" -- frontend/src ':!*.test.ts'`
  → `profiles/+page.svelte:12` vers `new`, `profiles/new/+page.svelte:8` → `goto` vers `[id]` — deux routes
  distinctes, donc un montage neuf ; `BankProfileList.svelte:83`, lien de la liste). Le défaut est **latent** ; il
  se corrige quand même (mécanique). **L'orchestrateur commente #655** pour le dire.
  *Annotation du 2026-10-14 (validation P1, F4)* : la phrase « ne se rejoue que si `page.params.id` change » est
  **fausse au code** : l'effet dépend du signal `page.params`, que SvelteKit réassigne (objet neuf) à **toute**
  navigation dans la route, même à `id` égal (tableau ci-dessus). Aucun appelant ne le fait au sol (le défaut reste
  latent), mais un `isLoading = true` en tête d'un tel effet démonterait le formulaire et **perdrait une saisie en
  cours** à chaque navigation de ce genre. D'où AC6 réécrit : l'effet ne dépend plus que de l'identifiant, par un
  `$derived` (C-29-9-13).
  *Précision du 2026-10-14 (validation P2, F5)* : « à **toute** navigation » est trop large — la réassignation n'a lieu
  que si `page_changed` est vrai (`client.js:763-772`) ; une navigation à `id` égal qui change l'URL ou les données
  réassigne bien, ce qui suffit à la conclusion, qui ne change pas.
- **#646 — exacte.** Précision : la suppression est un archivage sans retour possible à l'écran — c'est ce qui rend la
  confirmation utile, et le texte de la confirmation le dit.

## Critères d'acceptation

### #649 — la liste de la réconciliation dit qu'il en reste, et montre la suite

**AC1 — S'il en reste, l'écran le dit et offre la suite (C-29-9-2).**

Dans `ReconciliationProposals.svelte`, branche « liste non vide », **immédiatement après `</table>`** (`:401`),
**avant toute instance de modale** : celles du sol (`<ManualMatchModal` `:403`, `<TransactionSplitModal`) et la
`LinkSettlementModal` que la 29-8 ajoute à ce composant sans en prescrire l'emplacement (fiche 29-8 `:370`, T4) — son
emplacement naturel est le même, à côté des deux autres. Au rebase (T0), l'avis et le bouton se placent donc entre
`</table>` et la première modale, quelle qu'elle soit (validation P2, F8 ; C-29-9-11 annoté) :

- si `hasMore` est vrai **et** `limit < PROPOSALS_MAX_LIMIT` : un paragraphe `data-testid="reconciliation-more"`
  (`reconciliation-labels-more-hidden`, `{ $count }` = `proposals.length`) et un bouton
  `data-testid="reconciliation-show-older-btn"` (`reconciliation-actions-show-older`, `{ $max }` =
  `PROPOSALS_MAX_LIMIT`), `disabled={busy}` — pendant un lot (*Accepter*, *Rejeter*), la liste reste affichée
  (`busy` passe à `true` `:194`/`:239`, `loading` seulement au `load()` final), et un clic rechargerait la liste au
  milieu du lot ; V18 et M21 le prouvent (validation P2, R8) ;
- si `hasMore` est faux : rien (ni avis, ni bouton).

Style : celui des boutons secondaires voisins (`rounded border border-text px-4 py-2`, comme *Rejeter* `:290-298`),
jetons du thème seulement (B2 de l'Epic 27 : pas de couleur codée en dur), lisible en clair et en sombre.

**AC2 — Le bouton passe au plafond, et le plafond tient pour les rechargements suivants (C-29-9-5, C-29-9-6).**

- `let limit = $state(PROPOSALS_DEFAULT_LIMIT)` et `let hasMore = $state(false)` ; `load()` appelle
  `getProposals(bankAccountId, limit)` et, sur une réponse non périmée, pose `hasMore = r.hasMore` avec `proposals`.
- Le bouton ne fait **que** `limit = PROPOSALS_MAX_LIMIT` : l'`$effect` de `:118-120`, qui suit `limit` parce que
  `load()` le lit avant son premier `await`, recharge. ⚠️ **Ne pas appeler `load()` en plus** : deux requêtes.
  Commentaire au site, qui dit aussi que le `{#key selectedId}` de `routes/(app)/reconciliation/+page.svelte:73` remet
  `limit` à 100 à chaque changement de compte (« ⚠️ retirer ce `{#key}` exigerait de remettre `limit` à zéro »).
- Après un lot, une affectation manuelle ou un éclatement, `load()` relit avec le `limit` **courant** (la liste ne
  retombe pas à 100).
- La sélection est vidée par le rechargement (`:109`, inchangé ; C-29-9-5).
- **Saut de défilement, assumé et déclaré (C-29-9-15).** Pendant le rechargement, `{#if loading}` (`:266-269`)
  remplace la table, l'avis et le bouton par le message de chargement — comme il le fait déjà après *Accepter* ou
  *Rejeter* ; le document se raccourcit, la page remonte, puis la liste revient à 500 lignes. `load()` **ne change
  pas** sur ce point : garder la liste affichée pendant un rechargement la laisserait cliquable pendant un lot en vol,
  pour tous les déclencheurs et pas seulement ce bouton. Le Dev Agent Record le déclare avec la capture.

**AC3 — Au plafond, un avis sans bouton, qui ne conseille pas le rejet (C-29-9-3).**

Si `hasMore` est vrai **et** `limit >= PROPOSALS_MAX_LIMIT` : un paragraphe `data-testid="reconciliation-more-capped"`
(`reconciliation-labels-more-capped`, `{ $max }`), **aucun** bouton. Le texte nomme *accepter, affecter
manuellement, éclater* — **pas** *rejeter* (une transaction rejetée reste `pending` et bloque la clôture du compte :
fiche 29-6, #670).

**AC4 — Le contrat se lit d'un seul endroit de chaque côté, et un test Rust le fige (C-29-9-4).**

- `reconciliation.api.ts` exporte `PROPOSALS_DEFAULT_LIMIT = 100` et `PROPOSALS_MAX_LIMIT = 500`, avec un
  doc-comment qui renvoie aux constantes Rust ; `getProposals(bankAccountId, limit = PROPOSALS_DEFAULT_LIMIT)`.
- `routes/reconciliation.rs` : les doc-comments de `DEFAULT_PROPOSALS_LIMIT` / `MAX_PROPOSALS_LIMIT` (`:62-67`)
  nomment leur reflet frontend ; celui de `has_more` (`:86-89`) ne dit plus « v0.1 … pas de UI dédiée » mais ce que
  fait l'écran depuis la 29-9. **Aucun code de production Rust ne change.**
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts:52-55` : le commentaire de `hasMore` (« v0.1 :
  structure présente mais pas de UI dédiée (bouton « Charger plus » v0.2) ») est réécrit comme celui du Rust — ce
  que fait l'écran depuis la 29-9 (avis, bouton jusqu'au plafond, avis plafonné) ; il ne parle plus de « Charger
  plus » (validation P1, R3/F3).
- **Commentaire de module `kesh-db` réécrit en T1** (décision de l'orchestrateur, validation P2, R1 = F1 ; C-29-9-16
  annoté) : `crates/kesh-db/src/repositories/reconciliation.rs:18-19` (« paginées (limit hardcodé 100 au caller, cf.
  L24) ») est **faux depuis la Story 8-4** (`22fd6d4a`, le commit même qui l'a écrit) — l'appelant ne code pas 100 en
  dur : la route prend `?limit=`, 100 par défaut, 500 au plus (`routes/reconciliation.rs:598-602`), et passe
  `limit + 1` au repository (`:622-626`) ; après la story, l'écran demande lui-même 500. Il devient : « paginées
  (`limit` fourni par la route : défaut 100, plafond 500, sentinelle `limit + 1` pour `hasMore` — cf.
  `DEFAULT_PROPOSALS_LIMIT` / `MAX_PROPOSALS_LIMIT`, `kesh-api/src/routes/reconciliation.rs`) ». Le renvoi `L24`
  (limitation « pas de UI » de la 8-4, que la 29-9 lève) disparaît. C'est un **commentaire** : aucun code de production
  Rust ne change, l'affirmation du point précédent tient.
  ⚠️ **Conséquence : `kesh-db` entre dans le diff**, et avec lui l'exception *kesh-db* de CLAUDE.md (« dès qu'un
  **patch** touche … un repository, le ciblage est interdit »). Elle vaut pour **le commit qui touche ce fichier** —
  celui de T1 — et pour lui seul : ce commit-là passe le gate complet ; les commits suivants, qui ne touchent pas
  `kesh-db`, reviennent au gate ciblé. Si une passe de revue retouche ce fichier, son patch reprend le gate complet
  (AC10). Un module de plus au sens de la règle de découpage (§ *Dérogation*).
- Tests neufs dans `crates/kesh-api/tests/reconciliation_e2e.rs`, à côté de `get_proposals_paginates_at_100_default`,
  même décor (`setup_company`, `make_new_tx`, `seed_bank_transactions`) :
  `get_proposals_limit_is_capped_at_500` — 501 transactions, `?limit=1000` → **500** rendues, `hasMore` **vrai** ; et
  `get_proposals_limit_500_exact_has_no_more` — **exactement 500** transactions, `?limit=500` → **500**, `hasMore`
  **faux** (le second état de l'écran, pris **à la frontière** : `>=` pour `>` y rougit, M18 ; validation P1, F8).
  ⚠️ La mutation `limit` pour `limit + 1` (M20) **ne** le rougit **pas** — 500 lignes lues sur 500, `500 > 500` est faux,
  `hasMore` faux comme attendu — : elle est tuée par le test **à 501** (`get_proposals_limit_is_capped_at_500` : 500
  lues, `hasMore` faux au lieu de vrai) et par `get_proposals_paginates_at_100_default` (validation P2, R2 = F3 ; la
  phrase de la P1 qui la lui attribuait était fausse).

### #655 — une relecture ancienne ne masque plus une page à jour

**AC5 — Détail d'un import : numéro de séquence (C-29-9-7).**

`routes/(app)/bank-import/[id]/+page.svelte`, `load()` (`:47-70`) : `let loadSeq = 0` ; `const mySeq = ++loadSeq` en
**première** instruction de `load()`, avant la remise à `null` et avant la branche « Identifiant invalide » ; après
l'`await`, `detail` et `errorMessage` ne s'écrivent que si `mySeq === loadSeq` (dans le `try` et le `catch`). La
remise à `null` de la 28-5b (`:51`) et son commentaire restent ; un commentaire dit le patron
(`ReconciliationProposals.svelte`, `loadGen`) et la raison (#655).

**`loading` n'est pas gardé (C-29-9-14).** `loading` ne passe à `true` qu'à la déclaration (`:43`) ; `load()` ne le
remet jamais à `true`, et une relecture ne peut partir qu'une fois la page rendue, donc après que la première lecture
l'a posé à `false` (les deux déclencheurs, `UnrejectTransactionAction` `:160` et le bouton qui ouvre
`CancelReconciliationDialog` `:150`/`:169-176`, ne sont rendus que dans la branche `{:else if detail}` `:96`, derrière
`{#if loading}` `:86`). Un `finally` périmé qui écrit `false` sur un `false` n'a aucun effet observable : une garde y serait
intuable (validation P1, F7 et R8). Le `finally` reste `loading = false`, avec une ligne de commentaire qui le dit —
et qui dit qu'il faudra le garder le jour où `load()` remettra `loading` à `true`.

**AC6 — Page d'un profil bancaire : remise à zéro et séquence (C-29-9-8).**

`routes/(app)/bank-import/profiles/[id]/+page.svelte`, `$effect` (`:13-31`). ⛔ **Contrainte première : la
correction ne démonte jamais le formulaire ni ne perd une saisie en cours au même identifiant** (C-29-9-13).

- **L'effet dépend de l'identifiant, et de lui seul** : `const rawId = $derived(page.params.id)` au niveau du script ;
  l'effet lit `rawId` (puis `Number(rawId)`), **jamais** `page.params`. Un `$derived` d'une **chaîne** compare par
  `===` (`equality.js:4-6`) : la réassignation de `page.params` à `id` égal le recalcule à la même valeur, et
  l'effet ne se rejoue pas. ⚠️ Dériver la chaîne, **pas** `Number(page.params.id)` : `NaN !== NaN`, si bien qu'un
  identifiant invalide relancerait l'effet à chaque réassignation de `page.params` (navigation où `page_changed` est
  vrai, `client.js:763-772`).
- **En tête de l'effet** — donc au début d'un chargement **réel**, puisqu'il ne se rejoue qu'à `id` changé :
  `const mySeq = ++loadSeq` en **première** instruction, **avant** la branche `!Number.isFinite(id)` (une réponse
  ancienne encore en vol ne doit pas écraser « ID invalide » ; validation P1, R7), puis `errorMessage = null` et
  `isLoading = true`.
- **`isLoading = true` est retenu, et c'est établi au code qu'il ne touche pas le formulaire au même id** : il n'est
  exécuté que dans l'effet, qui ne se rejoue qu'à `rawId` changé. À id changé, il est **nécessaire** :
  `BankProfileForm` initialise ses champs **une fois** depuis `existing` (`BankProfileForm.svelte:46-60`, `$state(existing?.…)`),
  si bien que sans démontage le profil 6 s'afficherait avec les champs du profil 5.
- **Garde de séquence** : dans le `.then` et le `.catch`, `profile`, `errorMessage` **et** `isLoading` ne s'écrivent
  que si `mySeq === loadSeq` — une réponse plus ancienne, succès comme échec, est ignorée en entier.
- ⚠️ Pas de boucle : l'effet lit `rawId` et le compteur (variable non réactive) ; il écrit `errorMessage`,
  `isLoading`, `profile`, qu'il ne lit pas.
- **Ce que l'effet exécute aussi : la partie synchrone de `getBankProfile(id)`** (validation P2, F6 ; C-29-9-18).
  « L'effet ne dépend que de l'identifiant » suppose qu'elle ne lit aucun signal. C'est vrai au sol : `request` →
  `buildHeaders` → `fetchWithRetry` → `fetchWithTimeout` ne lisent aucun `$state` avant le premier `await`
  (`bank-profile.api.ts:19-21`, `api-client.ts:181-199`, `:267-280`, `:329-336`, `:392-399`). Mais V13 mocke l'API et
  ne verrait pas une lecture synchrone d'`authState` qui y entrerait un jour (l'effet se relancerait au renouvellement
  du jeton, et `isLoading = true` perdrait la saisie). **Retenu : un commentaire au site** qui écrit cette dépendance
  et dit le remède (`untrack(() => getBankProfile(id))`) ; **pas** d'`untrack` dans le code — garde qu'aucun test ne
  peut rougir, même règle que C-29-9-14.
- **`onUpdated` reçoit la même garde** (décision de l'orchestrateur, validation P2, F2 ; C-29-9-17). Au sol
  (`:33-35`), `onUpdated(updated)` écrit `profile` sans condition. Scénario, possible dès qu'un lien mènera d'un profil
  à un autre : sur 5, *Mettre à jour* part (`BankProfileForm.svelte:113-114`, `existing.id` = 5 lu **avant** l'`await`) ;
  navigation vers 6, le formulaire de 5 est démonté mais `handleSubmit` continue ; 6 se charge, un formulaire neuf
  s'initialise sur p6 ; la sauvegarde de 5 se termine, `onSuccess(p5)` → `profile = p5` ; le formulaire monté reçoit
  `existing` = p5 mais garde les champs de 6 (initialisés une fois, `:46-60`) ; la sauvegarde suivante écrit les valeurs
  de 6 **dans le profil 5**. Correctif : une sauvegarde qui se termine après une navigation vers un autre profil
  n'écrit plus `profile`.
  ⚠️ **Forme de la garde, établie au code** : un numéro de séquence capturé dans une fermeture passée au formulaire
  (`onSuccess={onUpdatedFor(seq)}`) **ne tient pas**, parce que le formulaire lit ses props **en direct** : Svelte
  5.55.1 compile `let { existing, onSuccess } = $props()` suivi d'un appel après `await` en
  `$$props.onSuccess?.($$props.existing)` (compilation d'un composant d'essai, 2026-10-14, hors dépôt) — le formulaire
  démonté de 5 appellerait le gestionnaire **courant**, celui de 6, et la garde passerait. La garde se pose donc
  **dans** `onUpdated`, évaluée à l'appel, sur l'identité du chargement courant :
  `if (updated.id !== Number(rawId)) return;` puis `profile = updated`. C'est l'équivalent, pour une écriture que la
  page ne lance pas elle-même, du `mySeq === loadSeq` du chargement : est ignorée toute écriture qui ne relève plus du
  profil affiché. Retour de 6 à 5 avant la fin de la sauvegarde de 5 : la réponse est acceptée, et c'est juste (même
  profil, valeurs enregistrées). Si V17 montrait une autre lecture des props, le dire au Dev Agent Record avant de
  changer de forme.
- Commentaire au site : défaut latent au sol (aucun lien d'un profil à l'autre), corrigé pour le jour où il y en aura
  un ; pourquoi `$derived` de la chaîne (réassignation de `page.params`, `NaN`) ; pourquoi `isLoading = true` est
  sans effet à id égal et nécessaire à id changé ; la dépendance synchrone de `getBankProfile` (C-29-9-18) ; pourquoi
  la garde d'`onUpdated` porte sur l'identifiant et non sur une fermeture (C-29-9-17).

### #646 — une personne de contact ne s'efface pas sans confirmation

**AC7 — Confirmation avant la suppression (C-29-9-10).**

`ContactPersonsManager.svelte`, `removePerson` : la fonction reçoit la personne (ou son nom) ; avant tout appel,
`if (!window.confirm(i18nMsg('contact-persons-confirm-delete', '…', { name: \`${p.firstName} ${p.lastName}\` })))
return;` — refus → **aucun** appel à `deleteContactPerson`, la liste et le message d'erreur inchangés. Acceptation →
comportement actuel. Le bouton, sa classe et son `data-testid` ne changent pas (l'inventaire des boutons rouges le
suit, `boutons-rouges-inventory.test.ts:231`).

### Textes, tests, preuves

**AC8 — Textes : i18n quatre locales, manuel FR et son PDF, API, CHANGELOG, inventaire.**

- **Quatre clés neuves**, dans les quatre `crates/kesh-i18n/locales/*/messages.ftl`, à côté de leurs voisines
  (`reconciliation-labels-*` `fr-CH:1208-1216`, `reconciliation-actions-*` `:1252`, `contact-persons-*`
  `:1806-1815`) ; le fallback du `.svelte` = la valeur `fr-CH`, au caractère près (l'apostrophe droite de
  `jusqu'à`, celle du `.ftl`, impose un repli entre guillemets doubles ou échappé ; jamais un gabarit JS) :

  | Clé | fr-CH |
  |---|---|
  | `reconciliation-labels-more-hidden` | `Seules les { $count } transactions en attente les plus récentes sont affichées : il en reste de plus anciennes.` |
  | `reconciliation-actions-show-older` | `Afficher aussi les plus anciennes (jusqu'à { $max } en tout)` |
  | `reconciliation-labels-more-capped` | `Les { $max } transactions en attente les plus récentes sont affichées, le maximum que cette page montre à la fois. Il en reste de plus anciennes : elles apparaîtront à mesure que celles-ci seront acceptées, affectées manuellement ou éclatées.` |
  | `contact-persons-confirm-delete` | `Supprimer la personne de contact { $name } ? Aucun écran ne permet de la rétablir.` |

  de-CH, en-CH, it-CH : traductions fidèles, même vouvoiement / registre que les clés voisines du même fichier.
- **Une clé réécrite**, `reconciliation-unreject-success` ×4 : « …dont la liste montre **d'abord** les 100 plus
  récentes en attente. » (de : « zuerst », en : « first », it : « prima ») — le fallback de
  `UnrejectTransactionAction.svelte:49` suit ; `UnrejectTransactionAction.test.ts:147` lit la valeur `fr-CH` et suit
  seul.
- `frontend/src/lib/shared/i18n-keys.test.ts` : `sitesTotal` (`:634`, 2107 au sol) se **recompte à la source**
  après le dernier rebase (la 29-3, la 29-6 et la 29-8 en ajoutent), avec une ligne de commentaire datée qui donne les
  deux bornes et la commande, sur le modèle des lignes `:620-633`.
- **`frontend/scripts/lint-i18n-ownership.js`** : la clé `contact-persons-confirm-delete` entre dans
  `KNOWN_VIOLATIONS` (ensemble `:22-140` ; bloc `ContactPersonsManager.svelte` `:117-139`, commentaire `:117-124`,
  entrées `:125-139`), **après la ligne `:139`**, à la suite des autres clés de ce composant (validation P2, R5 = F9) :
  `'src/lib/features/contacts/ContactPersonsManager.svelte:contact-persons-confirm-delete',` — le lint compare le
  préfixe `contact` au dossier `contacts` (dette #30), et le rougirait sinon. La dette #30 s'accroît d'**une** entrée,
  écrite comme telle au Dev Agent Record (validation P1, R2/F2). Les trois clés `reconciliation-*` sont dans le
  dossier `reconciliation` : aucune entrée. `npm run lint-i18n-ownership` vert.
- **Gardes i18n à relancer et qui portent sur ces clés** (validation P1, R6) :
  `frontend/src/lib/shared/i18n-entrees-a-variables.test.ts` (variables `{ $count }`, `{ $max }`, `{ $name }`
  identiques entre site, repli et **quatre** locales ; repli littéral) ; `i18n-repli-divergent-actif.test.ts` (repli =
  valeur `fr-CH`, la clé figurant au catalogue) ; `i18n-keys.test.ts` ; `i18n-harvest.test.ts`. Toutes au
  `test:unit` ; leur vert se cite au Dev Agent Record. Hors de portée, relevé au sol : `i18n-un-repli-par-cle.test.ts`
  ne vise que le domaine `supplier-invoices` (en-tête du fichier) ; `crates/kesh-api/tests/textes_coherents.rs`,
  parcouru par la lentille R, n'a aucune garde sur ces phrases.
- **Manuel** `docs/manual/fr/user-manual.tex` :
  - `:2045` — « les 100 plus récentes au plus » devient : les 100 plus récentes d'abord ; s'il en reste, un avis le dit
    sous la liste et le bouton *Afficher aussi les plus anciennes* l'étend à 500 ; au-delà, les plus anciennes
    apparaissent à mesure que les premières sont acceptées, affectées ou éclatées ; le bouton vide la sélection ;
  - `:2057` — **en delta sur la phrase rebasée** : le paragraphe *Rejeter* est réécrit par la 29-6 (et relu par la
    29-8) ; la 29-9 ne le remplace pas. Dans la phrase telle que `main` la porte au rebase, elle ne touche **que** la
    parenthèse « qui ne montre que les 100 plus récentes en attente~: une transaction plus ancienne n'y reparaît
    qu'une fois les plus récentes traitées » : le chiffre en sort, et la parenthèse renvoie à l'avis et au bouton
    décrits plus haut (`:2045`). Tout le reste de la phrase — ce que la 29-6 et la 29-8 y ont écrit — reste à
    l'octet ; le diff de ce paragraphe ne porte que sur la parenthèse (validation P1, R4) ;
  - `:869-874` — la `keshnote` des personnes de contact ajoute : *Supprimer* demande confirmation ; la personne quitte
    la fiche, et aucun écran ne la rétablit.
  - PDF régénéré (`make fr` dans `docs/manual/`, **sous le verrou**), contrôlé aplati (§ *Propagation*).
- **`docs/api-external.md:594`** : « (100 par défaut) » → « (100 par défaut, 500 au plus ; `hasMore` vaut `true` s'il
  en reste) » ; le lien #649 est retiré (l'issue sera close) ; le reste de la phrase, vrai pour l'API, ne change pas.
  **En delta** sur le texte rebasé : la 29-8 ajoute ses deux routes en paragraphes à côté de `…/unreject` (`:594`),
  qui restent à l'octet (zones partagées ; validation P2, R6).
- **CHANGELOG** : dans la section de la v0.13.4 que porte `main` au moment du rebase (la 29-2 la crée — sinon
  `## [Non publié]`), sous `### Corrigé`, trois puces : la suite de la liste de réconciliation (#649), la bannière
  périmée du détail d'un import (#655), la confirmation de suppression d'une personne de contact (#646). La section
  `[0.13.3]` ne se touche pas.
- **`boutons-rouges-inventory.test.ts:233`** : motif → `'D — suppression d\'une personne de contact, confirmée (#646) ;
  texte rouge sans bordure (#648)'` (le motif dirait faux sinon ; l'entrée et ses jetons restent).

**AC9 — Tests Vitest rouges d'abord, E2E, captures.**

Vitest (écrits **avant** le correctif et vus rouges, sortie copiée au Dev Agent Record) :

- `ReconciliationProposals.test.ts` (patrons du fichier : `mockApi.getProposals`, `satisfies GetProposalsResponse`) :
  - ⛔ **Étape 0, avant les tests neufs (V1-V5, V12, V18) — le mock du module API** (validation P1, R1/F1). La fabrique
    `vi.mock('./reconciliation.api', () => ({ getProposals: vi.fn(), … }))` (`:22-27`) n'exporte pas les constantes
    neuves : dès que le composant importe `PROPOSALS_DEFAULT_LIMIT`/`PROPOSALS_MAX_LIMIT`, Vitest lève à l'accès
    (« No "…" export is defined on the mock ») et **tout** le fichier rougit, sans rapport avec son objet. **Geste en
    delta, sur la fabrique telle que `main` la porte au rebase** (validation P2, R6) : elle passe à
    `vi.mock('./reconciliation.api', async (importOriginal) => ({ ...(await importOriginal<typeof
    import('./reconciliation.api')>()), /* les vi.fn() de la fabrique rebasée, tous gardés */ }))` — chaque entrée que
    la fabrique porte au rebase reste un `vi.fn()` (au sol, quatre : `getProposals`, `acceptProposals`,
    `rejectProposals`, `manualMatchTransaction` ; la 29-8 en ajoute, `getLinkCandidates`/`linkTransaction`, fiche 29-8
    `:387`, `:550`), les constantes sont les vraies. Ne **jamais** recopier une fabrique complète de cette fiche : ce
    serait effacer les entrées d'une fiche voisine.
    Ce changement se fait **en premier**, avec les constantes de l'AC4 (T1), et le fichier existant se relance **vert**
    avant d'écrire les tests neufs : ainsi V2-V5 rougissent pour la **bonne** raison (le comportement absent), et la
    sortie copiée au Dev Agent Record le montre ; V1 et V12, qui exigent une **absence**, sont verts au sol (T2 ;
    validation P2, F7). Seul ce fichier mocke `./reconciliation.api` pour un composant qui importera les constantes
    (`git grep -ln "reconciliation.api'" -- frontend/src`, relu par les deux lentilles).
    **Dépendance neuve, à savoir d'avance** (validation P2, R4) : au sol, ce fichier de test ne charge **pas**
    `api-client` — les deux modales sont des doublures (`:44-49`), `error-message.ts` n'importe que `./api-error`, les
    API des comptes sont mockées. `importOriginal` de `./reconciliation.api` fait entrer `api-client.ts` et ses imports :
    `$lib/app/stores/auth.svelte` (qui crée un `BroadcastChannel` dès son chargement, `auth.svelte.ts:34-36`, toléré
    absent sous jsdom), `api-health.svelte`, et `i18n.svelte` (mocké dans ce fichier, `:53`). Risque faible —
    `bank-import-page.test.ts:19-20` charge déjà le vrai `api-client` par `importOriginal` —, mais un échec au
    chargement du fichier se lit d'abord là.
    Si `importOriginal` se révèle impossible (import réel qui casse le montage), repli : loger les deux constantes dans
    un module sans I/O `reconciliation.limits.ts`, non mocké, et **`ReconciliationProposals.svelte` importe les
    constantes directement depuis `./reconciliation.limits`** — une réexportation par `reconciliation.api.ts` reste
    possible pour les autres lecteurs, mais le composant qui passerait par elle traverserait le module mocké, et l'échec
    « No "…" export » reviendrait (validation P2, R4). Repli à déclarer au Dev Agent Record.
  - V1 `hasMore: false` → ni `reconciliation-more`, ni bouton, ni `reconciliation-more-capped` ;
  - V2 `hasMore: true` (100 lignes) → avis avec « 100 », bouton présent ; `getProposals` appelé avec `(id, 100)` ;
  - V3 clic → liste de la seconde réponse rendue, **puis** `await tick()`, **puis seulement** l'assertion :
    `getProposals` appelé **exactement deux fois** en tout, la seconde avec `(id, 500)` (une troisième requête — M3 —
    part au tick suivant : compter avant elle la laisserait survivre ; validation P1, F10) ;
  - V4 seconde réponse `hasMore: true` → `reconciliation-more-capped` présent, **aucun** bouton ; le texte ne
    contient pas « rejet » ;
  - V5 après le clic, un *Accepter* réussi recharge avec `(id, 500)` (pas 100) ;
  - V12 `hasMore` **retombe** : premier chargement `hasMore: true`, *Accepter* réussi, relecture `hasMore: false` → ni
    `reconciliation-more`, ni bouton, ni `reconciliation-more-capped` (validation P1, F6) ;
  - V18 **bouton désactivé pendant un lot** : premier chargement `hasMore: true`, une ligne cochée, `acceptProposals`
    **différé** (promesse tenue par le test), clic *Accepter* → `reconciliation-show-older-btn` porte `disabled` et
    `getProposals` n'a été appelé qu'une fois ; puis la promesse se résout (validation P2, R8) ;
  - aucun test existant ne vérifie les arguments de `getProposals` (`grep -n "toHaveBeenCalledWith"
    ReconciliationProposals.test.ts` au sol : aucun sur `getProposals`) ; `reconciliation.api.test.ts` vérifie l'URL
    (`limit=100` par défaut) et reste vert.
- `bank-import-detail-page.test.ts` :
  - V6 deux relectures (deux *Annuler le rejet*, patron du test `:171-217`) ; la **plus récente** réussit d'abord,
    la plus ancienne **échoue ensuite** → aucune bannière, lignes de la plus récente ;
  - V7 même décor, la plus ancienne **réussit** ensuite avec des données périmées → les lignes restent celles de la
    plus récente ;
  - le test existant `:171` reste vert (ordre direct) ;
  - pas de test sur `loading` : non gardé, et sans effet observable (C-29-9-14).
- `profiles/[id]/profile-page.test.ts` : un `page` **réactif qui imite SvelteKit** (validation P1, F4) — fixture
  `profile-page-state.svelte.ts` à côté du test : `class FakePage { params = $state.raw<Record<string, string>>({ id:
  '5' }); } export const fakePage = new FakePage();`, servie par `vi.mock('$app/state', async () => ({ page: (await
  import('./profile-page-state.svelte')).fakePage }))`. Une navigation **réassigne** l'objet entier
  (`fakePage.params = { id: '6' }`), comme `client.js:1888` ; jamais `fakePage.params.id = …`. `beforeEach` remet
  `fakePage.params = { id: '5' }` (module partagé : sans cela l'ordre des tests décide ; F10). `await tick()` après
  chaque réassignation.
  - V8 id 5 en échec → message ; id 6 en succès → message disparu, formulaire rendu ;
  - V9 id 5 lent puis id 6 rapide ; 6 réussit, 5 échoue ensuite → aucun message, profil 6 ;
  - V13 **navigation au même id** : id 5 en succès, une saisie dans le champ `bank-name-input` (`BankProfileForm.svelte:135-137`),
    puis `fakePage.params = { id: '5' }` (objet neuf) et `await tick()` → `getBankProfile` appelé **une seule fois**,
    jamais « Chargement… », la saisie **intacte** ;
  - V14 **montage neuf à id changé** : id 5 en succès (`bankName` « Banque 5 »), puis id 6 (« Banque 6 ») → pendant
    la lecture de 6, « Chargement… » et aucun formulaire ; ensuite le champ montre « Banque 6 » (validation P1, F5) ;
  - V15 **5 se termine pendant que 6 est en vol** : id 5 lent, id 6 plus lent ; 5 **réussit** pendant que 6 court →
    « Chargement… » toujours affiché, aucun champ « Banque 5 » ; puis 6 réussit → « Banque 6 » (validation P1, R8) ;
  - V16 **identifiant invalide après une lecture lente** : id 5 lent, puis `{ id: 'abc' }` → « ID invalide » ; 5
    échoue ensuite → « ID invalide » reste (validation P1, R7) ;
  - V17 **sauvegarde tardive après navigation** (validation P2, F2) : la fabrique de
    `$lib/features/bank-import/bank-profile.api` gagne `updateBankProfile: (id, req) => updateMock(id, req)` (et
    `createBankProfile: vi.fn()`, jamais appelé, pour que le module mocké reste complet — le formulaire importe les deux,
    `BankProfileForm.svelte:11-14`) ; id 5 en succès (« Banque 5 », fabrique de profil **complète** : le formulaire lit
    `existing.columnMapping`, `:34-62`), `updateMock` **différé**, envoi du formulaire (`fireEvent.submit` sur
    `bank-import-profile-form`) ; `fakePage.params = { id: '6' }`, 6 en succès (« Banque 6 ») ; **puis** la sauvegarde
    de 5 se résout avec p5 → le champ montre « Banque 6 », et un second envoi appelle `updateMock` avec **6** en premier
    argument (sous M19 : 5) ;
  - les deux tests existants restent verts. Si la fixture réactive se révèle impossible sous Vitest, le dire au Dev
    Agent Record avec l'erreur, et **ne pas** déclarer AC6 prouvé.
- `ContactPersonsManager.test.ts` (neuf ; `vi.stubGlobal('confirm', …)`, patron `BankProfileList.test.ts:57-58`) :
  - V10 refus → `deleteContactPerson` **non** appelé, la personne reste listée ; `confirm` reçu avec le nom ;
  - V11 acceptation → `deleteContactPerson(id)` appelé, liste relue.

E2E (Playwright, montage `docs/testing.md` § *Prérequis Playwright local*, port **3049**, base `kesh_e2e_299`) :

- `frontend/tests/e2e/reconciliation-show-older.spec.ts` (neuf, patron `reconciliation-unreject.spec.ts:79-121`) :
  import par l'API d'un CAMT **synthétique** de **501** transactions (le bloc `<Ntry>` de
  `camt053_v04_minimal.xml` répété), `confirmBalanceMismatch` (la somme de 501 lignes ne retombe pas sur `CLBD`) ;
  page *Réconciliation*, compte sélectionné → 100 lignes, avis et bouton ; clic → 500 lignes, avis plafonné, aucun
  bouton ; la 150e de nos transactions (par ordre décroissant) absente avant, présente après ; import supprimé en
  `finally`. Captures clair et sombre des deux états (`capturePath` de `reconciliation-unreject.spec.ts:125`).
  Contraintes de la fixture, établies par la lentille F de la validation P1 (F11) :
  - **chaque entrée unique** — montant, `AcctSvcrRef` et `EndToEndId` distincts par entrée (et par run) : le
    dédoublonnage par défaut de l'import est un *skip* silencieux (`ConfirmDuplicateLines::Skip`) sur (date, montant,
    référence normalisée, compte) ; une entrée en double disparaîtrait sans erreur et le compte de 501 serait faux. La
    spec vérifie, à la réponse de l'import, que **501** lignes sont créées ;
  - `FrToDt` cohérent avec les dates des entrées ;
  - **pas de contrainte de date** vis-à-vis des autres fixtures : la contrainte « dates antérieures à celles de
    toutes les autres fixtures E2E » de la rédaction initiale est **retirée** — `seedTestState('with-company')` en
    `beforeAll` remet la base à zéro et `workers: 1` (`playwright.config.ts:36-53`) : la spec ne voit que ses propres
    transactions, et l'ancienne hypothèse « moins de 350 autres en attente » devient sans objet.
- `frontend/tests/e2e/contact-persons-delete.spec.ts` (neuf) : un contact *Entreprise* et une personne créés par
  l'API (noms uniques), dialogue d'édition ouvert ; `page.once('dialog', d => d.dismiss())` → la personne reste ;
  `page.once('dialog', d => d.accept())` → elle part, et `GET /api/v1/contacts/{id}/persons` (route `lib.rs:792-795`) ne la rend plus. Le message du dialogue contient le nom.
- **Aucune capture** de la confirmation (dialogue natif, invisible des captures Playwright ; C-29-9-10) — à déclarer.
  Pages #655 : rendu inchangé, aucune capture.
- **Suite E2E complète** au dernier commit de code, jugée fichier par fichier contre `docs/testing.md` § *Les échecs
  attendus*.

**AC10 — Mutations et gates.**

Mutations (motif vérifié unique avant écriture ; restauration `git checkout -- f && touch f`) :

| # | Mutation | Doit tuer |
|---|---|---|
| M1 | `hasMore = r.hasMore` retiré (reste `false`) | V2 |
| M2 | le bouton pose `limit = PROPOSALS_DEFAULT_LIMIT` | V3 |
| M3 | le bouton pose `limit` **et** appelle `load()` | V3 (deux appels → trois) |
| M4 | condition du bouton sans `limit < PROPOSALS_MAX_LIMIT` | V4 |
| M5 | `load()` appelle `getProposals(bankAccountId)` (sans `limit`) | V3, V5 |
| M6 | `MAX_PROPOSALS_LIMIT = 499` (Rust) | `get_proposals_limit_is_capped_at_500`, `get_proposals_limit_500_exact_has_no_more` |
| M7 | garde `mySeq !== loadSeq` retirée du `catch` du détail | V6 |
| M8 | garde retirée du `try` du détail | V7 |
| M9 | `errorMessage = null` retiré de l'effet du profil | V8 |
| M10 | garde de séquence retirée du `.catch` du profil | V9 |
| M11 | `if (!confirm(…)) return;` → `confirm(…);` | V10 |
| M12 | `return` placé **avant** l'appel quelle que soit la réponse | V11 |
| M13 | `isLoading = true` retiré de l'effet du profil | V14 |
| M14 | `hasMore = r.hasMore` → `if (r.hasMore) hasMore = true` | V12 |
| M15 | l'effet du profil relit `page.params.id` au lieu de `rawId` (`$derived` contourné) | V13 |
| M16 | profil : `isLoading = false` écrit **hors** de la garde de séquence, dans le `.then` et le `.catch` | V15 |
| M17 | profil : `const mySeq = ++loadSeq` placé **après** la branche `!Number.isFinite(id)` | V16 |
| M18 | `has_more = transactions.len() as i64 >= limit` (Rust, `routes/reconciliation.rs:628`) | `get_proposals_limit_500_exact_has_no_more` |
| M19 | profil : garde `updated.id !== Number(rawId)` retirée d'`onUpdated` | V17 |
| M20 | `limit + 1` → `limit` dans l'appel du repository (Rust, `routes/reconciliation.rs:625`) | `get_proposals_limit_is_capped_at_500`, `get_proposals_paginates_at_100_default` |
| M21 | `disabled={busy}` retiré du bouton `reconciliation-show-older-btn` | V18 |

M15 n'est tuée que parce que la fixture **réassigne** `params` (comme SvelteKit) : avec un `$state` profond dont seul
`id` bougerait, l'effet ne se rejouerait pas à id égal, mutation ou non — c'est ce que F4 a relevé. M18 et M6
sont des mutations temporaires de production Rust, restaurées par `git checkout -- f && touch f` comme les autres ; M20
aussi.

Gates : gate ciblé entre les passes ; **exception *kesh-db*** : le commit de T1, qui réécrit le commentaire de
`crates/kesh-db/src/repositories/reconciliation.rs:18-19`, passe le **gate complet** (`scripts/test-fast.sh` sur
`kesh_299` remise à zéro) — à ce commit-là seulement, et à tout patch de revue qui retoucherait ce fichier ; les autres
commits restent au gate ciblé (validation P2, R1 = F1). **Au dernier commit de code** : `scripts/test-fast.sh` complet sur base
`kesh_299` remise à zéro (migrations du worktree, seed), frontend complet (`check`, `lint-i18n-ownership`,
`test:unit`, `build`), E2E complet — sous le verrou `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`. Ne déclarer
que ce qui a tourné, avec ses chiffres.

## Tasks / Subtasks

- [ ] **T0** — Montage : `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-29-9/target` ; `node_modules` par `cp -al` depuis le
  dépôt principal ; bases `kesh_299` et `kesh_e2e_299` (migrations du worktree, `scripts/seed-dev-db.sql`) ; port
  3049. **Rebaser d'abord** sur `main` si la 29-3, la 29-6 ou la 29-8 y sont mergées, puis relire chaque numéro de
  ligne de cette fiche.
- [ ] **T1** — #649 contrat (AC4) : constantes frontend **et**, dans le même geste, le mock de
  `ReconciliationProposals.test.ts` passé à `importOriginal` (AC9, étape 0, en delta ; fichier relancé vert) ;
  commentaire de `reconciliation.types.ts:52-55` ; commentaire de module de `kesh-db` `:18-19` réécrit (AC4) ;
  doc-comments Rust, deux tests Rust ; M6, M18, M20 ; **gate complet à ce commit** (exception *kesh-db*, AC10) ; commit
  `test(29-9): … (refs #649)`.
- [ ] **T2** — #649 écran (AC1-AC3) : V2-V5 et V18 rouges, puis le correctif ; V1 et V12 exigent une **absence** et
  sont donc verts au sol par construction — ils se déclarent tels, et V12 vaut par M14 ; M1-M5, M14, M21 ; commit
  `fix(reconciliation): … (refs #649)`.
- [ ] **T3** — #655 (AC5, AC6) : V6-V9 et V13-V17 rouges (V13 l'est au sol : l'effet actuel relit à id égal), puis les
  deux pages ; M7-M10, M13, M15-M17, M19 ; commit `fix(bank-import): … (refs #655)`.
- [ ] **T4** — #646 (AC7) : V10-V11 rouges, puis le correctif ; M11-M12 ; commit `fix(contacts): … (refs #646)`.
- [ ] **T5** — i18n (AC8) : quatre clés, une réécrite, ×4 ; `sitesTotal` recompté ; `KNOWN_VIOLATIONS` de
  `lint-i18n-ownership.js` (+1) ; inventaire des boutons rouges ; gardes i18n nommées à l'AC8 relancées.
- [ ] **T6** — Manuel FR, `make fr` sous le verrou, contrôle aplati ; `api-external.md` ; CHANGELOG ; commit
  `docs(29-9): … (refs #649, refs #646)`.
- [ ] **T7** — E2E (AC9) : deux specs, captures clair et sombre (hors dépôt, chemins au Dev Agent Record) ; commit
  `test(e2e): … (refs #649, refs #646)`.
- [ ] **T8** — Propagation (§ ci-dessous), puis gates complets au dernier commit de code (AC10).
- [ ] **T9** — Dev Agent Record, File List, Change Log ; statut `review` ; registre de sprint. Rapport : demander le
  push, la PR (`closes #649 #655 #646` au corps seulement), le commentaire de #655 (C-29-9-8), et la décision sur une
  issue de pagination au-delà de 500 (C-29-9-3).

## Dev Notes

### Dérogation à la règle de découpage

Modules touchés au sens de la règle : (1) le frontend — trois zones, `features/reconciliation`, `routes/(app)/
bank-import`, `features/contacts` ; (2) `kesh-api` — deux doc-comments et deux tests, **aucun code de production** ;
(3) les catalogues `kesh-i18n` ; (4) la documentation (manuel, PDF, `api-external.md`, CHANGELOG). **Quatre**, sous
la limite de cinq : le critère « scope cross-cutting » n'est pas atteint. Si l'on compte chaque zone du frontend
comme un module, on en compte six : la section tient alors lieu de la `Dérogation règle de splitting`, **déclarée au
Project Lead par le rapport de l'orchestrateur**. *Annotation du 2026-10-14 (validation P2, R1 = F1)* : la réécriture
du commentaire de module de `kesh-db` (AC4) ajoute un module, (5) `kesh-db` — **un commentaire**, aucun code. Cinq dans
la première lecture, ce qui reste sous le seuil (« plus de 5 ») ; sept dans la seconde, couverte par la même
dérogation déclarée. Dans les deux lectures, pas de découpage : les trois correctifs sont
indépendants, chacun tient en un ou deux fichiers de production, aucun ne change une règle métier, et la revue se
fait correctif par correctif (précédent : 28-8, C-28-8-1). Un correctif fautif **sort du périmètre** plutôt que de
retenir les deux autres. C-29-9-1.

### Zones partagées avec les fiches voisines (non mergées)

| Fiche | Ce qu'elle touche de commun | Ce que fait la 29-9 |
|---|---|---|
| 29-3 (#527) | `ReconciliationProposals.svelte` : en-tête `:8`, paragraphe de priorité **avant** la table ; `i18n-keys.test.ts` `sitesTotal` +1 ; manuel `:2025-2060`, dont `:2045-2050`, `:2049`, `:2056` | ajout **après** `</table>` ; `sitesTotal` recompté ; phrase `:2045` réécrite **sur la version 29-3** |
| 29-6 (#665) | même composant : case sur toutes les lignes, *Accepter* grisé, indication dans la rangée des boutons, garde d'`onAccept` ; s'appuie sur « `load()` vide `selected` » (état (a)) ; manuel `:2050`, `:2056`, **`:2057`** (paragraphe *Rejeter* réécrit : « écarte la transaction, sans écriture »), `:2901`, `:2910-2911` | ne touche ni la case, ni la rangée, ni `onAccept` ; **garde** la remise à vide de `selected` (C-29-9-5) ; à `:2057`, **delta** sur la parenthèse seule (AC8) |
| 29-8 (#670) | même composant : troisième bouton de ligne, modale `LinkSettlementModal` (fiche 29-8 `:370`, emplacement non prescrit — naturellement après `</table>`, à côté des deux autres) ; manuel `:2021`, **`:2057`** (y garde « une transaction rejetée compte comme non rapprochée »), `:2043-2060` | ne touche pas la ligne de table ; avis et bouton **immédiatement** après `</table>`, avant toute modale, la sienne comprise (AC1 ; validation P2, F8) ; à `:2057`, **delta** sur la parenthèse seule (AC8) |
| 29-3, 29-6, 29-8 — `docs/api-external.md` | 29-3 `:549` et 29-6 `:551` (§ *Accepter des propositions*) ; 29-8 : ses deux routes en paragraphes « à côté de `…/unreject`, `:594` » (fiche 29-8 `:474`) | `:594`, **en delta** sur la phrase rebasée : la parenthèse « (100 par défaut) » seule, les paragraphes de la 29-8 gardés (validation P2, R6) |
| 29-8 — `reconciliation.api.ts`, `reconciliation.types.ts` | `getLinkCandidates`, `linkTransaction` et leurs types (fiche 29-8 `:387-388`) | deux constantes et le défaut de `getProposals` ; commentaire de `hasMore` (`.types.ts:52-55`) — ajouts **à côté**, rien de la 29-8 retiré (R6) |
| 29-3, 29-6, 29-8 — `ReconciliationProposals.test.ts` | fabriques et tests neufs (29-3 `:532`, 29-8 `:550`) ; la 29-6 réécrit le test `:179` (fiche 29-6 `:636`) ; la 29-8 peut ajouter des entrées au mock du module API | mock passé à `importOriginal` **en delta** sur la fabrique rebasée, toutes ses entrées gardées (AC9, étape 0 ; R6) ; tests neufs à la suite |

**Recensement au sol** (`grep -noE "user-manual\.tex:?[0-9]{3,4}(-[0-9]+)?|:2057|:2045|:2050|:2901|:2910"` sur les
trois fiches, 2026-10-14, validation P1 R4) : `:2057` est édité par **trois** fiches (29-6, 29-8, 29-9), `:2045` par
deux (29-3, 29-9) ; `:869-874` par la 29-9 seule. L'emplacement « après `</table>` » n'est prescrit par aucune des trois (le rappel `reconciliation-rejected-hint`
que garde la 29-6 est plus bas, `.svelte:455`, hors de la branche liste non vide).
*Précision du 2026-10-14 (validation P2, F8)* : « prescrit par aucune » reste vrai à la lettre, mais la 29-8 instancie
une modale sans dire où, et son emplacement naturel est ce même point ; d'où la règle de l'AC1 — l'avis et le bouton
d'abord, les modales ensuite, quelle que soit la fiche qui les a posées.

Les numéros de ligne des fiches voisines cités dans ce tableau sont ceux du 2026-10-14 (validation P2) : ces fiches
évoluent, et T0 les relit.

**Ordre** : la 29-9 se rebase et se merge **après** les trois (C-29-9-11). Au rebase : jamais `--ours`/`--theirs` sur
les PDF, régénérer ; recompter `sitesTotal` à la source.

### Ce qui doit être préservé

- Le plafond serveur de 500 (M9 de la 8-4) et le défaut de 100 : aucun code de production Rust ne change.
- `loadGen` de `ReconciliationProposals.svelte` et la remise à vide de `selected` dans `load()`.
- La remise à `null` de la 28-5b dans le détail d'un import et le test `bank-import-detail-page.test.ts:171`.
- `{#key selectedId}` de la page de réconciliation.
- Le bouton de suppression d'une personne : classe, texte, `data-testid` (l'inventaire des boutons rouges et #648 le
  suivent).
- La section `[0.13.3]` du CHANGELOG (publiée).

### Exclusions écrites (périmètre minimal)

- **Pas de pagination serveur** au-delà de 500 (C-29-9-2, C-29-9-3) ; l'orchestrateur décide s'il ouvre une issue.
- **Pas de filtre par période** (proposé par #649 parmi d'autres moyens ; le bouton suffit).
- **#655** : seuls les deux sites de l'issue ; la classe n'est pas revendiquée fermée (C-29-9-9). Le détail d'import
  charge à l'`onMount` et ne relit pas sur changement de `page.params.id` — non atteignable aujourd'hui (aucun lien d'un
  import à un autre), **non traité**.
- **#646** : pas de désarchivage d'une personne, pas de dialogue maison ; l'aspect du bouton relève de #648.
- **Manuels DE/IT/EN** : vides (`docs/manual/{de,en,it}/`) ; rien à traduire.

### Propagation — le symptôme à greper après chaque patch

```sh
git grep -nE "\b100\b" -- docs/manual/fr/user-manual.tex | sed -n '/20[0-9][0-9]:/p'      # #649, zone réconciliation
git grep -nE "100 plus récentes|100 most recent|100 neuesten|100 più recenti" -- crates docs frontend/src
git grep -nE "pas de UI dédiée|Charger plus|L24\b" -- crates frontend/src docs              # #649, commentaires (Rust ET frontend ; R3/F3)
git grep -n "hasMore" -- frontend/src ':!*.test.ts'                                     # #649 : types + composant
git grep -n "issues/649" -- docs README.md website                                      # lien retiré d'api-external
git grep -n "errorMessage" -- "frontend/src/routes/(app)/bank-import"                    # #655
git grep -n "sans confirmation" -- frontend/src docs                                    # #646
git grep -n "contact-persons-confirm-delete" -- frontend crates/kesh-i18n                # #646 : site, repli, 4 locales, KNOWN_VIOLATIONS
git grep -n "page.params" -- "frontend/src/routes/(app)/bank-import/profiles"           # #655 : l'effet ne relit plus page.params (rawId seul)
git grep -ln "reconciliation.api'" -- frontend/src                                     # mocks du module API (R1/F1)
git grep -n "onUpdated\|onSuccess=" -- "frontend/src/routes/(app)/bank-import/profiles" # #655 : garde d'onUpdated (P2, F2)
git grep -n "hardcodé 100" -- crates                                                    # kesh-db réécrit (P2, R1 = F1) : vide
pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "plus récentes.\{0,200\}"
pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "personnes de contact.\{0,300\}"
```

Les occurrences légitimes se trient à la main et se citent au Dev Agent Record. Liste attendue **après** la story
(validation P2, R9) :

- le CHANGELOG `[0.13.3]` (publié) ;
- la valeur 100 des constantes (`PROPOSALS_DEFAULT_LIMIT`, `DEFAULT_PROPOSALS_LIMIT`) et des doc-comments qui les
  nomment, dont le commentaire de module de `kesh-db` réécrit (« défaut 100, plafond 500 ») ;
- la clé réécrite `reconciliation-unreject-success`, dans ses **quatre** traductions (« montre **d'abord** les 100 plus
  récentes… », « first », « zuerst », « prima »), et son repli `UnrejectTransactionAction.svelte:49-50` ;
- la phrase neuve du manuel à `:2045` (« les 100 plus récentes d'abord ») et son reflet dans le PDF.

Ce qui ne doit **plus** sortir : `L24` et « hardcodé 100 » (`kesh-db` réécrit, C-29-9-16), « pas de UI dédiée » et
« Charger plus » (doc-comment Rust et `reconciliation.types.ts` réécrits), « 100 plus récentes au plus » et « qui ne
montre que les 100 plus récentes ».

### Pièges connus du dépôt, applicables ici

- **Effet qui suit `limit`** (AC2) : poser `limit` puis appeler `load()` double la requête ; V3 l'attrape.
- **`page.params` est réassigné à chaque navigation où `page_changed` est vrai** — URL, erreur, formulaire ou données
  changés, `client.js:763-772` (AC6) : un effet qui lit `page.params.id` se rejoue à id égal ;
  dériver la **chaîne** (`$derived(page.params.id)`), pas le nombre (`NaN !== NaN`). V13 l'attrape.
- **Mock par fabrique sans `importOriginal`** : un export neuf du module mocké est absent du mock et fait lever
  Vitest à l'accès ; `ReconciliationProposals.test.ts` (AC9, étape 0) — et `profile-page.test.ts`, dont la fabrique
  de `bank-profile.api` doit gagner `updateBankProfile` (et `createBankProfile`) pour V17.
- **Un composant Svelte 5 lit ses props en direct** (`$$props.x` à chaque accès, même démonté) : un numéro de séquence
  capturé dans une fermeture passée en prop ne garde rien (AC6, C-29-9-17).
- **Fallbacks Svelte = valeur `fr-CH`**, au caractère près (apostrophe typographique ou droite : copier celle du
  `.ftl`).
- **Marques d'isolation Fluent** : `i18nMsg` les consomme avec la variable (`i18n.svelte.ts:22-27`) ; ne pas les
  re-coder dans un test.
- **jsdom n'a pas `window.confirm`** : `vi.stubGlobal` (patron `BankProfileList.test.ts:57`).
- **Base remise à zéro avant chaque gate complet** (KF-039), sur `kesh_299` seulement ; jamais `docker restart` du
  MariaDB partagé.
- **Gates lourds et `make` des manuels sous le verrou** ; tuer le backend E2E **par PID**.
- **`pdftotext` coupe les lignes** : aplatir avant de greper.
- **Mutation restaurée, binaire périmé** : `git checkout -- f && touch f`.
- **Édition par l'outil Edit uniquement** ; aucun `sed -i`/`perl -pi` sur du code.
- **Données réelles** : fixtures CAMT synthétiques seulement ; aucune donnée du relevé de Guy.

### References

- [#649], [#655], [#646] (`gh issue view N`, lues le 2026-10-14).
- `crates/kesh-api/src/routes/reconciliation.rs:56-90`, `:578-631` ; `crates/kesh-db/src/repositories/reconciliation.rs:18-19`
  (commentaire de module réécrit), `:276-311` ;
  `crates/kesh-api/tests/reconciliation_e2e.rs:2065-2116`.
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `reconciliation.api.ts:16-26`,
  `reconciliation.types.ts:52-55`, `ReconciliationProposals.test.ts:22-27` (mock) ; `frontend/scripts/lint-i18n-ownership.js:117-139`.
- `frontend/src/routes/(app)/bank-import/[id]/+page.svelte`, `bank-import-detail-page.test.ts:171-217` ;
  `frontend/src/routes/(app)/bank-import/profiles/[id]/+page.svelte`, `profile-page.test.ts`.
- `frontend/src/lib/features/contacts/ContactPersonsManager.svelte` ; `BankProfileList.svelte:30`,
  `BankProfileList.test.ts:57-58` ; `crates/kesh-db/src/repositories/contact_persons.rs:158`.
- `docs/manual/fr/user-manual.tex:869-874`, `:2043-2060` ; `docs/api-external.md:594`.
- Fiches voisines : `/home/gcorbaz/devel/kesh-29-3/…/29-3-meilleure-proposition.md`,
  `/home/gcorbaz/devel/kesh-29-6/…/29-6-trop-percu-centimes.md`, `/home/gcorbaz/devel/kesh-29-8/…/29-8-transaction-reglee-ailleurs.md`.
- Précédent de forme : `28-8-petits-correctifs.md`. Consignes : `docs/agents-orchestration.md` ; rétrospective
  `epic-28-retro-2026-10-14.md` § 4, § 7.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-14 — Fiche créée (`bmad-create-story`, Opus 5.5, autonomie), sur `fcd2ae75`. Prémisse du second cas de
  #655 rectifiée (défaut latent, C-29-9-8). Choix C-29-9-1 à 12.

- 2026-10-14 — **Validation P1 remédiée** (Opus 5.5 ; passe Sonnet, lentilles R et F, rapports
  `kesh-gate-logs/29-9-validate-p1-{R,F}.md`). Bilan de la passe : R **0/0/4/5**, F **0/0/5/6** (CRITICAL/HIGH/MEDIUM/LOW),
  soit 20 findings dont trois paires convergentes (R1=F1, R2=F2, R3=F3) : **17 défauts distincts**, 6 MEDIUM et 11 LOW.
  **Tous traités, aucun refus.**
  - R1/F1 (MEDIUM) — mock de `./reconciliation.api` passé à `importOriginal` **avant** V1-V5 (AC9 étape 0, T1).
  - R2/F2 (MEDIUM) — `contact-persons-confirm-delete` entre dans `KNOWN_VIOLATIONS` (AC8, T5) ; dette #30 +1.
  - R3/F3 (MEDIUM) — commentaire de `reconciliation.types.ts:52-55` réécrit (AC4) ; grep de propagation étendu à
    `crates frontend/src docs` ; occurrence `kesh-db` triée et laissée (C-29-9-16).
  - F4 (MEDIUM, décision de l'orchestrateur) — prémisse de C-29-9-8 **fausse au code** (SvelteKit réassigne
    `page.params` à chaque navigation), corrigée par annotation datée (fiche et registre) ; AC6 réécrit : `$derived` de
    la chaîne `page.params.id`, remise à zéro et `isLoading = true` au début d'un chargement réel, établi sans effet sur
    le formulaire à id égal ; fixture qui réassigne `params` ; V13 (même id : rien ne se relance) ; C-29-9-13.
  - F5 (MEDIUM) et R8 (LOW) — V14/M13 (`isLoading = true`), V15/M16 (garde sur `isLoading`) ; `loading` du détail non
    gardé, sans effet observable (C-29-9-14, avec F7).
  - R4 (MEDIUM) — zones partagées complétées (29-3 `:2049`/`:2056`, 29-6 `:2056-2057`/`:2901`/`:2910-2911`, 29-8
    `:2021`/`:2057`) ; AC8 `:2057` écrit en delta sur la phrase rebasée.
  - LOW : R5 (C-29-9-12 corrigé par annotation : 501, 150e) ; R6 (gardes i18n nommées, apostrophe du repli) ; R7
    (séquence avant la branche « ID invalide », V16/M17, et au détail) ; R9 (numéros de ligne `:112-119`, `:290-298`) ;
    F6 (V12/M14) ; F7 (C-29-9-14) ; F8 (second test Rust à **exactement 500**, M18) ; F9 (saut de défilement
    **déclaré**, C-29-9-15) ; F10 (ordre de l'assertion de V3, `beforeEach` de la fixture) ; F11 (entrées CAMT
    uniques, 501 vérifiées à l'import ; « dates antérieures » retiré, `seedTestState` remet la base à zéro).
  - Recomptes à la source après remédiation : 10 AC, T0-T9 (10), **V1-V16** (16 ; +5 : V12-V16), **M1-M18** (18 ;
    +6 : M13-M18), choix **C-29-9-1 à 16** (+4 : 13-16). Symptômes grepés : `pas de UI dédiée|Charger plus|L24` (4
    sites : deux doc-comments Rust prescrits, `reconciliation.types.ts:54` désormais prescrit, `kesh-db` trié) ;
    `101` dans fiche et registre (C-29-9-12 seul, corrigé) ; `ne se rejoue que si` (fiche et registre, annotés) ;
    `111-119`, `291-299` (aucun résidu).

- 2026-10-14 — **Validation P2 remédiée** (Opus 5.5 ; passe Opus, lentilles R et F, rapports
  `kesh-gate-logs/29-9-validate-p2-{R,F}.md`). Bilan de la passe : R **0/0/1/8**, F **0/0/2/7**, soit 18 findings dont
  quatre paires convergentes (R1=F1, R2=F3, R3=F4, R5=F9) : **14 défauts distincts**, 2 MEDIUM et 12 LOW. Tendance :
  P1 6 MEDIUM → P2 2 MEDIUM. **Tous traités, aucun refus** ; décisions de l'orchestrateur consignées comme telles.
  - R1 = F1 (MEDIUM, **décision de l'orchestrateur**) — le commentaire de module `kesh-db/src/repositories/
    reconciliation.rs:18-19`, faux depuis la 8-4, est **réécrit en T1** (AC4) ; C-29-9-16 annoté et renversé.
    `kesh-db` entre dans le diff pour un commentaire : l'exception *kesh-db* ne vaut que pour le commit de T1 (et un
    patch de revue qui retoucherait ce fichier), gate complet à ce commit-là (AC10, T1) ; module de plus déclaré
    (§ *Dérogation*, C-29-9-1 annoté).
  - F2 (MEDIUM, **décision de l'orchestrateur**) — `onUpdated` gardé (AC6, C-29-9-17), V17, M19. ⚠️ La décision dit
    « la même garde de séquence que le chargement » ; sa forme littérale (numéro capturé dans une fermeture passée au
    formulaire) ne tient pas, le formulaire lisant ses props en direct (`$$props.onSuccess`, établi par compilation d'un
    composant d'essai sous Svelte 5.55.1). La garde est donc évaluée à l'appel, sur l'identifiant courant
    (`updated.id !== Number(rawId)`) — même effet voulu : une sauvegarde qui finit après une navigation vers un autre
    profil n'écrit plus `profile`. Signalé à l'orchestrateur.
  - LOW : R2 = F3 (la mutation `limit` pour `limit + 1` n'est pas tuée par le test à 500 mais par celui à 501 ; phrase
    corrigée, mutation inscrite M20) ; R3 = F4 (`:627` → `:628`) ; R4 (repli `reconciliation.limits.ts` : import direct
    par le composant ; `importOriginal` fait entrer `api-client` et ses imports, écrit à l'étape 0) ; R5 = F9
    (`KNOWN_VIOLATIONS` : bloc `:117-139`, insertion après `:139`, deux sites) ; R6 (trois zones partagées ajoutées :
    `api-external.md`, `reconciliation.api.ts`/`.types.ts`, `ReconciliationProposals.test.ts` ; étape 0 et `:594` en
    delta sur le texte rebasé) ; R7 (`sprint-status.yaml` : « 1 à 18 ») ; R8 (bouton désactivé pendant un lot : V18,
    M21) ; R9 (liste des occurrences légitimes attendues après la story, et de celles qui ne doivent plus sortir) ; F5
    (`page_changed`, `client.js:763-772` : formulation exacte, cinq sites vivants corrigés, deux annotations datées) ;
    F6 (commentaire au site plutôt qu'`untrack`, C-29-9-18) ; F7 (V1 et V12 verts au sol, « V2-V5 rougissent ») ; F8
    (avis et bouton immédiatement après `</table>`, avant toute modale dont celle de la 29-8 ; C-29-9-11 annoté).
  - Recomptes à la source après remédiation : 10 AC, T0-T9 (10), **V1-V18** (18 ; +2 : V17, V18), **M1-M21** (21 ;
    +3 : M19-M21), choix **C-29-9-1 à 18** (+2 : 17, 18). Symptômes grepés : `chaque navigation|toute navigation`
    (fiche et registre ; sites vivants corrigés, entrées datées annotées, Change Log P1 laissé) ; `:627`, `117-133`
    (aucun résidu vivant) ; `reste vrai` (registre C-29-9-16, annoté et renversé) ; `V1-V5` (étape 0 corrigée ; Change
    Log P1 laissé) ; `C-29-9-1 à 12` (`sprint-status.yaml`, corrigé).

[#646]: https://github.com/guycorbaz/kesh/issues/646
[#649]: https://github.com/guycorbaz/kesh/issues/649
[#655]: https://github.com/guycorbaz/kesh/issues/655
- 2026-10-14 — **Validation P3, passe ciblée** (Haiku 5.5, une lentille sur `13ac02c5` ; rapport
  `kesh-gate-logs/29-9-validate-p3-ciblee.md`) : 0/0/0/1. **Validation close.** Le LOW est à appliquer à l'étape 0 du
  développement : P3-1 (l. 506, les citations « 29-3 :549 » et « 29-6 :551 » désignent `docs/api-external.md`, pas les
  fiches — écrire « `docs/api-external.md:549`, cité par la fiche 29-3 l. 534 », idem 29-6 l. 347). Trend : P1 R 0/0/4/5
  F 0/0/5/6 → P2 R 0/0/1/8 F 0/0/2/7 → P3 ciblée 0/0/0/1.
